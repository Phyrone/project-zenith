#[derive(
    Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct ChunkSegment {
    x: i32,
    y: i32,
    z: i32,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chunk_segment() {
        let chunk_segment = ChunkSegment {
            x: 1,
            y: i32::MAX,
            z: 3,
        };
        let bytes = rkyv::to_bytes::<_, 256>(&chunk_segment).unwrap();
        let bytes = lz4_flex::compress_prepend_size(&bytes);
        println!("{:?}", bytes);
    }
}
