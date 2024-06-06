use std::hash::Hash;
use std::ops::{Neg, Not, Shl};

#[cfg(feature = "bevy")]
use bevy::{ecs::component::Component, reflect::Reflect};
use hashbrown::HashMap;
use rayon::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;
use smallvec::SmallVec;
use wide::u32x8;

use crate::{FaceDirection, GreedyQuad, MeshingResult};

pub fn build_mesh32<'a, S>(
    matrix: &VoxelCubeOcclusionMatrix32,
    get_surface: impl Fn(usize, usize, usize, &FaceDirection) -> Option<S>,
) -> MeshingResult<S>
where
    S: Hash + Eq + PartialEq + 'a,
{
    //Oclussion culling and grouping by type
    //direction -> surface -> slice
    let mut slice_by_axis_by_group: [HashMap<(S, usize), [u32; u32::BITS as usize]>; 6] =
        Default::default();

    //let mut time1 = std::time::Instant::now();
    matrix.find_surfaces::<true>(|x, y, z, face| {
        let surface = get_surface(x, y, z, &face);
        if let Some(surface) = surface {
            let (i, j, k) = face.absolute_to_axis_rel(x, y, z);

            let slice = slice_by_axis_by_group[face.to_index()]
                .entry((surface, k))
                .or_insert_with(|| [0; u32::BITS as usize]);
            slice[i] |= 1 << j;
        }
    });

    let mut grouped_quads = HashMap::new();
    for (axis, entries) in slice_by_axis_by_group.into_iter().enumerate() {
        let axis = FaceDirection::from_index(axis);
        for ((surface, k), slice) in entries {
            let mut quads = Vec::new();
            greedy_mesh_slice_32_no_alloc(slice, |i, j, w, h| {
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
pub fn greedy_mesh_slice_32_no_alloc(
    mut slice: [u32; 32],
    mut cb: impl FnMut(usize, usize, usize, usize),
) {
    const SIZE: usize = 32;

    for x in 0..SIZE {
        //as long as the row is not empty (zero) we look for quads
        while slice[x] != 0 {
            let row = slice[x];
            let y = row.trailing_zeros();

            //find the next contiguous occupied quads on the y axis
            let h = (row >> y).trailing_ones();
            //FIXME: overflow
            let mask = if h >= u32::BITS {
                u32::MAX
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
pub struct VoxelCubeOcclusionMatrix32 {
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    x_axis: [u32; (u32::BITS * u32::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    y_axis: [u32; (u32::BITS * u32::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    z_axis: [u32; (u32::BITS * u32::BITS) as usize],

    /* Neighbours */
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_zp: [u32; u32::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_zn: [u32; u32::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_yp: [u32; u32::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_yn: [u32; u32::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_xp: [u32; u32::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_xn: [u32; u32::BITS as usize],
}

impl Default for VoxelCubeOcclusionMatrix32 {
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

impl VoxelCubeOcclusionMatrix32 {
    const SIZE_1_DIM: usize = u32::BITS as usize;
    const SIZE_2_DIM: usize = (u32::BITS * u32::BITS) as usize;

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
                    *col |= (solid as u32) << z;
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
                            *col |= (solid as u32) << x;
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
                            *col |= (solid as u32) << y;
                        }
                    });
            },
        );
    }

    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, solid: bool) {
        if solid {
            self.x_axis[z + y * (u32::BITS as usize)] |= 1 << x;
            self.y_axis[x + z * (u32::BITS as usize)] |= 1 << y;
            self.z_axis[x + y * (u32::BITS as usize)] |= 1 << z;
        } else {
            self.x_axis[z + y * (u32::BITS as usize)] &= !(1 << x);
            self.y_axis[x + z * (u32::BITS as usize)] &= !(1 << y);
            self.z_axis[x + y* (u32::BITS as usize)] &= !(1 << z);
        }
    }

    const NEIB_POS_MASK: u32 = !(1 << (u32::BITS - 1));
    const NEIB_NEG_MASK: u32 = !1;
    const ALL_MASK: u32 = u32::MAX;

    #[inline]
    fn surfaces_mask(col: u32, neg: bool, neib: bool) -> u32 {
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
        col: [u32; 8],
        neib_pos: [bool; 8],
        neib_neg: [bool; 8],
    ) -> ([u32; 8], [u32; 8]) {
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
        let neib_pos_masks: u32x8 = wide::u32x8::from(neib_pos_masks);
        let neib_neg_masks: u32x8 = wide::u32x8::from(neib_neg_masks);

        let col: u32x8 = wide::u32x8::from(col);

        let tmp1: u32x8 = col >> 1;
        let tmp2: u32x8 = col << 1;
        let inverter: u32x8 = u32x8::from([u32::MAX; 8]);
        let tmp1 = tmp1 ^ inverter;
        let tmp2 = tmp2 ^ inverter;
        let mask1: u32x8 = col & tmp1;
        let mask2: u32x8 = col & tmp2;
        let mask1 = mask1 & neib_pos_masks;
        let mask2 = mask2 & neib_neg_masks;
        let mask1 = mask1.to_array();
        let mask2 = mask2.to_array();
        (mask1, mask2)
    }

    #[inline]
    fn parse_surface_mask(mut mask: u32, mut cn: impl FnMut(usize)) {
        while mask != 0 {
            let k = mask.trailing_zeros() as usize;
            mask &= !(1 << k);
            cn(k);
        }
    }

    pub fn update_neighbour_out(
        &self,
        face_self: FaceDirection,
        neighbour: &mut VoxelCubeOcclusionMatrix32,
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
    fn get_neib(neibs: &[u32; Self::SIZE_1_DIM], i: usize, j: usize) -> bool {
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
                const BLOCK_SIZE: usize = 8;
                for (p, data) in axis.chunks(BLOCK_SIZE).enumerate() {
                    assert_eq!(data.len(), BLOCK_SIZE);
                    let p = p * BLOCK_SIZE;
                    let data = [
                        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                    ];
                    let i0 = p % Self::SIZE_1_DIM;
                    let [i0, i1, i2, i3, i4, i5, i6, i7] =
                        [i0, i0 + 1, i0 + 2, i0 + 3, i0 + 4, i0 + 5, i0 + 6, i0 + 7];
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
                    ];
                    let (mask_pos, mask_neg) = Self::surfaces_mask_wide(data, neibs_pos, neibs_neg);
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
