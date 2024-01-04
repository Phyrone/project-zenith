use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use bevy::prelude::Component;
use itertools::Itertools;
use packedvec::PackedVec;
use rayon::prelude::*;

use crate::{CHUNK_SIZE, CHUNK_VOLUME};

fn empty_grid() -> PackedVec<usize> {
    PackedVec::new(vec![0; CHUNK_VOLUME])
}

type BlockArray<Block> = [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component, serde::Serialize, serde::Deserialize)]
//TODO add serde
/// stores a matrix of blocks (thats its orginal intention but you can also store other things)
/// this storage works the best when there are many identical objects and there are more reads than writes
pub struct ChunkStorage<Block: Debug + Clone + Eq + Ord + Send + Sync> {
    //a ordered list of all blocks (including blockstates)
    //TODO maybe replace with smallvec (find a suitable size)
    palette: Vec<Block>,

    //2 bytes are sufficient as the amount of different block-(states) is limited to max 32^3 = 32768
    grid: PackedVec<usize>,
}

impl<Block> ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Default + Send + Sync,
{
    pub fn empty() -> Self {
        Self {
            palette: vec![Block::default()],
            grid: empty_grid(),
        }
    }

    pub fn clear(&mut self) {
        self.palette = vec![Block::default()];
        self.grid = empty_grid()
    }
}

impl<Block> Default for ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Default + Send + Sync,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<Block> From<BlockArray<Block>> for ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Send + Sync,
{
    fn from(array: BlockArray<Block>) -> Self {
        return Self::new(&array);
    }
}

impl<Block> ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Send + Sync,
{
    pub fn new(blocks: &BlockArray<Block>) -> Self {
        let mut palette = Vec::from(blocks);
        palette.par_sort_unstable();
        palette.dedup();
        palette.shrink_to_fit();
        let grid = blocks
            .par_iter()
            .map(|block| palette.binary_search(block).unwrap())
            .collect::<Vec<usize>>();
        let packed_grid = PackedVec::new(grid);
        Self {
            palette,
            grid: packed_grid,
        }
    }

    pub fn contains(&self, block: &Block) -> bool {
        self.palette.binary_search(block).is_ok()
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> &Block {
        if x >= CHUNK_SIZE as u32 || y >= CHUNK_SIZE as u32 || z >= CHUNK_SIZE as u32 {
            panic!("index out of bounds");
        }
        let index = x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE;
        //bounds where already checked above
        let block = unsafe { self.grid.get_unchecked(index) };
        self.palette.get(block).unwrap()
    }

    ///returns an array of all block-(states) in the chunk
    /// the array is ordered
    pub fn blocks(&self) -> &[Block] {
        &self.palette
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, block: Block) {
        if x >= CHUNK_SIZE as u32 || y >= CHUNK_SIZE as u32 || z >= CHUNK_SIZE as u32 {
            panic!("index out of bounds");
        }
        let index = x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE;

        let former_palette_id = unsafe { self.grid.get_unchecked(index) };
        let mut unpacked = self.grid.iter().collect::<Vec<_>>();
        let palette_id = self.get_or_create_pallete_id(block, &mut unpacked);
        unpacked[index] = palette_id;

        self.remove_if_unused(former_palette_id, &mut unpacked);
        self.grid = PackedVec::new(unpacked);
    }

    pub fn set_many(
        &mut self,
        range_x: std::ops::Range<u32>,
        range_y: std::ops::Range<u32>,
        range_z: std::ops::Range<u32>,
        block: Block,
    ) {
        if range_x.end > CHUNK_SIZE as u32
            || range_y.end > CHUNK_SIZE as u32
            || range_z.end > CHUNK_SIZE as u32
        {
            panic!("index out of bounds");
        }

        let mut unpacked = self.grid.iter().collect::<Vec<_>>();
        let palette_id = self.get_or_create_pallete_id(block, &mut unpacked);

        //create cartesian product of all indices
        let all_ranges = range_x.cartesian_product(range_y.cartesian_product(range_z));

        //TODO try parallelizing
        for (x, (y, z)) in all_ranges {
            let former_palette_id = unpacked
                [x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE];
            unpacked[x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE] =
                palette_id;
            self.remove_if_unused(former_palette_id, &mut unpacked);
        }

        self.grid = PackedVec::new(unpacked);
    }

    fn get_or_create_pallete_id(&mut self, block: Block, unpacked: &mut Vec<usize>) -> usize {
        self.palette.binary_search(&block).unwrap_or_else(|index| {
            self.palette.insert(index, block);
            Self::pallete_id_inserted(index, unpacked);
            index
        })
    }

    /*
    fn remove_many_if_unused(&mut self, palette_ids: &[usize], unpacked: &mut Vec<usize>) {
        let mut unused_ids = palette_ids.par_iter()
            .filter(|palette_id| unpacked.par_iter().any(|block| *block == **palette_id).not())
            .copied()
            .collect::<Vec<_>>();
        unused_ids.par_sort_unstable();
        let range = unused_ids[0]..unused_ids[unused_ids.len() - 1];


        unpacked.par_iter_mut()
            .for_each(|block| {
                if range.contains(block) {

                    let decrement_by = unused_ids
                        .iter()
                        .copied()
                        .skip_while(|unused_id| unused_id < *block)
                        .count();
                    *block -= decrement_by;
                }
            });


        for palette_id in unused_ids {
            self.palette.remove(palette_id);
        }
    }
     */

    ///looks if the any block in the grid point to the index and if not removes it from the palette
    fn remove_if_unused(&mut self, palette_id: usize, unpacked: &mut Vec<usize>) {
        let can_be_removed = unpacked.par_iter().any(|block| *block != palette_id).not();
        if can_be_removed {
            self.palette.remove(palette_id);
            unpacked.par_iter_mut().for_each(|block| {
                if *block > palette_id {
                    *block -= 1;
                }
            });
        }
    }

    fn pallete_id_inserted(palette_id: usize, unpacked: &mut Vec<usize>) {
        //increment all indices that are greater or eq than index by 1 to make them not point to the wrong block
        unpacked.par_iter_mut().for_each(|block| {
            if *block >= palette_id {
                *block += 1;
            }
        });
    }

    pub fn iter(&self) -> impl Iterator<Item = &Block> + '_ {
        self.grid
            .iter()
            .map(|palette_id| self.palette.get(palette_id).unwrap())
    }

    ///returns the estimated memory usage in bytes of the chunk including overhead
    /// when [Block] contains pointers/references only the size of the pointers/references will taken into account
    pub fn memory_usage(&self) -> usize {
        let struct_size = std::mem::size_of::<Self>();
        let palette_size = self.palette.capacity() * std::mem::size_of::<Block>();
        let grid_size = (self.grid.len() * self.grid.bwidth()) / 8;
        struct_size + palette_size + grid_size
    }

    pub fn export(&self) -> Box<BlockArray<Block>> {
        let vec = self.par_iter().cloned().collect::<Vec<_>>();

        let attempt: [Block; CHUNK_VOLUME] = vec.try_into().unwrap();
        Box::new(attempt)
    }

    pub fn par_iter(&self) -> impl ParallelIterator<Item = &Block> + '_ {
        self.grid
            .iter()
            .par_bridge()
            .map(|palette_id| self.palette.get(palette_id).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::compressible::Compressible;
    use crate::humanize::humanize_memory;

    use super::*;

    #[derive(
        Debug,
        Default,
        Clone,
        Copy,
        Eq,
        PartialEq,
        Ord,
        PartialOrd,
        Hash,
        serde::Serialize,
        serde::Deserialize,
    )]
    struct TestBlock(usize);

    type TestChunkStorage = ChunkStorage<TestBlock>;

    fn create_sample_blocks(block_count: usize) -> Box<[TestBlock; CHUNK_VOLUME]> {
        let mut input_data = Box::new([TestBlock::default(); CHUNK_VOLUME]);

        //create some random data with about 200 different block ids
        for i in 0..CHUNK_VOLUME {
            let id = i % block_count;
            input_data[i] = TestBlock(id);
        }
        input_data
    }

    #[test]
    fn test_chunk_creation() {
        const BLOCK_COUNT: usize = CHUNK_VOLUME;
        let input_data = create_sample_blocks(BLOCK_COUNT);
        let chunk_storage = TestChunkStorage::new(&input_data);
        println!(
            "memory usage: {}",
            humanize_memory(chunk_storage.memory_usage())
        );
        println!(
            "memory usage (lz4) {}",
            humanize_memory(chunk_storage.compress_lz4().memory_usage())
        );
        println!(
            "memory usage (snappy) {}",
            humanize_memory(chunk_storage.compress_snappy().memory_usage())
        );
        println!(
            "memory usage (zstd) {}",
            humanize_memory(chunk_storage.compress_zstd().memory_usage())
        );
        println!(
            "memory usage (zstd - best) {}",
            humanize_memory(chunk_storage.compress_zstd_best().memory_usage())
        );

        assert_eq!(chunk_storage.blocks().len(), BLOCK_COUNT);
    }
}
