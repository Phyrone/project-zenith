use std::mem::size_of;

const EMPTY_RAW_FALLBACK: [u8; 0] = [];

/// represents the usize as a byte array without copying the data
/// the length of the bytes array is fields.len() * std::mem::size_of::<usize>()
/// !! Keep in mind that the endianness of bytes varies between systems. !!
pub fn raw_bytes<'a, T>(object: &'a T) -> &'a [u8]
where
    T: Sized,
{
    let size: usize = size_of::<T>();
    if size == 0 {
        return &EMPTY_RAW_FALLBACK;
    }

    //this is unsafe because we represent the data as different on wich it orginally not is
    //since we only use the raw bytes here, this is safe as long as the length is correct
    let bytes = unsafe { std::slice::from_raw_parts((object as *const T) as *const u8, size) };
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

pub fn into_percent(value: f64) -> String {
    format!("{:.2}%", value * 100.0)
}

#[macro_use]
pub mod error{
    #[macro_export]
    macro_rules! error_object {
        ($name:ident,$msg:expr) => {
            #[derive(Debug)]
            pub struct $name;
            impl std::fmt::Display for $name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, $msg)
                }
            }
            impl std::error::Error for $name {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let fields = [1, 2, 3, 4];
        let expected_size = std::mem::size_of_val(&fields);
        let bytes = raw_bytes(&fields);
        let hex = hex::encode(bytes);
        println!("bytes: {}", hex);
        assert_eq!(bytes.len(), expected_size);
    }
}
