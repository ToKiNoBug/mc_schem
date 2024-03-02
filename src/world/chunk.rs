use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time;
use fastnbt::Value;
use math::round::{ceil, floor};
use crate::error::Error;
use crate::region::{Light, Region};
use crate::schem::common::ceil_up_to;
use crate::{unwrap_opt_tag, unwrap_tag};
use crate::schem::common;
use crate::schem::id_of_nbt_tag;
use crate::world::{Chunk, ChunkStatus};

impl Chunk {
    pub fn new() -> Chunk {
        return Chunk {
            time_stamp: time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as u32,
            sub_chunks: [Region::new(), Region::new(), Region::new(), Region::new(), Region::new(),
                Region::new(), Region::new(), Region::new(), Region::new(), Region::new(),
                Region::new(), Region::new(), Region::new(), Region::new(), Region::new(),
                Region::new(), Region::new(), Region::new(), Region::new(), Region::new(),
                Region::new(), Region::new(), Region::new(), Region::new(), Region::new(), ],
        };
    }
    pub fn from_nbt(mut nbt: HashMap<String, Value>, path_in_saves: String) -> Result<Chunk, Error> {
        let mut result = Chunk::new();


        !todo!()
    }
}

fn parse_section(sect: &HashMap<String, Value>, path: &str) -> Result<Region, Error> {
    let mut reg = Region::with_shape([16, 16, 16]);
    let block_states = unwrap_opt_tag!(sect.get("block_states"),Compound,HashMap::new(),format!("{path}/block_states"));

    // palette
    {
        let palette = unwrap_opt_tag!(block_states.get("palette"),List,vec![],format!("{path}/block_states/palette"));
        let mut pal = Vec::with_capacity(palette.len());
        for (idx, blk) in palette.iter().enumerate() {
            let path = format!("{path}/block_states/palette[{idx}]");
            let blk = unwrap_tag!(blk,Compound,HashMap::new(),path);
            let blk = common::parse_block(blk, &path)?;
            pal.push(blk);
        }
        reg.palette = pal;
    }
    if reg.palette.len() <= 0 {
        return Err(Error::PaletteIsEmpty { tag_path: format!("{path}/block_states/palette") });
    }
    if reg.palette.len() > 65535 {
        return Err(Error::PaletteTooLong(reg.palette.len()));
    }
    // blocks
    if reg.palette.len() == 1 {
        reg.array_yzx.fill(0);
    } else {
        let path = format!("{path}/block_states/data");
        let array_i64 = unwrap_opt_tag!(block_states.get("data"),LongArray,fastnbt::LongArray::new(vec![]),path);

        let block_id_max = reg.palette.len();
        let bits_per_block = ceil((block_id_max as f64).log2(), 0) as u8;
        let mut mbs = MultiBitSet::new(4096, bits_per_block);

        if array_i64.len() != mbs.num_u64() {
            return Err(Error::InvalidValue {
                tag_path: path,
                error: format!("This subchunk has 4096 blocks of {} types, required {} i64 element to store them, but found {}",
                               reg.palette.len(), mbs.num_u64(), array_i64.len()),
            });
        }
        mbs.set_array_from_nbt(&array_i64);

        let mut counter = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let blk_id = mbs.get(counter) as u16;
                    if blk_id > block_id_max as u16 {
                        return Err(Error::BlockIndexOutOfRange {
                            tag_path: format!("{path}/block_states/data"),
                            index: blk_id as i32,
                            range: [0, block_id_max as i32],
                        });
                    }
                    reg.array_yzx[[x, y, z]] = blk_id;
                    counter += 1;
                }
            }
        }
    }
    // skylight and block light
    {
        let sky_light = if let Some(s) = sect.get("SkyLight") {
            let tag_path = format!("{path}/SkyLight");
            let arr = unwrap_tag!(s,ByteArray,fastnbt::ByteArray::new(vec![]),tag_path).as_ref();
            if arr.len() != 2048 {
                return Err(Error::InvalidValue { tag_path, error: format!("The length should be 2048, but found {}", arr.len()) });
            }
            arr
        } else {
            &[]
        };
        let block_light = if let Some(s) = sect.get("BlockLight") {
            let tag_path = format!("{path}/BlockLight");
            let arr = unwrap_tag!(s,ByteArray,fastnbt::ByteArray::new(vec![]),tag_path).as_ref();
            if arr.len() != 2048 {
                return Err(Error::InvalidValue { tag_path, error: format!("The length should be 2048, but found {}", arr.len()) });
            }
            arr
        } else {
            &[]
        };

        let mut counter = 0;
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    let sl: u8 = if sky_light.is_empty() {
                        15
                    } else {
                        let b = u8::from_ne_bytes(sky_light[counter / 2].to_ne_bytes());
                        b >> (4 * (counter % 2))
                    };
                    debug_assert!(sl <= 15);
                    let bl: u8 = if block_light.is_empty() {
                        15
                    } else {
                        let b = u8::from_ne_bytes(block_light[counter / 2].to_ne_bytes());
                        b >> (4 * (counter % 2))
                    };
                    debug_assert!(bl <= 15);

                    let light = Light::new(sl, bl);
                    reg.sky_block_light[[y, z, x]] = light;
                    counter += 1;
                }
            }
        }
    }

    //biomes
    !todo!();

    return Ok(reg);
}

impl Display for ChunkStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "minecraft:{}", self.name_without_namespace());
    }
}

impl ChunkStatus {
    pub fn name_without_namespace(&self) -> &'static str {
        return match self {
            ChunkStatus::Empty => "empty",
            ChunkStatus::StructureStarts => "structure_starts",
            ChunkStatus::StructureReferences => "structure_references",
            ChunkStatus::Biomes => "biomes",
            ChunkStatus::Noise => "noise",
            ChunkStatus::Surface => "surface",
            ChunkStatus::Carvers => "carvers",
            ChunkStatus::Features => "features",
            ChunkStatus::InitializeLight => "initialize_light",
            ChunkStatus::Light => "light",
            ChunkStatus::Spawn => "spawn",
            ChunkStatus::Full => "full",
        };
    }

    fn all() -> &'static [ChunkStatus] {
        return &[
            ChunkStatus::Empty,
            ChunkStatus::StructureStarts,
            ChunkStatus::StructureReferences,
            ChunkStatus::Biomes,
            ChunkStatus::Noise,
            ChunkStatus::Surface,
            ChunkStatus::Carvers,
            ChunkStatus::Features,
            ChunkStatus::InitializeLight,
            ChunkStatus::Light,
            ChunkStatus::Spawn,
            ChunkStatus::Full, ];
    }

    pub fn from_str(str: &str) -> Option<ChunkStatus> {
        let without_namespace: &str;
        if str.starts_with("minecraft:") {
            without_namespace = &str[10..str.len()];
        } else {
            without_namespace = str;
        }

        for cs in Self::all() {
            if str == cs.name_without_namespace() {
                return Some(*cs);
            }
        }
        return None;
    }
}

struct MultiBitSet {
    array: Vec<u64>,
    num_elements: usize,
    element_bits: u8,
}

impl MultiBitSet {
    pub fn num_element_per_u64(element_bits: u8) -> u8 {
        return floor(64.0 / element_bits as f64, 0) as u8;
    }

    pub fn required_num_u64(num_elements: usize, element_bits: u8) -> usize {
        let num_per_u64 = Self::num_element_per_u64(element_bits) as isize;
        (ceil_up_to(num_elements as isize, num_per_u64) / num_per_u64) as usize
    }
    pub fn new(len: usize, element_bits: u8) -> Self {
        assert!(element_bits < 64);
        let mut result = Self {
            array: Vec::new(),
            num_elements: len,
            element_bits,
        };
        result.reset(len, element_bits);
        return result;
    }
    pub fn reset(&mut self, num_elements: usize, element_bits: u8) {
        assert!(element_bits < 64);
        self.element_bits = element_bits;
        self.num_elements = num_elements;
        self.array.resize(Self::required_num_u64(num_elements, element_bits), 0);
    }

    pub fn index_of_element(&self, ele_idx: usize) -> (usize, u8) {
        let num_per_u64 = Self::num_element_per_u64(self.element_bits) as usize;
        let u64_idx = ele_idx / num_per_u64;
        let bit_index_beg: u8 = ((ele_idx % num_per_u64) * self.element_bits as usize) as u8;
        debug_assert!(bit_index_beg + self.element_bits <= 64);
        return (u64_idx, bit_index_beg);
    }

    pub fn mask(element_bits: u8, bit_index_beg: u8) -> u64 {
        assert!(element_bits < 64);
        let mut mask = (1u64 << element_bits) - 1;
        return mask << bit_index_beg;
    }

    pub fn get(&self, ele_idx: usize) -> u64 {
        let (u64_idx, bit_index_beg) = self.index_of_element(ele_idx);
        let mask = Self::mask(self.element_bits, bit_index_beg);
        return (self.array[u64_idx] & mask) >> bit_index_beg;
    }

    pub fn set(&mut self, ele_idx: usize, value: u64) {
        debug_assert!(value <= Self::mask(self.element_bits, 0));
        let value = value & Self::mask(self.element_bits, 0);

        let (u64_idx, bit_index_beg) = self.index_of_element(ele_idx);
        let inv_mask = !Self::mask(self.element_bits, bit_index_beg);
        self.array[u64_idx] &= inv_mask;
        self.array[u64_idx] |= value << bit_index_beg;

        debug_assert!(self.get(ele_idx) == value);
    }

    pub fn num_u64(&self) -> usize {
        return self.array.len();
    }

    pub fn set_array_from_nbt(&mut self, i64_ne: &[i64]) {
        self.array.clear();
        self.array.reserve(i64_ne.len());

        for val in i64_ne {
            let val = u64::from_be_bytes(val.to_ne_bytes());
            self.array.push(val);
        }
    }
}

#[test]
fn test_multi_bit_set() {
    let bits = 63;
    let len = 8192;

    let mut vec = MultiBitSet::new(len, bits);
    for i in 0..len {
        vec.set(i, (i as u64) & (1u64 << bits - 1));
    }
    for i in 0..len {
        let expected = (i as u64) & (1u64 << bits - 1);
        assert_eq!(vec.get(i), expected);
    }
}