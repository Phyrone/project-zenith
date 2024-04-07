use std::io::{Read, Write};

use huffman_coding::{HuffmanReader, HuffmanTree, HuffmanWriter};

pub fn to_huffman(fields: &[usize]) -> Vec<u8> {
    let bytes = as_bytes(fields);
    let tree = HuffmanTree::from_data(bytes);

    let mut output = Vec::new();
    output
        .write_all(&tree.to_table())
        .expect("writing the tree to the vec should not fail");

    let mut writer = HuffmanWriter::new(&mut output, &tree);
    writer
        .write_all(bytes)
        .expect("writing data to the vec shoudl not fail");
    writer.flush().expect("flushing the writer should not fail");
    //for some reason rust had problems to drop the writer automatically
    drop(writer);
    output
}

fn from_huffman(data: &[u8]) -> Vec<usize> {
    let tree = HuffmanTree::from_table(&data[0..256]);
    let data = &data[256..];
    let mut reader = HuffmanReader::new(data, tree);
    let mut output = Vec::new();
    reader
        .read_to_end(&mut output)
        .expect("reading from the vec should not fail");
    if output.len() % std::mem::size_of::<usize>() != 0 {
        panic!("the length of the output should be a multiple of the size of usize");
    }
    unsafe { unsafe_vec_transform::<u8, usize>(output) }
}

/// represents the usize as a byte array without copying the data
/// the length of the bytes array is fields.len() * std::mem::size_of::<usize>()
/// !! Keep in mind that the endianness of bytes varies between systems. !!
pub fn as_bytes(fields: &[usize]) -> &[u8] {
    let bytes = fields;
    //this is unsafe because we represent the data as different on wich it orginally not is
    //since we only use the raw bytes here, this is safe as long as the length is correct
    let bytes = unsafe {
        std::slice::from_raw_parts(
            bytes.as_ptr() as *const u8,
            std::mem::size_of_val(bytes),
        )
    };
    bytes
}

/// # Safety
/// This function is unsafe because we force the data to be interpreted as a different type
///  - Do not use it for types with a size of 0  
///  - Do not use it for types with pointers (e.g. Box, Rc, Arc, etc.)
///     the information will be lost and the memory will not be freed (a.k.a. memory leak)
///
pub unsafe fn unsafe_vec_transform<From, To>(from: Vec<From>) -> Vec<To> {
    if std::mem::size_of::<From>() == 0 || std::mem::size_of::<To>() == 0 {
        if std::mem::size_of::<From>() == 0 {
            panic!("the size of the input type should not be 0");
        } else {
            panic!("the size of the output type should not be 0");
        }
    }
    if from.len() * std::mem::size_of::<From>() % std::mem::size_of::<To>() != 0 {
        panic!("the new typt does not appear to fit into the allocated memory")
    }
    let length = from.len() * std::mem::size_of::<From>() / std::mem::size_of::<To>();
    let capacity = from.capacity() * std::mem::size_of::<From>() / std::mem::size_of::<To>();
    let capacity = if from.capacity() * std::mem::size_of::<From>() % std::mem::size_of::<To>() != 0
    {
        //if the capacity is not a multiple of the size of To, we have to decrease the capacity by 1
        //this makes some allocated memory unused, but is required to not cause a buffer overflow
        capacity - 1
    } else {
        capacity
    };

    let ptr = from.as_ptr() as *mut To;
    std::mem::forget(from);
    Vec::from_raw_parts(ptr, length, capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let fields = [1, 2, 3, 4];
        let bytes = as_bytes(&fields);
        assert_eq!(bytes.len(), 4 * std::mem::size_of::<usize>());
    }

    #[test]
    fn test_huffman() {
        let fields = vec![0; (32 * 32 * 32) * 64];

        let huffman = to_huffman(&fields);
        let ratio = huffman.len() as f64 / (fields.len() * std::mem::size_of::<usize>()) as f64;
        let fields2 = from_huffman(&huffman);
        println!("compression ratio: {}", ratio);
        assert_eq!(fields, fields2);
    }
}
