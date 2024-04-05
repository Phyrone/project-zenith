use bevy::utils::petgraph::visit::Walker;
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

pub fn lzw_decompress<I>(dictionary_size: usize, mut compressed: I) -> Vec<usize>
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
        output.extend_from_slice(&next);
        last_entry = next;
    }
    output
}

#[cfg(test)]
mod test {
    
}
