use packedvec::PackedVec;

pub fn packed_lzw_compress(
    //the amount of worlds already used in the dictionary
    dictionary_size: usize,
    data: &PackedVec<usize>,
) -> PackedVec<usize> {
    if data.is_empty() {
        return PackedVec::new(Vec::new());
    }

    let mut dictionary = Vec::with_capacity(dictionary_size);
    for i in 0..dictionary_size {
        dictionary.push(vec![i]);
    }

    let mut output = Vec::with_capacity(data.len());
    let mut current = Vec::new();
    let mut current_code = 0;

    for element in data.iter() {
        assert!(element < dictionary_size, "element {} is out of bounds", element);
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
        let code = dictionary.iter().position(|x| current.eq(x))
            .expect("last current should have been in the dictionary");
        output.push(code);
    }

    let compressed = PackedVec::new(output);
    let old_size = data.bwidth() * data.len();
    let new_size = compressed.bwidth() * compressed.len();

    //when the compressed data is not smaller than the original data, we return the original data
    if new_size >= old_size || data.len() >= compressed.len() {
        data.to_owned()
    } else {
        compressed
    }
}


pub trait IterLzwExt<T> {
    fn lzw_compress(&self, dictionary_size: usize, force: bool) -> impl Iterator<Item=T>;
    fn lzw_decompress(&self, dictionary_size: usize) -> impl Iterator<Item=T>;
}

struct LzwCompressIter<I> where I: Iterator {
    iter: I,
    current_word: Option<Vec<I::Item>>,
    current_code: usize,
    dictionary: Vec<Vec<I::Item>>,
}

impl<I, E> LzwCompressIter<I> where I: Iterator<Item=E>, E: Eq + Clone {
    fn handle_next(&mut self, next: E) -> Option<usize> {
        //TODO replace with references to avoid cloning?
        //TODO maybe use content hash?
        match &mut self.current_word {
            None => self.current_word = Some(vec![next.clone()]),
            Some(current_word)=> {current_word.push(next.clone())}
        }
        
        let code = self.find_entry();
        let result:Option<usize>;
        match code {
            None => {
                //move the current word to the dictionary
                let current = self.current_word.take()
                    .expect("current word should be present");
                self.current_word = Some(vec![next]);
                result = Some(self.current_code);
                self.current_code = self.find_entry()
                    .expect("element is not in the base dictionary");
                self.dictionary.push(current);
                
            }
            Some(code) => {
                self.current_code = code;
                result = None;
            }
        }
        result
    }
    fn handle_completion(&mut self) -> Option<usize> {
        if let None = self.current_word {
            None
        } else {
            let code = self.find_entry()
                .expect("last current should have been in the dictionary");
            self.current_word = None;
            Some(code)
        }
    }

    fn find_entry(&self) -> Option<usize> {
        match &self.current_word {
            None => None,
            Some(current_word) => self.dictionary.iter().position(|x| current_word.eq(x))
        }
    }
}

impl<I, E> Iterator for LzwCompressIter<I> where I: Iterator<Item=E>, E: Eq + Clone {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        loop {
            let next_item = self.iter.next();
            match next_item {
                None => {
                    return self.handle_completion();
                }
                Some(current) => {
                    let code = self.handle_next(current);
                    if let Some(code) = code {
                        return Some(code);
                    }
                }
            }
        }
    }
}


pub fn packed_lzw_decompress(
    dictionary_size: usize,
    compressed: &PackedVec<usize>,
) -> PackedVec<usize> {
    if compressed.is_empty() {
        return PackedVec::new(Vec::new());
    }

    let first = unsafe { compressed.get_unchecked(0) };
    let mut output = vec![first];
    let mut dictionary = Vec::with_capacity(dictionary_size);
    for i in 0..dictionary_size {
        dictionary.push(vec![i]);
    }
    let mut last_entry = vec![first];

    for element in compressed.iter().skip(1) {
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
        output.extend_from_slice(&next);
        last_entry = next;
    }
    PackedVec::new(output)
}


#[cfg(test)]
mod test {
    use packedvec::PackedVec;

    use crate::lzw::{packed_lzw_compress, packed_lzw_decompress};

    #[test]
    fn test_compression() {
        let max = 32;
        let mut data = vec![];
        data.extend((0..32 * 32 * 32).map(|x| x % max));

        let packed = PackedVec::new(data);
        println!("{} * {} = {}", packed.len(), packed.bwidth(), packed.len() * packed.bwidth());
        let compressed = packed_lzw_compress(max, &packed);
        println!("{} * {} = {}", compressed.len(), compressed.bwidth(), compressed.len() * compressed.bwidth());
        let decompressed = packed_lzw_decompress(max, &compressed);
        println!("{} * {} = {}", decompressed.len(), decompressed.bwidth(), decompressed.len() * decompressed.bwidth());
        let decompressed2 = packed_lzw_decompress(max, &packed);
        println!("{} * {} = {}", decompressed2.len(), decompressed2.bwidth(), decompressed2.len() * decompressed2.bwidth());

        println!("compression ratio: {}", (compressed.len() * compressed.bwidth()) as f64 / (packed.len() * packed.bwidth()) as f64);

        assert_eq!(packed, decompressed);
        assert_eq!(packed, decompressed2);
    }

    #[test]
    fn test_compression2() {
        let max = 32;
        let mut data = vec![];
        data.extend((0..32 * 32 * 32).map(|x| x % max));

        let packed = PackedVec::new(data);
        println!("o {} * {} = {}", packed.len(), packed.bwidth(), packed.len() * packed.bwidth());
        let compressed = packed_lzw_compress(max, &packed);
        println!("c {} * {} = {}", compressed.len(), compressed.bwidth(), compressed.len() * compressed.bwidth());
        let decompressed = packed_lzw_decompress(max, &compressed);
        println!("d {} * {} = {}", decompressed.len(), decompressed.bwidth(), decompressed.len() * decompressed.bwidth());

        assert_eq!(packed, decompressed);


        let compressed_compressed = packed_lzw_compress(compressed.iter().max().unwrap() + 1, &compressed);
        println!("cx2 {} * {} = {}", compressed_compressed.len(), compressed_compressed.bwidth(), compressed_compressed.len() * compressed_compressed.bwidth());

        let lz4_compressed = lz4_flex::compress_prepend_size(&bincode::serialize(&packed).unwrap());
        println!("l4 {} * {} = {}", lz4_compressed.len(), 1, lz4_compressed.len());
        let lz4_compressed_compressed = lz4_flex::compress_prepend_size(&bincode::serialize(&lz4_compressed).unwrap());
        println!("c + l4 {} * {} = {}", lz4_compressed_compressed.len(), 1, lz4_compressed_compressed.len());

        let lz4_double_compressed = lz4_flex::compress_prepend_size(&bincode::serialize(&lz4_compressed).unwrap());
        println!("l4x2 {} * {} = {}", lz4_double_compressed.len(), 1, lz4_double_compressed.len());
    }
}

