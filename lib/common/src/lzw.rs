use packedvec::PackedVec;

pub fn lzw_compress_raw<I>(
    //the amount of worlds already used in the dictionary
    dictionary_size: usize,
    data: I,
) -> Vec<usize>
where
    I: Iterator<Item = usize>,
{
    let mut dictionary = Vec::with_capacity(dictionary_size);
    for i in 0..dictionary_size {
        dictionary.push(vec![i]);
    }

    let mut output = Vec::new();
    let mut current = Vec::new();
    let mut current_code = 0;

    for element in data {
        assert!(
            element < dictionary_size,
            "element {} is out of bounds",
            element
        );
        current.push(element);
        let code = dictionary.iter().position(|x| current.eq(x));
        match code {
            None => {
                //println!("{:?} | {} | {:?} | {}", &current[0..(current.len() - 1)], element, current, current_code);
                dictionary.push(current);
                output.push(current_code);
                current = vec![element];
                current_code = element;
            }
            Some(code) => {
                //println!("{:?} | {} | - | - |", &current[0..(current.len() - 1)], element, );
                current_code = code;
            }
        }
    }
    if !current.is_empty() {
        let code = dictionary
            .iter()
            .position(|x| current.eq(x))
            .expect("last current should have been in the dictionary");
        output.push(code);
    }
    output
}

pub fn packed_lzw_compress(
    //the amount of worlds already used in the dictionary
    dictionary_size: usize,
    data: &PackedVec<usize>,
) -> PackedVec<usize> {
    let compressed = lzw_compress_raw(dictionary_size, data.iter());
    let compressed = PackedVec::new(compressed);
    let old_size = data.bwidth() * data.len();
    let new_size = compressed.bwidth() * compressed.len();
    //when the compressed data is not smaller than the original data, we return the original data
    if new_size >= old_size || compressed.len() >= data.len() {
        data.to_owned()
    } else {
        compressed
    }
}

pub fn lzw_decompress<I>(
    dictionary_size: usize,
    mut compressed: I,
    limit: Option<usize>,
) -> Vec<usize>
where
    I: Iterator<Item = usize>,
{
    let first = compressed.next();
    let first = match first {
        None => return Vec::new(),
        Some(first) => first,
    };

    let mut output = vec![first];
    let mut dictionary = Vec::with_capacity(dictionary_size);
    for i in 0..dictionary_size {
        dictionary.push(vec![i]);
    }
    let mut last_entry = vec![first];

    for element in compressed {
        let next = dictionary.get(element).cloned();
        let next: Vec<usize> = match next {
            Some(next) => {
                let mut new_entry = last_entry;
                new_entry.push(next[0]);
                dictionary.push(new_entry);
                next
            }
            None => {
                let mut new_entry = last_entry.clone();
                new_entry.push(last_entry[0]);
                dictionary.push(new_entry.clone());
                new_entry
            }
        };
        //check limit to prevent getting "zip-bombed"
        if let Some(limit) = limit {
            if output.len() + next.len() > limit {
                let amount = limit - output.len();
                if amount > 0 {
                    output.extend_from_slice(&next[0..amount]);
                }
                //TODO return error instead of cutting off
                break;
            }
        }
        output.extend_from_slice(&next);
        last_entry = next;
    }
    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lzw_compress_raw_compression_works() {
        //create a vector with 10 elements all 0 which should be easy to compress
        let data = vec![0; 10];
        let compressed = lzw_compress_raw(10, data.into_iter());
        assert!(compressed.len() < 10);
    }

    #[test]
    fn lzw_compress_raw_compression_with_empty_data() {
        let data: Vec<usize> = Vec::new();
        let compressed = lzw_compress_raw(10, data.into_iter());
        assert_eq!(compressed.len(), 0);
    }

    #[test]
    fn packed_lzw_compress_compression_works() {
        let data = PackedVec::new(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let compressed = packed_lzw_compress(10, &data);
        //since compression does not help here, we should get the original data back
        assert_eq!(data, compressed);
    }

    #[test]
    fn packed_lzw_compress_compression_with_empty_data() {
        let data = PackedVec::new(Vec::new());
        let compressed = packed_lzw_compress(10, &data);
        assert_eq!(compressed.len(), 0);
    }

    #[test]
    fn lzw_decompress_decompression_works() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let compressed = lzw_compress_raw(10, data.clone().into_iter());
        let decompressed = lzw_decompress(10, compressed.into_iter(), None);
        assert_eq!(data, decompressed);
    }

    #[test]
    fn lzw_decompress_decompression_with_empty_data() {
        let compressed: Vec<usize> = Vec::new();
        let decompressed = lzw_decompress(10, compressed.into_iter(), None);
        assert_eq!(decompressed.len(), 0);
    }

    #[test]
    fn lzw_decompress_decompression_with_limit() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let compressed = lzw_compress_raw(10, data.clone().into_iter());
        let decompressed = lzw_decompress(10, compressed.into_iter(), Some(5));
        assert_eq!(decompressed.len(), 5);
    }
}
