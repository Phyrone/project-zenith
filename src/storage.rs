use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::{Not, Range};

use bevy::prelude::Component;
use itertools::Itertools;
use packedvec::PackedVec;
use rayon::prelude::*;

use crate::CHUNK_SIZE;

type BlockArray<Block> = [Block; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component, serde::Serialize, serde::Deserialize)]
//TODO add serde
/// stores a matrix of blocks (thats its orginal intention but you can also store other things)
/// this storage works the best when there are many identical objects and there are more reads than writes
pub struct Storage<const SIZE: usize, ITEM: Debug + Clone + Eq + Ord + Send + Sync> {
    //a ordered list of all blocks (including blockstates)
    //TODO maybe replace with smallvec (find a suitable size)
    palette: Vec<ITEM>,

    //2 bytes are sufficient as the amount of different block-(states) is limited to max 32^3 = 32768
    grid: PackedVec<usize>,
}

impl<const SIZE: usize, ITEM> Storage<SIZE, ITEM>
where
    ITEM: Debug + Clone + Ord + Eq + Hash + Default + Send + Sync,
{
    fn empty_grid() -> PackedVec<usize> {
        PackedVec::new(vec![0; SIZE])
    }

    pub fn empty() -> Self {
        Self {
            palette: vec![ITEM::default()],
            grid: Self::empty_grid(),
        }
    }

    pub fn clear(&mut self) {
        self.palette = vec![ITEM::default()];
        self.grid = Self::empty_grid()
    }
}

impl<const SIZE: usize, ITEM> Default for Storage<SIZE, ITEM>
where
    ITEM: Debug + Clone + Ord + Eq + Hash + Default + Send + Sync,
{
    fn default() -> Self {
        Self::empty()
    }
}

impl<const LIMIT: usize, ITEM> Storage<LIMIT, ITEM>
where
    ITEM: Debug + Clone + Ord + Eq + Hash + Send + Sync,
{
    pub fn new(blocks: &Vec<ITEM>) -> Self {
        if blocks.len() != LIMIT {
            panic!("invalid block array size (must be {})", LIMIT);
        }

        let mut palette = blocks.clone();
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

    pub fn contains(&self, block: &ITEM) -> bool {
        self.palette.binary_search(block).is_ok()
    }

    pub fn get(&self, i: usize) -> &ITEM {
        let item = self.grid.get(i);
        if let Some(item) = item {
            return self.palette.get(item).unwrap();
        } else {
            panic!("index out of bounds");
        }
    }

    ///returns an array of all block-(states) in the chunk
    /// the array is ordered
    pub fn blocks(&self) -> &[ITEM] {
        &self.palette
    }

    pub fn set(&mut self, i: usize, block: ITEM) {
        if i >= LIMIT {
            panic!("index out of bounds");
        }

        let former_palette_id = unsafe { self.grid.get_unchecked(i) };
        let mut unpacked = self.grid.iter().collect::<Vec<_>>();
        let palette_id = self.get_or_create_pallete_id(block, &mut unpacked);
        unpacked[i] = palette_id;

        self.remove_if_unused(former_palette_id, &mut unpacked);
        self.grid = PackedVec::new(unpacked);
    }

    //TODO set many takes to long for large ranges optimize it
    pub fn set_many(&mut self, range: Range<usize>, block: ITEM) {
        if range.end > LIMIT || range.start > LIMIT {
            panic!("index out of bounds");
        }

        let mut unpacked = self.grid.iter().collect::<Vec<_>>();
        let palette_id = self.get_or_create_pallete_id(block, &mut unpacked);

        let maybe_unused = unpacked
            .par_iter_mut()
            .skip(range.start)
            .take(range.end - range.start)
            .map(|block| {
                let old = *block;
                *block = palette_id;
                old
            })
            .collect::<Vec<_>>();

        self.remove_many_if_unused(&maybe_unused, &mut unpacked);

        self.grid = PackedVec::new(unpacked);
    }

    fn get_or_create_pallete_id(&mut self, block: ITEM, unpacked: &mut Vec<usize>) -> usize {
        self.palette.binary_search(&block).unwrap_or_else(|index| {
            self.palette.insert(index, block);
            Self::pallete_id_inserted(index, unpacked);
            index
        })
    }

    pub fn shrink_to_fit(&mut self) {
        self.palette.shrink_to_fit();
    }

    fn remove_many_if_unused(&mut self, maybe_unused: &[usize], unpacked: &mut Vec<usize>) {
        //TODO replace with batch and parallel removal
        for (index, palette_id) in maybe_unused.iter().sorted_unstable().dedup().enumerate() {
            self.remove_if_unused(*palette_id - index, unpacked);
        }
    }

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

    pub fn iter(&self) -> impl Iterator<Item = &ITEM> + '_ {
        self.grid
            .iter()
            .map(|palette_id| unsafe { self.palette.get_unchecked(palette_id) })
    }

    ///returns the estimated memory usage in bytes of the chunk including overhead
    /// when [ITEM] contains pointers/references only the size of the pointers/references will taken into account
    pub fn memory_usage(&self) -> usize {
        let struct_size = std::mem::size_of::<Self>();
        let palette_size = self.palette.capacity() * std::mem::size_of::<ITEM>();
        let grid_size = (self.grid.len() * self.grid.bwidth()) / 8;
        struct_size + palette_size + grid_size
    }

    pub fn export(&self) -> Vec<ITEM> {
        self.iter().cloned().collect::<Vec<_>>()
    }
}
