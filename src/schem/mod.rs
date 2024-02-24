pub mod world_edit12;
pub mod world_edit13;
pub mod litematica;

pub mod vanilla_structure;
pub mod mc_version;
pub mod common;


use std::cmp::max;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::block::{Block, CommonBlock};
use fastnbt;
use flate2::Compression;
use crate::error::{Error};
//use schem::mc_version;
use crate::{PendingTick, schem};
use crate::region::{BlockEntity, Region};


pub type DataVersion = mc_version::DataVersion;


#[derive(Debug, Clone)]
pub struct LitematicaMetaData {
    pub data_version: i32,

    pub version: i32,
    pub sub_version: Option<i32>,
    pub time_created: i64,
    pub time_modified: i64,
    pub author: String,
    pub name: String,
    pub description: String,
    //pub total_volume: i64,
}

#[allow(dead_code)]
impl LitematicaMetaData {
    // pub fn new() -> LitematicaMetaData {
    //     return Self::default();
    // }

    pub fn default() -> LitematicaMetaData {
        return Self::from_data_version(DataVersion::Java_1_20_4).unwrap();
    }

    pub fn data_version_to_lite_version(data_version: i32) -> Option<i32> {
        return if data_version < DataVersion::Java_1_12 as i32 {
            None
        } else if data_version < DataVersion::Java_1_13 as i32 {
            Some(4)
        } else if data_version < DataVersion::Java_1_18 as i32 {
            Some(5)
        } else {
            Some(6)
        };
    }

    pub fn data_version_to_lite_subversion(data_version: i32) -> Option<i32> {
        return if data_version < DataVersion::Java_1_18 as i32 {
            None
        } else {
            Some(1)
        };
    }

    pub fn from_data_version_i32(data_version: i32) -> Result<LitematicaMetaData, Error> {
        use std::time::{SystemTime, UNIX_EPOCH};
        if data_version < DataVersion::Java_1_12 as i32 {
            return Err(Error::UnsupportedVersion { data_version_i32: data_version });
        }
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let result = LitematicaMetaData {
            data_version,
            version: Self::data_version_to_lite_version(data_version).unwrap_or(-1),
            sub_version: Self::data_version_to_lite_subversion(data_version),
            time_created: time,
            time_modified: time,
            author: String::from("mc_schem.rs"),
            name: String::from("Default litematica"),
            description: String::from("Default litematica generated by mc_schem.rs"),
            //total_volume: 0,
        };
        return Ok(result);
    }

    pub fn from_data_version(data_version: DataVersion) -> Result<LitematicaMetaData, Error> {
        return Self::from_data_version_i32(data_version as i32);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WE12MetaData {
    pub materials: String,
    pub we_offset: [i32; 3],
    pub we_origin: [i32; 3],
}

#[allow(dead_code)]
impl WE12MetaData {
    pub fn default() -> WE12MetaData {
        return WE12MetaData {
            materials: "Alpha".to_string(),
            we_offset: [0, 0, 0],
            we_origin: [0, 0, 0],
        };
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WE13MetaData {
    pub data_version: i32,
    pub version: i32,
    pub we_offset: [i32; 3],
    pub offset: [i32; 3],
    //time stamp in milliseconds
    pub date: Option<i64>,
    pub v3_extra: Option<WE13MetaDataV3Extra>,
}

// Introduced in 1.20, version 3
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WE13MetaDataV3Extra {
    pub world_edit_version: String,
    pub editing_platform: String,
    pub origin: [i32; 3],
}

impl Default for WE13MetaData {
    fn default() -> WE13MetaData {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        return WE13MetaData {
            data_version: DataVersion::new() as i32,
            version: 5,
            we_offset: [0, 0, 0],
            offset: [0, 0, 0],
            date: Some(time),
            v3_extra: None,
        };
    }
}

impl Default for WE13MetaDataV3Extra {
    fn default() -> Self {
        return WE13MetaDataV3Extra {
            world_edit_version: "(unknown)".to_string(),
            editing_platform: "".to_string(),
            origin: [0, 0, 0],
        }
    }
}

#[allow(dead_code)]
impl WE13MetaData {
    pub fn from_data_version(dv: DataVersion) -> Result<WE13MetaData, Error> {
        return Self::from_data_version_i32(dv as i32);
    }

    pub fn from_data_version_i32(dv: i32) -> Result<WE13MetaData, Error> {
        let mut result = Self::default();
        result.data_version = dv;
        if dv < DataVersion::Java_1_13 as i32 {
            return Err(Error::UnsupportedVersion { data_version_i32: dv });
        }
        // 1.13.2 => 2
        // 1.14.4 => 2
        // 1.18.2 => 2
        // 1.19.4 => 2
        // 1.20.2 => 3
        result.version = if dv < DataVersion::Java_1_20 as i32 {
            2
        } else {
            3
        };

        return Ok(result);
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct VanillaStructureMetaData {
    pub data_version: i32,
}

#[allow(dead_code)]
impl VanillaStructureMetaData {
    pub fn default() -> VanillaStructureMetaData {
        return VanillaStructureMetaData {
            data_version: DataVersion::new() as i32,
        };
    }


    pub fn from_data_version(dv: DataVersion) -> Result<VanillaStructureMetaData, Error> {
        return Self::from_data_version_i32(dv as i32);
    }

    pub fn from_data_version_i32(dv: i32) -> Result<VanillaStructureMetaData, Error> {
        return Ok(VanillaStructureMetaData {
            data_version: dv
        });
    }
}


#[derive(Debug)]
pub enum RawMetaData {
    Litematica(LitematicaMetaData),
    WE12(WE12MetaData),
    WE13(WE13MetaData),
    VanillaStructure(VanillaStructureMetaData),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MetaDataIR {
    pub mc_data_version: i32,

    pub time_created: i64,
    pub time_modified: i64,
    pub author: String,
    pub name: String,
    pub description: String,

    pub litematica_version: i32,
    pub litematica_subversion: Option<i32>,

    pub schem_version: i32,
    pub schem_offset: [i32; 3],
    pub schem_we_offset: Option<[i32; 3]>,

    pub date: Option<i64>,

    pub schem_world_edit_version: Option<String>,
    pub schem_editing_platform: Option<String>,
    pub schem_origin: Option<[i32; 3]>,
    pub schem_material: String,
    //pub raw_metadata: Option<MetaData>,
}

#[allow(dead_code)]
impl MetaDataIR {
    pub fn default() -> MetaDataIR {
        return Self::from_data_version(DataVersion::new()).unwrap();
    }

    pub fn from_data_version(version: DataVersion) -> Result<MetaDataIR, Error> {
        return Self::from_data_version_i32(version as i32);
    }

    pub fn from_data_version_i32(version: i32) -> Result<MetaDataIR, Error> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

        let result = MetaDataIR {
            mc_data_version: version,
            time_created: time,
            time_modified: time,
            author: String::from("mc_schem"),
            name: String::from("DefaultMetaDataIR"),
            description: String::from("Default metadata generated by mc_schem"),
            litematica_version: LitematicaMetaData::default().version,
            litematica_subversion: LitematicaMetaData::default().sub_version,
            schem_version: WE13MetaData::default().version,
            schem_offset: [0, 0, 0],
            schem_we_offset: Some([0, 0, 0]),
            date: Some(time),
            schem_world_edit_version: None,
            schem_editing_platform: None,
            schem_origin: Some([0, 0, 0]),
            schem_material: "Alpha".to_string(),
        };
        return Ok(result);
    }
}

#[derive(Debug)]
pub struct Schematic {
    pub metadata: MetaDataIR,

    pub regions: Vec<Region>,
    //pub enclosing_size: [i64; 3],

}


// enum SchemFormat {
//     Litematica,
//     WorldEdit12,
//     WorldEdit13,
//     VanillaStructure,
// }
#[allow(dead_code)]
impl Schematic {
    pub fn new() -> Schematic {
        return Schematic {
            //data_version: mc_version::DataVersion::new() as i32,
            metadata: MetaDataIR::default(),
            regions: Vec::new(),
            //enclosing_size: [1, 1, 1],

        };
    }

    // pub fn metadata(&self) -> &MetaDataIR {
    //     return &self.metadata;
    // }
    //
    // pub fn set_metadata(&mut self, md: MetaDataIR) {
    //     self.metadata = md;
    // }
    //
    // pub fn regions(&self) -> &[Region] {
    //     return &self.regions;
    // }
    //
    // pub fn regions_mut(&mut self) -> &mut Vec<Region> {
    //     return &mut self.regions;
    // }

    pub fn block_indices_at(&self, g_pos: [i32; 3]) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(g_pos);
            if let Some(blk) = reg.block_index_at(cur_pos) {
                result.push(blk);
            }
        }
        return result;
    }

    pub fn blocks_at(&self, pos: [i32; 3]) -> Vec<&Block> {
        let mut result = Vec::new();
        self.get_blocks_at(pos, &mut result);
        return result;
    }

    pub fn get_blocks_at<'a>(&'a self, pos: [i32; 3], dest: &mut Vec<&'a Block>) {
        dest.clear();
        dest.reserve(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(pos);
            if let Some(blk) = reg.block_at(cur_pos) {
                dest.push(blk);
            }
        }
    }

    pub fn block_entities_at(&self, pos: [i32; 3]) -> Vec<&BlockEntity> {
        let mut result = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let cur_pos = reg.global_pos_to_relative_pos(pos);
            if let Some(blk) = reg.block_entities.get(&cur_pos) {
                result.push(blk);
            }
        }
        return result;
    }

    pub fn first_region_index_at(&self, pos: [i32; 3]) -> Option<usize> {
        for (idx, reg) in self.regions.iter().enumerate() {
            let r_pos = reg.global_pos_to_relative_pos(pos);
            if reg.contains_coord(r_pos) {
                return Some(idx);
            }
        }
        return None;
    }


    pub fn first_block_index_at(&self, pos: [i32; 3]) -> Option<u16> {
        for reg in &self.regions {
            if let Some(bid) = reg.block_index_at(reg.global_pos_to_relative_pos(pos)) {
                return Some(bid);
            }
        }
        return None;
    }
    pub fn first_block_at(&self, pos: [i32; 3]) -> Option<&Block> {
        for reg in &self.regions {
            if let Some(b) = reg.block_at(reg.global_pos_to_relative_pos(pos)) {
                return Some(b);
            }
        }
        return None;
    }
    pub fn first_block_entity_at(&self, pos: [i32; 3]) -> Option<&BlockEntity> {
        for reg in &self.regions {
            if let Some(b) = reg.block_entities.get(&reg.global_pos_to_relative_pos(pos)) {
                return Some(b);
            }
        }
        return None;
    }

    pub fn first_block_info_at(&self, pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)> {
        for reg in &self.regions {
            let r_pos = reg.global_pos_to_relative_pos(pos);
            if !reg.contains_coord(r_pos) {
                continue;
            }
            if let Some(info) = reg.block_info_at(r_pos) {
                return Some(info);
            }
        }
        return None;
    }

    pub fn shape(&self) -> [i32; 3] {
        let mut result = [0, 0, 0];
        for reg in &self.regions {
            for dim in 0..3 {
                result[dim] = max(result[dim], reg.offset[dim] + reg.shape()[dim]);
            }
        }
        return result;
    }

    pub fn volume(&self) -> u64 {
        let mut result: u64 = 1;
        for sz in self.shape() {
            result *= sz as u64;
        }
        return result;
    }

    pub fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;
        for reg in &self.regions {
            counter += reg.total_blocks(include_air);
        }
        return counter;
    }


    // (Vec<(block, hash)>, Vec<LUT-cur-block-index-to-global-block-index>)
    pub fn full_palette(&self) -> (Vec<(&Block, u64)>, Vec<Vec<usize>>) {
        let possible_max_palette_size;
        {
            let mut pmps: usize = 0;
            for reg in &self.regions {
                pmps = max(pmps, reg.palette.len());
            }
            possible_max_palette_size = pmps;
        }

        let mut palette: Vec<(&Block, u64)> = Vec::with_capacity(possible_max_palette_size);
        let mut lut_lut: Vec<Vec<usize>> = Vec::with_capacity(self.regions.len());
        for reg in &self.regions {
            let mut lut: Vec<usize> = Vec::with_capacity(reg.palette.len());

            for cur_blk in &reg.palette {
                let mut hasher = DefaultHasher::new();
                cur_blk.hash(&mut hasher);
                let cur_hash = hasher.finish();

                let mut cur_block_index_in_full_palette = palette.len();
                for (idx, (blk, hash)) in palette.iter().enumerate() {
                    if *hash != cur_hash {
                        continue;
                    }
                    if *blk != cur_blk {
                        continue;
                    }
                    cur_block_index_in_full_palette = idx;
                    break;
                }

                if cur_block_index_in_full_palette >= palette.len() {
                    palette.push((cur_blk, cur_hash));
                }
                lut.push(cur_block_index_in_full_palette);
            }
            lut_lut.push(lut);
        }
        return (palette, lut_lut);
    }


    pub fn from_file(filename: &str) -> Result<(Schematic, RawMetaData), Error> {
        if filename.ends_with(".litematic") {
            let (schem, raw) = Self::from_litematica_file(filename, &LitematicaLoadOption::default())?;
            return Ok((schem, RawMetaData::Litematica(raw)));
        }
        if filename.ends_with(".nbt") {
            let (schem, raw) = Self::from_vanilla_structure_file(filename, &VanillaStructureLoadOption::default())?;
            return Ok((schem, RawMetaData::VanillaStructure(raw)));
        }
        if filename.ends_with(".schem") {
            let (schem, raw) = Self::from_world_edit_13_file(filename, &WorldEdit13LoadOption::default())?;
            return Ok((schem, RawMetaData::WE13(raw)));
        }
        if filename.ends_with(".schematic") {
            let (schem, raw, ..) = Self::from_world_edit_12_file(filename, &WorldEdit12LoadOption::default())?;
            return Ok((schem, RawMetaData::WE12(raw)));
        }

        let split = filename.split(".");
        let extension = split.last().unwrap_or_else(|| "");


        return Err(Error::UnrecognisedExtension { extension: extension.to_string() });
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), Error> {
        if filename.ends_with(".litematic") {
            return self.save_litematica_file(filename, &LitematicaSaveOption::default());
        }
        if filename.ends_with(".nbt") {
            return self.save_vanilla_structure_file(filename, &VanillaStructureSaveOption::default());
        }
        if filename.ends_with(".schem") {
            return self.save_world_edit_13_file(filename, &WorldEdit13SaveOption::default());
        }

        let split = filename.split(".");
        let extension = split.last().unwrap_or_else(|| "");


        return Err(Error::UnrecognisedExtension { extension: extension.to_string() });
    }

    pub fn duplicated_blocks(&self) -> HashMap<[i32; 3], Vec<&Block>> {
        let mut result = HashMap::new();
        let mut temp = Vec::new();

        fn deduplicate<'a>(src: &[&'a Block], dest: &mut Vec<&'a Block>) {
            dest.reserve(src.len());
            dest.clear();
            for blk in src {
                if dest.contains(&*blk) {
                    continue;
                }
                dest.push(&*blk);
            }
        }

        let mut temp_deduplicated = Vec::with_capacity(self.regions.len());

        for y in 0..self.shape()[1] {
            for z in 0..self.shape()[2] {
                for x in 0..self.shape()[0] {
                    let pos = [x, y, z];
                    self.get_blocks_at(pos, &mut temp);
                    deduplicate(&temp, &mut temp_deduplicated);
                    if temp_deduplicated.len() >= 2 {
                        result.insert(pos, temp_deduplicated.clone());
                    }
                }
            }
        }
        return result;
    }

    pub fn to_single_region(&self, background_block: &Block) -> Region {
        let mut region = Region::new();
        region.reshape(&self.shape());
        {
            let mut entity_num = 0;
            let mut be_num = 0;
            let mut pb_num = 0;
            for reg in &self.regions {
                entity_num += reg.entities.len();
                be_num += reg.block_entities.len();
                pb_num += reg.pending_ticks.len();
            }
            region.entities.reserve(entity_num);
            region.pending_ticks.reserve(pb_num);
            region.block_entities.reserve(be_num);
        }

        let (full_pal, lut_lut) = self.full_palette();
        let background_block_index;
        {
            region.palette.clear();
            region.palette.reserve(full_pal.len() + 1);
            for (blk, _hash) in &full_pal {
                region.palette.push((*blk).clone());
            }
            background_block_index = region.find_or_append_to_palette(background_block);
        }
        let shape = self.shape();
        for y in 0..shape[1] {
            for z in 0..shape[2] {
                for x in 0..shape[0] {
                    let g_pos = [x, y, z];
                    {
                        let res = region.set_block_id(g_pos, background_block_index);
                        debug_assert!(res.is_ok());
                    }
                    let reg_idx = self.first_region_index_at(g_pos);
                    let info = self.first_block_info_at(g_pos);
                    debug_assert!(reg_idx.is_some() == info.is_some());
                    let reg_idx = match reg_idx {
                        Some(ri) => ri,
                        None => continue,
                    };
                    let (local_block_idx, _blk, be_opt, pd_opt) = info.unwrap();
                    let global_block_idx = lut_lut[reg_idx][local_block_idx as usize] as u16;
                    {
                        let res = region.set_block_id(g_pos, global_block_idx);
                        debug_assert!(res.is_ok());
                    }
                    if let Some(be) = be_opt {
                        region.block_entities.insert(g_pos, be.clone());
                    }
                    if let Some(pb) = pd_opt {
                        region.pending_ticks.insert(g_pos, pb.clone());
                    }
                }
            }
        }

        // entities
        {
            for reg in &self.regions {
                for entity in &reg.entities {
                    let mut e = entity.clone();
                    e.pos_shift(reg.offset);
                    region.entities.push(e);
                }
            }
        }

        return region;
    }

    pub fn merge_regions(&mut self, background_block: &Block) {
        let new_reg = self.to_single_region(background_block);
        self.regions = vec![new_reg];
    }
}

pub fn id_of_nbt_tag(tag: &fastnbt::Value) -> u8 {
    return match tag {
        fastnbt::Value::Byte(_) => 1,
        fastnbt::Value::Short(_) => 2,
        fastnbt::Value::Int(_) => 3,
        fastnbt::Value::Long(_) => 4,
        fastnbt::Value::Float(_) => 5,
        fastnbt::Value::Double(_) => 6,
        fastnbt::Value::ByteArray(_) => 7,
        fastnbt::Value::String(_) => 8,
        fastnbt::Value::List(_) => 9,
        fastnbt::Value::Compound(_) => 10,
        fastnbt::Value::IntArray(_) => 11,
        fastnbt::Value::LongArray(_) => 12,
    }
}

#[derive(Debug)]
pub struct VanillaStructureLoadOption {
    pub background_block: CommonBlock,
}

impl VanillaStructureLoadOption {
    pub fn default() -> VanillaStructureLoadOption {
        return VanillaStructureLoadOption {
            background_block: CommonBlock::StructureVoid
        }
    }
}

#[derive(Debug)]
pub struct VanillaStructureSaveOption {
    pub compress_level: Compression,
    pub keep_air: bool,
}

impl Default for VanillaStructureSaveOption {
    fn default() -> VanillaStructureSaveOption {
        return VanillaStructureSaveOption {
            keep_air: true,
            compress_level: Compression::best(),
        }
    }
}

//#[derive(Debug)]
pub struct LitematicaLoadOption {
}

impl LitematicaLoadOption {
    pub fn default() -> LitematicaLoadOption {
        return LitematicaLoadOption {
        };
    }
}


#[derive(Debug)]
pub struct LitematicaSaveOption {
    pub compress_level: Compression,
    pub rename_duplicated_regions: bool,
}

impl Default for LitematicaSaveOption {
    fn default() -> LitematicaSaveOption {
        return LitematicaSaveOption {
            rename_duplicated_regions: true,
            compress_level: Compression::best(),
        };
    }
}


#[derive(Debug)]
pub struct WorldEdit13LoadOption {}

#[allow(dead_code)]
impl WorldEdit13LoadOption {
    pub fn default() -> WorldEdit13LoadOption {
        return WorldEdit13LoadOption {};
    }
}

#[derive(Debug)]
pub struct WorldEdit13SaveOption {
    pub compress_level: Compression,
    pub background_block: CommonBlock,
}

#[allow(dead_code)]
impl Default for WorldEdit13SaveOption {
    fn default() -> WorldEdit13SaveOption {
        return WorldEdit13SaveOption {
            background_block: CommonBlock::Air,
            compress_level: Compression::best(),
        };
    }
}

#[derive(Debug)]
pub struct WorldEdit12LoadOption {
    pub data_version: DataVersion,
    pub fix_string_id_with_block_entity_data: bool,
    pub discard_number_id_array: bool,
}

impl Default for WorldEdit12LoadOption {
    fn default() -> Self {
        return WorldEdit12LoadOption {
            data_version: DataVersion::Java_1_12_2,
            fix_string_id_with_block_entity_data: true,
            discard_number_id_array: false,
        }
    }
}

