//T = type of compressed data
//A = type of compression algorithm

use bevy::prelude::Component;

#[derive(Debug, Clone, Eq, PartialEq, Component, serde::Serialize, serde::Deserialize)]
pub struct Compressed<T, A> {
    data: Vec<u8>,
    len: usize,
    _type: std::marker::PhantomData<T>,
    _algorithm: std::marker::PhantomData<A>,
}

impl<T, A> Compressed<T, A> {
    pub fn len_data(&self) -> usize {
        self.len
    }
    pub fn len_compressed(&self) -> usize {
        self.data.len()
    }
}

pub struct LZ4;

pub struct ZSTD;

//TODO add error return types
pub trait Compressible<T> {
    fn compress_lz4(&self) -> Compressed<T, LZ4>;

    fn compress_zstd(&self) -> Compressed<T, ZSTD>;

    fn compress_zstd_best(&self) -> Compressed<T, ZSTD> {
        self.compress_zstd_with_level(22)
    }

    fn compress_zstd_with_level(&self, level: i32) -> Compressed<T, ZSTD>;
}

impl<T> Compressible<T> for T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn compress_lz4(&self) -> Compressed<T, LZ4> {
        let bytes = bincode::serialize(self).expect("failed to serialize data");
        let size = bytes.len();
        let mut compressed = lz4_flex::compress(&bytes);
        compressed.shrink_to_fit();

        Compressed {
            data: compressed,
            len: size,
            _type: std::marker::PhantomData,
            _algorithm: std::marker::PhantomData,
        }
    }

    fn compress_zstd(&self) -> Compressed<T, ZSTD> {
        let bytes = bincode::serialize(self).expect("failed to serialize data");
        let size = bytes.len();
        let mut compressed = zstd::bulk::compress(&bytes, 0).unwrap();
        compressed.shrink_to_fit();

        Compressed {
            data: compressed,
            len: size,
            _type: std::marker::PhantomData,
            _algorithm: std::marker::PhantomData,
        }
    }

    fn compress_zstd_with_level(&self, level: i32) -> Compressed<T, ZSTD> {
        if !(0..=22).contains(&level) {
            panic!("level must be between 0 and 22");
        }

        let bytes = bincode::serialize(self).expect("failed to serialize data");
        let size = bytes.len();
        let mut compressed = zstd::bulk::compress(&bytes, level).unwrap();
        compressed.shrink_to_fit();

        Compressed {
            data: compressed,
            len: size,
            _type: std::marker::PhantomData,
            _algorithm: std::marker::PhantomData,
        }
    }
}

impl<T> Compressed<T, LZ4>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn decompress(&self) -> T {
        let bytes = lz4_flex::decompress(&self.data, self.len).unwrap();
        bincode::deserialize(&bytes).unwrap()
    }
}

impl<T> Compressed<T, ZSTD>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn decompress(&self) -> T {
        let bytes = zstd::bulk::decompress(&self.data, self.len).unwrap();
        bincode::deserialize(&bytes).unwrap()
    }
}
