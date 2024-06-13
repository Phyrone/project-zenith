use std::hash::Hash;
use std::hint::black_box;
use std::mem::size_of_val;
use std::ops::Not;

#[cfg(feature = "bevy")]
use bevy::{ecs::component::Component, reflect::Reflect};
use hashbrown::HashMap;
use rayon::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;
use smallvec::SmallVec;
use wide::u16x16;

use crate::{FaceDirection, GreedyQuad, MeshingResult};

pub fn build_mesh16<'a, S>(
    matrix: &VoxelCubeOcclusionMatrix16,
    get_surface: impl Fn(usize, usize, usize, &FaceDirection) -> Option<S>,
) -> MeshingResult<S>
where
    S: Hash + Eq + PartialEq + 'a,
{
    //Oclussion culling and grouping by type
    //direction -> surface -> slice
    let mut slice_by_axis_by_group: [HashMap<(S, usize), [u16; u16::BITS as usize]>; 6] =
        Default::default();

    //let mut time1 = std::time::Instant::now();
    matrix.find_surfaces::<true>(|x, y, z, face| {
        let surface = get_surface(x, y, z, &face);
        if let Some(surface) = surface {
            let (i, j, k) = face.absolute_to_axis_rel(x, y, z);

            let slice = slice_by_axis_by_group[face.to_index()]
                .entry((surface, k))
                .or_insert_with(|| [0; u16::BITS as usize]);
            slice[i] |= 1 << j;
        }
    });

    let mut grouped_quads = HashMap::new();
    for (axis, entries) in slice_by_axis_by_group.into_iter().enumerate() {
        let axis = FaceDirection::from_index(axis);
        for ((surface, k), slice) in entries {
            let mut quads = Vec::new();
            greedy_mesh_slice_16_no_alloc(slice, |i, j, w, h| {
                let (x, y, z) = axis.axis_rel_to_absolute(i, j, k);
                quads.push(GreedyQuad {
                    direction: axis,
                    x,
                    y,
                    z,
                    w,
                    h,
                });
            });
            grouped_quads
                .entry(surface)
                .or_insert_with(SmallVec::new)
                .extend(quads);
        }
    }
    grouped_quads
}

#[inline]
pub fn greedy_mesh_slice_16_no_alloc(
    mut slice: [u16; 16],
    mut cb: impl FnMut(usize, usize, usize, usize),
) {
    const SIZE: usize = 16;

    for x in 0..SIZE {
        //as long as the row is not empty (zero) we look for quads
        while slice[x] != 0 {
            let row = slice[x];
            let y = row.trailing_zeros();

            //find the next contiguous occupied quads on the y axis
            let h = (row >> y).trailing_ones();
            //FIXME: overflow
            let mask = if h >= u16::BITS {
                u16::MAX
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(Component, Reflect))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VoxelCubeOcclusionMatrix16 {
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    x_axis: [u16; (u16::BITS * u16::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    y_axis: [u16; (u16::BITS * u16::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    z_axis: [u16; (u16::BITS * u16::BITS) as usize],

    /* Neighbours */
    neib_zp: [u16; u16::BITS as usize],
    neib_zn: [u16; u16::BITS as usize],
    neib_yp: [u16; u16::BITS as usize],
    neib_yn: [u16; u16::BITS as usize],
    neib_xp: [u16; u16::BITS as usize],
    neib_xn: [u16; u16::BITS as usize],
}

impl Default for VoxelCubeOcclusionMatrix16 {
    fn default() -> Self {
        Self {
            x_axis: [0; Self::SIZE_2_DIM],
            y_axis: [0; Self::SIZE_2_DIM],
            z_axis: [0; Self::SIZE_2_DIM],
            neib_zp: [0; Self::SIZE_1_DIM],
            neib_zn: [0; Self::SIZE_1_DIM],
            neib_yp: [0; Self::SIZE_1_DIM],
            neib_yn: [0; Self::SIZE_1_DIM],
            neib_xp: [0; Self::SIZE_1_DIM],
            neib_xn: [0; Self::SIZE_1_DIM],
        }
    }
}

impl VoxelCubeOcclusionMatrix16 {
    const SIZE_1_DIM: usize = u16::BITS as usize;
    const SIZE_2_DIM: usize = (u16::BITS * u16::BITS) as usize;

    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn import(&mut self, importer: impl Fn(usize, usize, usize) -> bool) {
        for x in 0..Self::SIZE_1_DIM {
            for y in 0..Self::SIZE_1_DIM {
                for z in 0..Self::SIZE_1_DIM {
                    let solid = importer(x, y, z);
                    self.set_voxel(x, y, z, solid);
                }
            }
        }
    }

    pub fn par_import<F>(&mut self, importer: F)
    where
        F: Fn(usize, usize, usize) -> bool + Sync + Send,
    {
        //import z axis -> import x and y from z
        self.z_axis
            .par_iter_mut()
            .enumerate()
            .map(|(i, col)| (col, i % Self::SIZE_1_DIM, i / Self::SIZE_1_DIM))
            .for_each(|(col, x, y)| {
                for z in 0..Self::SIZE_1_DIM {
                    let solid = importer(x, y, z);
                    *col |= (solid as u16) << z;
                }
            });

        rayon::join(
            || {
                self.x_axis
                    .par_iter_mut()
                    .enumerate()
                    .map(|(i, col)| (col, i % Self::SIZE_1_DIM, i / Self::SIZE_1_DIM))
                    .for_each(|(col, y, z)| {
                        for x in 0..Self::SIZE_1_DIM {
                            let solid = self.z_axis[x + y * Self::SIZE_1_DIM] & (1 << z) != 0;
                            *col |= (solid as u16) << x;
                        }
                    });
            },
            || {
                self.y_axis
                    .par_iter_mut()
                    .enumerate()
                    .map(|(i, col)| (col, i % Self::SIZE_1_DIM, i / Self::SIZE_1_DIM))
                    .for_each(|(col, x, z)| {
                        for y in 0..Self::SIZE_1_DIM {
                            let solid = self.z_axis[x + y * Self::SIZE_1_DIM] & (1 << z) != 0;
                            *col |= (solid as u16) << y;
                        }
                    });
            },
        );
    }

    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, solid: bool) {
        if solid {
            self.x_axis[z + y * (u16::BITS as usize)] |= 1 << x;
            self.y_axis[x + z * (u16::BITS as usize)] |= 1 << y;
            self.z_axis[x + y * (u16::BITS as usize)] |= 1 << z;
        } else {
            self.x_axis[z + y * (u16::BITS as usize)] &= !(1 << x);
            self.y_axis[x + z * (u16::BITS as usize)] &= !(1 << y);
            self.z_axis[x + y * (u16::BITS as usize)] &= !(1 << z);
        }
    }

    const NEIB_POS_MASK: u16 = !(1 << (u16::BITS - 1));
    const NEIB_NEG_MASK: u16 = !1;
    const ALL_MASK: u16 = u16::MAX;

    #[inline]
    fn surfaces_mask(col: u16, neg: bool, neib: bool) -> u16 {
        let mask = col & (!if neg { (col << 1) } else { (col >> 1) });
        if neib {
            mask & if neg {
                Self::NEIB_NEG_MASK
            } else {
                Self::NEIB_POS_MASK
            }
        } else {
            mask
        }
    }

    #[inline]
    fn surfaces_mask_wide(
        col: [u16; 16],
        neib_pos: [bool; 16],
        neib_neg: [bool; 16],
    ) -> ([u16; 16], [u16; 16]) {
        let col: u16x16 = u16x16::from(col);

        let neib_pos_masks = neib_pos.map(|presetn| {
            if presetn {
                Self::NEIB_POS_MASK
            } else {
                Self::ALL_MASK
            }
        });
        let neib_neg_masks = neib_neg.map(|presetn| {
            if presetn {
                Self::NEIB_NEG_MASK
            } else {
                Self::ALL_MASK
            }
        });
        let neib_pos_masks: u16x16 = u16x16::from(neib_pos_masks);
        let neib_neg_masks: u16x16 = u16x16::from(neib_neg_masks);

        let tmp1: u16x16 = col >> 1;
        let tmp2: u16x16 = col << 1;
        //for some reason bitwise not does not work here
        let inverter: u16x16 = u16x16::new([u16::MAX; 16]);
        let tmp1: u16x16 = tmp1 ^ inverter;
        let tmp2: u16x16 = tmp2 ^ inverter;
        let mask1: u16x16 = col & tmp1;
        let mask2: u16x16 = col & tmp2;
        let mask1 = mask1 & neib_pos_masks;
        let mask2 = mask2 & neib_neg_masks;
        let mask1 = mask1.to_array();
        let mask2 = mask2.to_array();
        (mask1, mask2)
    }

    #[inline]
    fn parse_surface_mask(mut mask: u16, mut cn: impl FnMut(usize)) {
        while mask != 0 {
            let k = mask.trailing_zeros() as usize;
            mask &= !(1 << k);
            cn(k);
        }
    }

    pub fn update_neighbour_out(
        &self,
        face_self: FaceDirection,
        neighbour: &mut VoxelCubeOcclusionMatrix16,
    ) {
        let offset_self = if (face_self.is_positive()) {
            Self::SIZE_2_DIM - Self::SIZE_1_DIM
        } else {
            0
        };
        
        let axis = match face_self {
            FaceDirection::XNeg | FaceDirection::XPos => &self.y_axis,
            FaceDirection::YNeg | FaceDirection::YPos => &self.z_axis,
            FaceDirection::ZNeg | FaceDirection::ZPos => &self.x_axis,
        };
        let neib_axis = match face_self {
            FaceDirection::XNeg => &mut neighbour.neib_xp,
            FaceDirection::XPos => &mut neighbour.neib_xn,
            FaceDirection::YNeg => &mut neighbour.neib_yp,
            FaceDirection::YPos => &mut neighbour.neib_yn,
            FaceDirection::ZNeg => &mut neighbour.neib_zp,
            FaceDirection::ZPos => &mut neighbour.neib_zn,
        };

        neib_axis[..Self::SIZE_1_DIM]
            .copy_from_slice(&axis[offset_self..(Self::SIZE_1_DIM + offset_self)]);
    }

    //FIXME: swap i and j later in the future to not make muliple array accesses make the code more cache friendly
    #[inline]
    fn get_neib(neibs: &[u16; Self::SIZE_1_DIM], i: usize, j: usize) -> bool {
        let col = neibs[i];
        let bit = 1 << j;
        col & bit != 0
    }

    #[inline]
    pub fn find_surfaces<const SIMD: bool>(
        &self,
        mut found: impl FnMut(usize, usize, usize, FaceDirection),
    ) {
        for axis_i in 0..3 {
            let axis = match axis_i {
                0 => &self.x_axis,
                1 => &self.y_axis,
                2 => &self.z_axis,
                _ => unreachable!(),
            };
            let (pos_nib, neg_nib) = match axis_i {
                0 => (&self.neib_xp, &self.neib_xn),
                1 => (&self.neib_yp, &self.neib_yn),
                2 => (&self.neib_zp, &self.neib_zn),
                _ => unreachable!(),
            };

            let (dir_pos, dir_neg) = match axis_i {
                0 => (FaceDirection::XPos, FaceDirection::XNeg),
                1 => (FaceDirection::YPos, FaceDirection::YNeg),
                2 => (FaceDirection::ZPos, FaceDirection::ZNeg),
                _ => unreachable!(),
            };
            if SIMD {
                const BLOCK_SIZE: usize = 16;
                for (p, data) in axis.chunks(BLOCK_SIZE).enumerate() {
                    assert_eq!(data.len(), BLOCK_SIZE);
                    let p = p * BLOCK_SIZE;
                    let data = [
                        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                        data[8], data[9], data[10], data[11], data[12], data[13], data[14],
                        data[15],
                    ];
                    let i0 = p % Self::SIZE_1_DIM;
                    //FIXME: bah that's ugly fix it later
                    let [i0, i1, i2, i3, i4, i5, i6, i7, i8, i9, i10, i11, i12, i13, i14, i15] = [
                        i0,
                        i0 + 1,
                        i0 + 2,
                        i0 + 3,
                        i0 + 4,
                        i0 + 5,
                        i0 + 6,
                        i0 + 7,
                        i0 + 8,
                        i0 + 9,
                        i0 + 10,
                        i0 + 11,
                        i0 + 12,
                        i0 + 13,
                        i0 + 14,
                        i0 + 15,
                    ];
                    //j is always the same for all 4 elements
                    let j = p / Self::SIZE_1_DIM;
                    let neibs_pos = [
                        Self::get_neib(pos_nib, i0, j),
                        Self::get_neib(pos_nib, i1, j),
                        Self::get_neib(pos_nib, i2, j),
                        Self::get_neib(pos_nib, i3, j),
                        Self::get_neib(pos_nib, i4, j),
                        Self::get_neib(pos_nib, i5, j),
                        Self::get_neib(pos_nib, i6, j),
                        Self::get_neib(pos_nib, i7, j),
                        Self::get_neib(pos_nib, i8, j),
                        Self::get_neib(pos_nib, i9, j),
                        Self::get_neib(pos_nib, i10, j),
                        Self::get_neib(pos_nib, i11, j),
                        Self::get_neib(pos_nib, i12, j),
                        Self::get_neib(pos_nib, i13, j),
                        Self::get_neib(pos_nib, i14, j),
                        Self::get_neib(pos_nib, i15, j),
                    ];
                    let neibs_neg = [
                        Self::get_neib(neg_nib, i0, j),
                        Self::get_neib(neg_nib, i1, j),
                        Self::get_neib(neg_nib, i2, j),
                        Self::get_neib(neg_nib, i3, j),
                        Self::get_neib(neg_nib, i4, j),
                        Self::get_neib(neg_nib, i5, j),
                        Self::get_neib(neg_nib, i6, j),
                        Self::get_neib(neg_nib, i7, j),
                        Self::get_neib(neg_nib, i8, j),
                        Self::get_neib(neg_nib, i9, j),
                        Self::get_neib(neg_nib, i10, j),
                        Self::get_neib(neg_nib, i11, j),
                        Self::get_neib(neg_nib, i12, j),
                        Self::get_neib(neg_nib, i13, j),
                        Self::get_neib(neg_nib, i14, j),
                        Self::get_neib(neg_nib, i15, j),
                    ];
                    let ( mask_pos,mask_neg) = Self::surfaces_mask_wide(data, neibs_pos, neibs_neg);
                    for (o, mask_pos) in mask_pos.into_iter().enumerate() {
                        Self::parse_surface_mask(mask_pos, |k| {
                            let (x, y, z) = dir_pos.axis_rel_to_absolute(i0 + o, j, k);
                            found(x, y, z, dir_pos)
                        });
                    }
                    for (o, mask_neg) in mask_neg.into_iter().enumerate() {
                        Self::parse_surface_mask(mask_neg, |k| {
                            let (x, y, z) = dir_neg.axis_rel_to_absolute(i0 + o, j, k);
                            found(x, y, z, dir_neg)
                        });
                    }
                }
            } else {
                for (p, &data) in axis.iter().enumerate() {
                    let i = p % Self::SIZE_1_DIM;
                    let j = p / Self::SIZE_1_DIM;
                    let col = axis[i + j * Self::SIZE_1_DIM];

                    let (neib_pos, neib_neg) =
                        (Self::get_neib(pos_nib, i, j), Self::get_neib(neg_nib, i, j));
                    let (mask_pos, mask_neg) = (
                        Self::surfaces_mask(col, false, neib_pos),
                        Self::surfaces_mask(col, true, neib_neg),
                    );

                    Self::parse_surface_mask(mask_pos, |k| {
                        let (x, y, z) = dir_pos.axis_rel_to_absolute(i, j, k);
                        found(x, y, z, dir_pos)
                    });

                    Self::parse_surface_mask(mask_neg, |k| {
                        let (x, y, z) = dir_neg.axis_rel_to_absolute(i, j, k);
                        found(x, y, z, dir_neg)
                    });
                }
            }
        }
    }
}
