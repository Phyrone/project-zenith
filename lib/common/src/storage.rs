use bevy::prelude::Component;
use std::fmt::Debug;
use std::hash::Hash;
use std::io::{Read, Write};
use std::ops::{Not, Range};

use huffman_coding::HuffmanWriter;
use itertools::Itertools;
use packedvec::PackedVec;
use rayon::prelude::*;

use crate::lzw::lzw_decompress;

pub struct StorageCompressed;

pub struct StorageUncompressed;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Component)]
pub struct Storage<const SIZE: usize, ITEM: Debug + Clone + Eq + Ord + Send + Hash + Sync> {
    palette: Vec<ITEM>,
    data: PackedVec<usize>,
}

impl<const SIZE: usize, ITEM> Storage<SIZE, ITEM>
where
    ITEM: Debug + Clone + Ord + Eq + Hash + Default + Send + Sync,
{
    fn empty_grid() -> PackedVec<usize> {
        PackedVec::new(vec![0; SIZE])
    }

    /// creates a storage with [SIZE] items.
    /// it will use the [Default] value of [ITEM]
    /// Its storage usage should be minimal.
    pub fn empty() -> Self {
        Self {
            palette: vec![ITEM::default()],
            data: Self::empty_grid(),
        }
    }

    pub fn clear(&mut self) {
        self.palette = vec![ITEM::default()];
        self.data = Self::empty_grid()
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

impl<const SIZE: usize, ITEM> Storage<SIZE, ITEM>
where
    ITEM: Debug + Clone + Ord + Eq + Hash + Send + Sync,
{
    /// creates a storage from an array of items.
    /// the items will be cloned, sorted (using ord) and then deduplicated (using eq).
    /// [blocks] must have a length of [SIZE]
    pub fn new(blocks: &[ITEM]) -> Self {
        if blocks.len() != SIZE {
            panic!(
                "invalid array size (must be {} but is {})",
                SIZE,
                blocks.len()
            );
        }

        let mut palette = Vec::<ITEM>::from(blocks);
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
            data: packed_grid,
        }
    }

    pub fn contains(&self, block: &ITEM) -> bool {
        self.palette.binary_search(block).is_ok()
    }

    pub fn get(&self, i: usize) -> &ITEM {
        let item = self.data.get(i);
        if let Some(item_index) = item {
            return self.palette.get(item_index).unwrap();
        } else {
            panic!(
                "storage index out of bounds (index: {} of {})",
                i,
                self.data.len()
            );
        }
    }

    pub fn set(&mut self, i: usize, block: ITEM) {
        if i >= SIZE {
            panic!("index out of bounds");
        }
        let former_palette_id = unsafe { self.data.get_unchecked(i) };
        let mut unpacked = self.data.iter().collect::<Vec<_>>();
        let palette_id = self.get_or_create_pallete_id(block, &mut unpacked);
        unpacked[i] = palette_id;

        self.remove_if_unused(former_palette_id, &mut unpacked);
        self.data = PackedVec::new(unpacked);
    }

    //TODO set many takes to long for large ranges optimize it
    pub fn set_many(&mut self, range: Range<usize>, block: ITEM) {
        if range.end > SIZE || range.start > SIZE {
            panic!("index out of bounds");
        }
        let mut unpacked = self.data.iter().collect::<Vec<_>>();
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

        self.data = PackedVec::new(unpacked);
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

    fn create_gaps(_gap_ids: &[usize], grid: &mut [usize]) {
        grid.par_iter_mut().for_each(|_block| {});
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ ITEM> + '_ {
        self.data
            .iter()
            .map(|palette_id| unsafe { self.palette.get_unchecked(palette_id) })
    }

    ///returns the estimated memory usage in bytes of the cubes including overhead
    /// when [ITEM] contains pointers/references only the size of the pointers/references will taken into account
    pub fn memory_usage(&self) -> usize {
        let struct_size = std::mem::size_of::<Self>();
        let palette_size = self.palette.capacity() * std::mem::size_of::<ITEM>();
        let grid_size = (self.data.len() * self.data.bwidth()) / 8;
        struct_size + palette_size + grid_size
    }

    pub fn export(&self) -> Vec<ITEM> {
        self.iter().cloned().collect::<Vec<_>>()
    }

    pub fn palette(&self) -> &[ITEM] {
        &self.palette
    }

    pub fn data(&self) -> &PackedVec<usize> {
        &self.data
    }
    pub fn export_compressed_data(&self) -> Vec<u8> {
        if self.data.bwidth() == 0 {
            return Vec::new();
        }

        let compressed = lzw_decompress(self.palette.len(), self.data.iter(), None);
        let bytes = compressed
            .par_iter()
            .map(|x| x.to_be_bytes())
            .flatten()
            .collect::<Vec<_>>();
        let tree = huffman_coding::HuffmanTree::from_data(&bytes);
        let mut output = Vec::with_capacity(bytes.len() + 256);
        output.extend_from_slice(&tree.to_table());
        let mut writer = HuffmanWriter::new(&mut output, &tree);
        writer
            .write_all(&bytes)
            .expect("writing data to the vec should not fail");
        drop(writer);
        output.shrink_to_fit();
        output
    }
    pub fn import_from_compressed_data(palette: Vec<ITEM>, data: &[u8]) -> Self {
        if palette.is_empty() {
            panic!("palette must not be empty");
        }
        if data.is_empty() {
            let data = vec![0_usize; SIZE];
            return Self {
                palette,
                data: PackedVec::new(data),
            };
        }
        if data.len() < 256 {
            //TODO response with error instead of panic
            panic!("data must be at least 256 bytes long");
        }

        let tree = huffman_coding::HuffmanTree::from_table(&data[0..256]);
        let data = &data[256..];
        let mut reader = huffman_coding::HuffmanReader::new(data, tree);
        let mut output = Vec::new();
        reader
            .read_to_end(&mut output)
            .expect("reading from the vec should not fail");
        drop(reader);
        let data = output
            .chunks_exact(4)
            .map(|x| usize::from_be_bytes(x.try_into().unwrap()))
            .collect::<Vec<_>>();
        let decompressed = lzw_decompress(palette.len(), data.into_iter(), Some(SIZE));
        assert_eq!(
            decompressed.len(),
            SIZE,
            "data must have the same length as the limit"
        );
        //TODO return error instead of panic
        if decompressed.par_iter().all(|x| *x < palette.len()) {
            panic!("data must only contain valid palette pointers");
        }

        Self {
            palette,
            data: PackedVec::new(decompressed),
        }
    }
}

#[cfg(test)]
mod test {
    use packedvec::PackedVec;
    use rand::Rng;
    use rayon::prelude::*;

    use crate::humanize::humanize_memory;
    use crate::lzw::packed_lzw_compress;

    #[test]
    fn test_export() {
        const materials: usize = 128;
        const size: usize = 32;
        let mut numbers = vec![0; size * size * size];
        numbers
            .par_iter_mut()
            .take(size * size * 4 - 1)
            .for_each(|x| {
                *x = 13;
            });
        numbers
            .par_iter_mut()
            .skip(size * size * 4)
            .take(size * size * 5 - 1)
            .for_each(|x| {
                *x = 7;
            });
        numbers
            .par_iter_mut()
            .skip(size * size * 5)
            .take(size * size * 2 - 1)
            .for_each(|x| {
                let mut rng = rand::thread_rng();
                let random = rng.gen_range(0..materials);
                *x = random;
            });

        let numbers = PackedVec::new(numbers);
        println!(
            "original: {} * {} = {}",
            numbers.len(),
            numbers.bwidth(),
            humanize_memory((numbers.len() * numbers.bwidth()) / 8)
        );
        let compressed = packed_lzw_compress(materials, &numbers);
        println!(
            "compressed: {} * {} = {}",
            compressed.len(),
            compressed.bwidth(),
            humanize_memory((compressed.len() * compressed.bwidth()) / 8)
        );
        let data = bincode::serialize(&compressed).unwrap();
        println!("data: {}", humanize_memory(data.len()));
        println!("  {}", hex::encode(&data));
    }
}
