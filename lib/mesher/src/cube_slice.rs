use std::ops::BitAnd;

use bitvec::order::Lsb0;
use bitvec::prelude::{BitStore, BitVec};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SlicePartQuad {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub fn greedy_mesh_slice_dyn<T>(plane: &mut [BitVec<T, Lsb0>]) -> Vec<SlicePartQuad>
where
    T: BitStore,
{
    let size = plane.len();
    plane.iter_mut().for_each(|row| row.resize(size, false));

    let mut quads = Vec::new();
    for x in 0..size {
        //as long as the row is not empty (zero) we look for quads
        while plane[x].count_ones() > 0 {
            let mut row = plane[x].clone();
            let y = row.leading_zeros();
            //find the next contiguous occupied quads on the y axis
            if y > 0 {
                row.shift_left(y);
            }
            let h = row.leading_ones();
            let mut mask = BitVec::repeat(false, size);
            mask.splice(y..(y + h), std::iter::repeat(true).take(h));
            //remove claimed part of the row
            plane[x] &= !mask.clone();
            let mut w = 1;
            //try to expand the quad on the x axis
            for i in (x + 1)..size {
                if (plane[i].clone().bitand(&mask)).eq(&mask) {
                    w += 1;
                    //remove claimed part of the row
                    plane[i] &= !mask.clone();
                } else {
                    break;
                }
            }
            //create a quad
            quads.push(SlicePartQuad {
                x,
                y,
                width: w as usize,
                height: h,
            });
        }
    }
    quads
}

macro_rules! impl_mesh_plane {
    ($name:ident,$i:ty,$bits:literal) => {
        pub fn $name(mut slice: [$i; $bits]) -> Vec<SlicePartQuad> {
            let mut quads = Vec::new();

            //occupied cannot be more than 64
            const SIZE: usize = $bits;

            for x in 0..SIZE {
                //as long as the row is not empty (zero) we look for quads
                while slice[x] != 0 {
                    let row = slice[x];
                    let y = row.trailing_zeros();
                    //find the next contiguous occupied quads on the y axis
                    let h = (row >> y).trailing_ones();
                    let mask = ((1 << h) - 1) << y;
                    //remove claimed part of the row
                    slice[x] &= !mask;
                    let mut w = 1;
                    //try to expand the quad on the x axis
                    for i in (x + 1)..SIZE {
                        if (slice[i] & mask) == mask {
                            w += 1;
                            //remove claimed part of the row
                            slice[i] &= !mask;
                        } else {
                            break;
                        }
                    }
                    //create a quad
                    quads.push(SlicePartQuad {
                        x: x as usize,
                        y: y as usize,
                        width: w as usize,
                        height: h as usize,
                    });
                }
            }
            quads
        }
    };
}
impl_mesh_plane!(greedy_mesh_slice_128, u128, 128);
impl_mesh_plane!(greedy_mesh_slice_64, u64, 64);
impl_mesh_plane!(greedy_mesh_slice_32, u32, 32);
impl_mesh_plane!(greedy_mesh_slice_16, u16, 16);
impl_mesh_plane!(greedy_mesh_slice_8, u8, 8);



pub fn greedy_mesh_slice_64_no_alloc(
    mut slice: [u64; 64],
    mut cb: impl FnMut(usize, usize, usize, usize),
) {
    
    //occupied cannot be more than 64
    const SIZE: usize = 64;

    for x in 0..SIZE {
        //as long as the row is not empty (zero) we look for quads
        while slice[x] != 0 {
            let row = slice[x];
            let y = row.trailing_zeros();

            //find the next contiguous occupied quads on the y axis
            let h = (row >> y).trailing_ones();
            //FIXME: overflow
            let mask = if h >= u64::BITS {
                u64::MAX
            } else {
                ((1 << h) - 1) << y
            };
            //remove claimed part of the row
            slice[x] &= !mask;
            let mut w = 1;
            //try to expand the quad on the x axis
            for i in (x + 1)..SIZE {
                if (slice[i] & mask) == mask {
                    w += 1;
                    //remove claimed part of the row
                    slice[i] &= !mask;
                } else {
                    break;
                }
            }
            //create a quad
            cb(x, y as usize, w as usize, h as usize);
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use rstest::rstest;

    use super::*;

    #[rstest]
    #[timeout(Duration::from_secs(3))]
    fn test_greedy_mesh_plane_64() {
        println!("greedy_mesh_slice_64");
        let data = [
            0b1100_0011,
            0b1100_1111,
            0b0011_1111,
            0b0001_0000,
            0b1000_0000,
            0b1000_1000,
            0b1000_0100,
            0b1000_0010,
        ];
        let mut plane = vec![0; 64];
        plane.splice(0..8, data.into_iter());
        let plane: [u64; 64] = plane.try_into().unwrap();
        let quads = greedy_mesh_slice_64(plane);
        for quad in quads.iter() {
            println!("{:?}", quad);
        }
        //visualize the quads
        let mut grid = [['.'; 8]; 8];
        for quad in quads {
            for y in quad.y..(quad.y + quad.height) {
                for x in quad.x..(quad.x + quad.width) {
                    grid[7 - x as usize][7 - y as usize] = '#';
                }
            }
        }
        grid.reverse();
        for row in grid.iter() {
            println!("{:?}", row);
        }

        println!("greedy_mesh_slice_32");
        let data = [
            0b1100_0011,
            0b1100_1111,
            0b0011_1111,
            0b0001_0000,
            0b1000_0000,
            0b1000_1000,
            0b1000_0100,
            0b1000_0010,
        ];
        let mut plane = vec![0; 32];
        plane.splice(0..8, data.into_iter());
        let plane: [u32; 32] = plane.try_into().unwrap();
        let occupied = 8;
        let quads = greedy_mesh_slice_32(plane);
        for quad in quads.iter() {
            println!("{:?}", quad);
        }
        //visualize the quads
        let mut grid = [['.'; 8]; 8];
        for quad in quads {
            for y in quad.y..(quad.y + quad.height) {
                for x in quad.x..(quad.x + quad.width) {
                    grid[7 - x as usize][7 - y as usize] = '#';
                }
            }
        }
        grid.reverse();
        for row in grid.into_iter() {
            println!("{:?}", row);
        }

        println!("greedy_mesh_slice dyn");
        let data: [u32; 8] = [
            0b1100_0011,
            0b1100_1111,
            0b0011_1111,
            0b0001_0000,
            0b1000_0000,
            0b1000_1000,
            0b1000_0100,
            0b1000_0010,
        ];
        let mut plane = data
            .iter()
            .map(|&v| BitVec::from_element(v))
            .collect::<Vec<_>>();
        let quads = greedy_mesh_slice_dyn(&mut plane);
        for quad in quads.iter() {
            println!("{:?}", quad);
        }
        //visualize the quads
        let mut grid = [['.'; 8]; 8];
        for quad in quads {
            for y in quad.y..(quad.y + quad.height) {
                for x in quad.x..(quad.x + quad.width) {
                    grid[7 - x as usize][7 - y as usize] = '#';
                }
            }
        }
        grid.reverse();
        for row in grid.into_iter() {
            println!("{:?}", row);
        }
    }
}
