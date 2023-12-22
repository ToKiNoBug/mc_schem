mod world_edit12;
mod world_edit13;
mod litematica;

mod vanilla_structure;

mod schem {
    use std::collections::HashMap;
    use ndarray::Array3;
    use crate::block::Block;
    use nbt;
    use nbt::Blob;
    use crate::schem::schem::MetaData::Litematica;

    pub struct BlockEntity {
        pub tags: nbt::Blob,
    }

    impl BlockEntity {
        pub fn new() -> BlockEntity {
            return BlockEntity {
                tags: nbt::Blob::new(),
            };
        }
    }

    pub struct Entity {
        pub tags: nbt::Blob,
        pub position: [f32; 3],
    }

    impl Entity {
        pub fn new() -> Entity {
            return Entity {
                tags: nbt::Blob::new(),
                position: [0.0, 0.0, 0.0],
            };
        }
    }

    pub struct Region {
        pub name: String,
        array: Array3<u16>,
        //XYZ
        pub palette: Vec<Block>,
        block_entities: HashMap<[i64; 3], BlockEntity>,
        pub entities: Vec<Entity>,

        pub offset: [i64; 3],
    }

    impl Region {
        pub fn new() -> Region {
            return Region {
                name: String::from("NewRegion"),
                array: Array3::zeros([1, 1, 1]),
                palette: Vec::new(),
                block_entities: HashMap::new(),
                entities: Vec::new(),
                offset: [0, 0, 0],
            };
        }

        pub fn shape(&self) -> [i64; 3] {
            let shape = self.array.shape();
            if shape.len() != 3 {
                panic!("Invalid array dimensions: shoule be 3 but now it is {}", shape.len());
            }
            return [shape[0] as i64, shape[1] as i64, shape[2] as i64];
        }
        pub fn contains_coord(&self, coord: [i64; 3]) -> bool {
            for dim in 0..3 {
                if coord[dim] >= 0 && coord[dim] <= self.shape()[dim] {
                    continue;
                }
                return false;
            }
            return true;
        }
        pub fn block_at(&self, coord: [i64; 3]) -> Option<&Block> {
            if !self.contains_coord(coord) {
                return None;
            }

            let x = coord[0] as usize;
            let y = coord[1] as usize;
            let z = coord[2] as usize;

            let pid = self.array[[x, y, z]] as usize;
            return Some(&self.palette[pid]);
        }
    }

    pub struct LitematicaMetaData {
        pub version: i32,

        pub time_created: i64,
        pub time_modified: i64,
        pub author: String,
        pub name: String,
        pub description: String,
        pub total_volume: i64,
    }

    impl LitematicaMetaData {
        pub fn new() -> LitematicaMetaData {
            return LitematicaMetaData {
                version: 5,
                time_created: 0,
                time_modified: 0,
                author: String::from("mc_schem.rs"),
                name: String::from("Default litematica"),
                description: String::from("Default litematica generated by mc_schem.rs"),
                total_volume: 0,
            };
        }
    }

    pub struct WE12MetaData {}

    impl WE12MetaData {
        pub fn new() -> WE12MetaData {
            return WE12MetaData {};
        }
    }

    pub struct WE13MetaData {
        pub version: i32,
        pub we_offset: [i32; 3],
        pub offset: [i32; 3],
    }

    impl WE13MetaData {
        pub fn new() -> WE13MetaData {
            return WE13MetaData {
                version: 5,
                we_offset: [0, 0, 0],
                offset: [0, 0, 0],
            }
        }
    }

    pub struct VanillaStructureMetaData {}

    impl VanillaStructureMetaData {
        pub fn new() -> VanillaStructureMetaData {
            return VanillaStructureMetaData {};
        }
    }

    pub enum MetaData {
        Litematica(LitematicaMetaData),
        WE12(WE12MetaData),
        WE13(WE13MetaData),
        VanillaStructure(VanillaStructureMetaData),
    }

    pub struct Schematic {
        pub data_version: i32,

        pub metadata: MetaData,

        pub regions: Vec<Region>,
        pub enclosing_size: [i64; 3],
    }

    impl Schematic {
        pub fn new() -> Schematic {
            return Schematic {
                data_version: 0,
                metadata: Litematica(LitematicaMetaData::new()),
                regions: Vec::new(),
                enclosing_size: [1, 1, 1],
            }
        }
    }
}