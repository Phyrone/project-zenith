use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Not;

use bevy::prelude::Component;
use bevy::utils::HashMap;
use itertools::Itertools;
use rayon::prelude::*;

use crate::{CHUNK_SIZE, CHUNK_VOLUME};

type BlockArray<Block> = [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component, serde::Serialize, serde::Deserialize)]
//TODO add serde
pub struct ChunkStorage<Block: Debug + Clone + Eq + Ord> {
    //a ordered list of all blocks (including blockstates)
    //TODO maybe replace with smallvec (find a suitable size)
    palette: Vec<Block>,

    #[serde(with = "serde_big_array::BigArray")]
    //2 bytes are sufficient as the amount of different block-(states) is limited to max 32^3 = 32768
    grid: [u16; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

impl<Block> ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Default,
{
    pub fn empty() -> Self {
        Self {
            palette: vec![Block::default()],
            grid: [0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    pub fn clear(&mut self) {
        self.palette = vec![Block::default()];
        self.grid = [0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    }
}

impl<Block> Default for ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash + Default,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<Block> From<BlockArray<Block>> for ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash,
{
    fn from(array: BlockArray<Block>) -> Self {
        return Self::new(&array);
    }
}

impl<Block> ChunkStorage<Block>
where
    Block: Debug + Clone + Ord + Eq + Hash,
{
    fn new(blocks: &BlockArray<Block>) -> Self {
        let mut middle: HashMap<Block, Vec<usize>> = HashMap::default();
        for (pos, block) in blocks.iter().enumerate() {
            let positions = middle.get_mut(block);
            match positions {
                Some(positions) => positions.push(pos),
                None => {
                    middle.insert(block.clone(), vec![pos]);
                }
            }
        }

        let mut palette = Vec::new();
        let mut grid = [0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        for (index, (block, positions)) in middle
            .into_iter()
            .sorted_by_cached_key(|(block, _value)| block.clone())
            .enumerate()
        {
            palette.push(block);
            for pos in positions {
                grid[pos] = index as u16;
            }
        }

        Self { palette, grid }
    }
    pub fn contains(&self, block: &Block) -> bool {
        self.palette.binary_search(block).is_ok()
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> &Block {
        if x >= CHUNK_SIZE as u32 || y >= CHUNK_SIZE as u32 || z >= CHUNK_SIZE as u32 {
            panic!("index out of bounds");
        }

        let index =
            self.grid[x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE];
        self.palette.get(index as usize).unwrap()
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

        let former_index =
            self.grid[x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE];
        let index = self.get_or_create_index(block);
        self.grid[x as usize + y as usize * CHUNK_SIZE + z as usize * CHUNK_SIZE * CHUNK_SIZE] =
            index;
        self.remove_if_unused(former_index);
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

        let index = self.get_or_create_index(block);
        for x in range_x {
            for y in range_y.clone() {
                for z in range_z.clone() {
                    let former_index = self.grid[x as usize
                        + y as usize * CHUNK_SIZE
                        + z as usize * CHUNK_SIZE * CHUNK_SIZE];
                    self.grid[x as usize
                        + y as usize * CHUNK_SIZE
                        + z as usize * CHUNK_SIZE * CHUNK_SIZE] = index;
                    self.remove_if_unused(former_index);
                }
            }
        }
    }

    fn get_or_create_index(&mut self, block: Block) -> u16 {
        self.palette.binary_search(&block).unwrap_or_else(|index| {
            self.palette.insert(index, block);
            self.pallete_index_inserted(index as u16);
            index
        }) as u16
    }

    ///looks if the any block in the grid point to the index and if not removes it from the palette
    fn remove_if_unused(&mut self, index: u16) {
        let can_be_removed = self.grid.par_iter().any(|block| *block != index).not();
        if can_be_removed {
            self.palette.remove(index as usize);
            self.pallete_index_removed(index);
        }
    }

    fn pallete_index_inserted(&mut self, index: u16) {
        //increment all indices that are greater or eq than index by 1 to make them not point to the wrong block
        self.grid.par_iter_mut().for_each(|block| {
            if *block >= index {
                *block += 1;
            }
        });
    }
    fn pallete_index_removed(&mut self, index: u16) {
        //decrement all indices that are greater than index by 1 to make them not point to the wrong block
        self.grid.par_iter_mut().for_each(|block| {
            if *block > index {
                *block -= 1;
            }
        });
    }
}

impl<Block> ChunkStorage<Block>
where
    Block: Debug + Clone + Eq + Ord + Send + Sync,
{
    pub fn unpack(&self) -> Box<[Block; CHUNK_VOLUME]> {
        let slice = self
            .grid
            .par_iter()
            .map(|index| self.palette[*index as usize].clone())
            .collect::<Vec<Block>>();
        let attempt: [Block; CHUNK_VOLUME] = slice.try_into().unwrap();
        Box::new(attempt)
    }
}
