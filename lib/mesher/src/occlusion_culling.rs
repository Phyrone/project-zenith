use std::hash::Hash;

#[cfg(feature = "bevy")]
use bevy::{ecs::component::Component, reflect::Reflect};
use rayon::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "serde")]
use serde_big_array::BigArray;

use crate::FaceDirection;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(Component, Reflect))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VoxelCubeOcclusionMatrix64 {
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    x_axis: [u64; (u64::BITS * u64::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    y_axis: [u64; (u64::BITS * u64::BITS) as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    z_axis: [u64; (u64::BITS * u64::BITS) as usize],

    /* Neighbours */
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_zp: [u64; u64::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_zn: [u64; u64::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_yp: [u64; u64::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_yn: [u64; u64::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_xp: [u64; u64::BITS as usize],
    #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
    neib_xn: [u64; u64::BITS as usize],
}

impl VoxelCubeOcclusionMatrix64 {
    const SIZE_1_DIM: usize = u64::BITS as usize;
    const SIZE_2_DIM: usize = (u64::BITS * u64::BITS) as usize;
    const SIZE_3_DIM: usize = (u64::BITS * u64::BITS * u64::BITS) as usize;

    pub fn new() -> Self {
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
            .map(|(i, col)| {
                (
                    col,
                    i / Self::SIZE_1_DIM,
                    (i / Self::SIZE_1_DIM) % Self::SIZE_1_DIM,
                )
            })
            .for_each(|(col, x, y)| {
                let mut new_col = 0;
                for z in 0..Self::SIZE_1_DIM {
                    let solid = importer(x, y, z);
                    new_col |= (solid as u64) << z;
                }
                *col = new_col;
            });

        rayon::join(
            || {
                self.x_axis
                    .par_iter_mut()
                    .enumerate()
                    .map(|(i, col)| {
                        (
                            col,
                            i / Self::SIZE_1_DIM,
                            (i / Self::SIZE_1_DIM) % Self::SIZE_1_DIM,
                        )
                    })
                    .for_each(|(col, y, z)| {
                        let mut new_col = 0;
                        for x in 0..Self::SIZE_1_DIM {
                            let solid = self.z_axis[x + y * Self::SIZE_1_DIM] & (1 << z) != 0;
                            new_col |= (solid as u64) << x;
                        }
                        *col = new_col;
                    });
            },
            || {
                self.y_axis
                    .par_iter_mut()
                    .enumerate()
                    .map(|(i, col)| {
                        (
                            col,
                            i / Self::SIZE_1_DIM,
                            (i / Self::SIZE_1_DIM) % Self::SIZE_1_DIM,
                        )
                    })
                    .for_each(|(col, x, z)| {
                        let mut new_col = 0;
                        for y in 0..Self::SIZE_1_DIM {
                            let solid = self.z_axis[x + y * Self::SIZE_1_DIM] & (1 << z) != 0;
                            new_col |= (solid as u64) << y;
                        }
                        *col = new_col;
                    });
            },
        );
    }

    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, solid: bool) {
        if solid {
            self.z_axis[x + y * (u64::BITS as usize)] |= 1 << z;
            self.y_axis[x + z * (u64::BITS as usize)] |= 1 << y;
            self.x_axis[y + z * (u64::BITS as usize)] |= 1 << x;
        } else {
            self.z_axis[x + y * (u64::BITS as usize)] &= !(1 << z);
            self.y_axis[x + z * (u64::BITS as usize)] &= !(1 << y);
            self.x_axis[y + z * (u64::BITS as usize)] &= !(1 << x);
        }
    }

    const NEIB_POS_MASK: u64 = !(1 << (u64::BITS - 1));
    const NEIB_NEG_MASK: u64 = !1;

    #[inline]
    fn surfaces_mask(col: u64, neg: bool, neib: bool) -> u64 {
        let mask = col & (!if neg { (col >> 1) } else { (col << 1) });
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
    fn parse_surface_mask(mut mask: u64, mut cn: impl FnMut(usize)) {
        while mask != 0 {
            let k = mask.trailing_zeros() as usize;
            mask &= !(1 << k);
            cn(k);
        }
    }

    pub fn update_neighbour_out(
        &self,
        face_self: FaceDirection,
        neighbour: &mut VoxelCubeOcclusionMatrix64,
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

    #[inline]
    fn get_neib(neibs: &[u64; Self::SIZE_1_DIM], i: usize, j: usize) -> bool {
        let col = neibs[i];
        let bit = 1 << j;
        col & bit != 0
    }

    #[inline]
    pub fn find_surfaces(&self, mut found: impl FnMut(usize, usize, usize, FaceDirection)) {
        //lets pray the compiler flattens this (it should)
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
            for i in 0..Self::SIZE_1_DIM {
                for j in 0..Self::SIZE_1_DIM {
                    let (dir_pos, dir_neg) = match axis_i {
                        0 => (FaceDirection::XPos, FaceDirection::XNeg),
                        1 => (FaceDirection::YPos, FaceDirection::YNeg),
                        2 => (FaceDirection::ZPos, FaceDirection::ZNeg),
                        _ => unreachable!(),
                    };
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
