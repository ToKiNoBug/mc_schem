use crate::block::Block;
use crate::region::{BlockEntity, PendingTick, WorldSlice};
use crate::schem::{common, Schematic};

pub struct SchemSlice<'a> {
    source: &'a Schematic,
    offset: [i32; 3],
    shape: [i32; 3],
}

impl WorldSlice for SchemSlice<'_> {
    fn offset(&self) -> [i32; 3] {
        return self.offset;
    }

    fn shape(&self) -> [i32; 3] {
        return self.shape;
    }

    fn total_blocks(&self, include_air: bool) -> u64 {
        let mut counter = 0;
        for y in 0..self.shape[1] {
            for z in 0..self.shape[2] {
                for x in 0..self.shape[0] {
                    //let r_pos = [x, y, z];
                    let g_pos = [x + self.offset[0], y + self.offset[1], z + self.offset[2]];
                    let block = self.source.first_block_at(g_pos);
                    if let Some(block) = block {
                        if block.is_structure_void() {
                            continue;
                        }
                        if include_air && block.is_air() {
                            counter += 1;
                            continue;
                        }
                        counter += 1;
                    }
                }
            }
        }
        return counter;
    }

    fn block_info_at(&self, r_pos: [i32; 3]) -> Option<(u16, &Block, Option<&BlockEntity>, Option<&PendingTick>)> {
        let g_pos = [r_pos[0] + self.offset[0], r_pos[1] + self.offset[1], r_pos[2] + self.offset[2]];
        return self.source.first_block_info_at(g_pos);
    }
}

impl Schematic {
    /// Return a slice of a schematic
    pub fn slice(&self, offset: [i32; 3], shape: [i32; 3]) -> Option<SchemSlice> {
        for dim in 0..3 {
            if shape[dim] < 0 {
                panic!("Found negative shape: {}", common::format_size(&shape));
            }
            let min = offset[dim];
            let max = offset[dim] + shape[dim];
            if min < 0 || max >= self.shape()[dim] {
                return None;
            }
        }

        return Some(SchemSlice { source: self, offset, shape });
    }
}