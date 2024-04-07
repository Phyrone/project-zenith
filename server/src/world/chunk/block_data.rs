use bevy::prelude::Component;

use serde::{Deserialize, Serialize};

use game2::CHUNK_SIZE;

use crate::world::block::BlockData;

#[derive(Debug, Clone, Component, Serialize, Deserialize, Hash, Ord, PartialOrd, PartialEq, Eq)]
pub struct ChunkBlockData {
    pub blocks: Box<[[[BlockData; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>,
}

#[cfg(test)]
mod test {
    use super::*;
    use bevy::prelude::Entity;
    use game2::compressible::{Compressed, Compressible, LZ4, ZSTD};
    use rand::Rng;
    use std::mem::size_of;

    #[test]
    fn a() {
        let mut a = ChunkBlockData {
            blocks: Box::new([[[BlockData::default(); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]),
        };

        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                let mut rng = rand::thread_rng();
                let random = rng.gen_range(0..255);
                let material = match random {
                    0 => None,
                    _ => Some(Entity::from_raw(random)),
                };
                for z in 0..CHUNK_SIZE {
                    a.blocks[x][y][z] = BlockData::new(material);
                }
            }
        }

        let aser = bincode::serialize(&a).expect("Failed to serialize chunk block data");
        println!(
            "raw: size: {} + struct: {}",
            aser.len(),
            size_of::<Vec<u8>>()
        );
        let compressed_zstd = a.compress_zstd();
        println!(
            "zstd: size: {} + struct: {}",
            compressed_zstd.len_compressed(),
            size_of::<Compressed<ChunkBlockData, ZSTD>>()
        );
        let compressed_lz4 = a.compress_lz4();
        println!(
            "lz4: size: {} + struct: {}",
            compressed_lz4.len_compressed(),
            size_of::<Compressed<ChunkBlockData, LZ4>>()
        );
    }
}
