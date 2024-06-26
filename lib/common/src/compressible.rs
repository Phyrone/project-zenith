//T = type of compressed data
//A = type of compression algorithm

use bevy::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Component)]
pub struct Compressed<T, A> {
    data: Vec<u8>,
    len: usize,
    _type: std::marker::PhantomData<T>,
    _algorithm: std::marker::PhantomData<A>,
}

impl<T, A> Compressed<T, A>
where
    T: serde::Serialize,
{
    pub fn len_data(&self) -> usize {
        self.len
    }
    pub fn len_compressed(&self) -> usize {
        self.data.len()
    }

    pub fn memory_usage(&self) -> usize {
        let struct_size = std::mem::size_of::<Self>();
        let data_size = self.data.capacity();
        struct_size + data_size
    }
}

pub struct LZ4;


//TODO add error return types
pub trait Compressible<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn compress_lz4(&self) -> Compressed<T, LZ4>;

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


}

impl<T> Compressed<T, LZ4>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    pub fn decompress(&self) -> T {
        let bytes = lz4_flex::decompress(&self.data, self.len).unwrap();
        bincode::deserialize(&bytes).unwrap()
    }
}


#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde::Serialize;

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
    struct TestCompressibleStruct {
        an_int: i32,
        a_string: String,
        a_byte_array: Vec<u8>,
        a_bool: bool,
    }

    fn create_test_struct() -> TestCompressibleStruct {
        TestCompressibleStruct {
            an_int: 123,
            a_string: "hello world".to_string(),
            a_byte_array: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            a_bool: true,
        }
    }

    #[test]
    fn lz4_compression() {
        let test_struct = create_test_struct();
        let compressed = test_struct.compress_lz4();
        let decompressed = compressed.decompress();
        assert_eq!(test_struct, decompressed);
    }

    #[test]
    fn zstd_compression() {
        let test_struct = create_test_struct();
        let compressed = test_struct.compress_zstd();
        let decompressed = compressed.decompress();
        assert_eq!(test_struct, decompressed);
    }

    #[test]
    fn zstd_best_compression() {
        let test_struct = create_test_struct();
        let compressed = test_struct.compress_zstd_best();
        let decompressed = compressed.decompress();
        assert_eq!(test_struct, decompressed);
    }
}
