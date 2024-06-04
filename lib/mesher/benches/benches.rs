#![allow(unused_imports)]

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use rayon::prelude::*;

use mesher::build_mesh64;
#[cfg(feature = "bench-slices")]
use mesher::{
    greedy_mesh_binary_plane, greedy_mesh_slice_16, greedy_mesh_slice_32, greedy_mesh_slice_64,
    greedy_mesh_slice_8, greedy_mesh_slice_dyn,
};
use utils::*;

#[cfg(test)]
mod utils;

criterion_group!(benches, bench_greedy_meshing);
criterion_main!(benches);

fn bench_greedy_meshing(criterion: &mut Criterion) {
    criterion
        .benchmark_group("sub functions")
        .bench_function("volume to len", |bencher| {
            bencher.iter(|| {
                let len = mesher::volume_to_len(black_box(32768));
                black_box(len);
            });
        });

    criterion
        .benchmark_group("create-mesh")
        //almost worst case but insignoficant difference
        .bench_function("worst case, existing occl-matrix", |bencher| {
            let cube = build_worst_case_voxel_cube();
            bencher.iter_batched(
                || {
                    let cube = black_box(&cube);
                    let matrix = build_occlusion_matrix::<false>(cube);
                    (cube, matrix)
                },
                |(cube, matrix)| {
                    let quads = build_mesh64(&matrix, |x, y, z, _| {
                        if cube[x + y * 64 + z * 64 * 64].solid {
                            Some(0)
                        } else {
                            None
                        }
                    });
                    black_box((quads))
                },
                BatchSize::LargeInput,
            );
        })
        .bench_function("worst case, sync occl-matrix", |bencher| {
            let cube = build_worst_case_voxel_cube();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box((&cube));
                let matrix = build_occlusion_matrix::<false>(&cube);
                let quads = build_mesh64(&matrix, |x, y, z, _| {
                    if cube[x + y * 64 + z * 64 * 64].solid {
                        Some(0)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix));
            });
        })
        .bench_function("worst case, par occl-matrix", |bencher| {
            let cube = build_worst_case_voxel_cube();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box((&cube));
                let matrix = build_occlusion_matrix::<true>(&cube);
                let quads = build_mesh64(&matrix, |x, y, z, _| {
                    if cube[x + y * 64 + z * 64 * 64].solid {
                        Some(0)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix));
            });
        })
        .bench_function("full, existing occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube();
            let matrix = build_occlusion_matrix::<false>(&cube);

            bencher.iter(|| {
                let (cube, matrix) = black_box((&cube, &matrix));
                let quads = build_mesh64(matrix, |x, y, z, _| {
                    if cube[x + y * 64 + z * 64 * 64].solid {
                        Some(0)
                    } else {
                        None
                    }
                });
                black_box((quads));
            });
        })
        .bench_function("full, sync occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box((&cube));
                let matrix = build_occlusion_matrix::<false>(&cube);
                let quads = build_mesh64(&matrix, |x, y, z, _| {
                    if cube[x + y * 64 + z * 64 * 64].solid {
                        Some(0)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix));
            });
        })
        .bench_function("full, par occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box((&cube));
                let matrix = build_occlusion_matrix::<true>(&cube);
                let quads = build_mesh64(&matrix, |x, y, z, _| {
                    if cube[x + y * 64 + z * 64 * 64].solid {
                        Some(0)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix));
            });
        });

    #[cfg(feature = "bench-slices")]
    criterion
        .benchmark_group("greedy meshing 1 slice")
        .bench_function("64x64, random", |bencher| {
            bencher.iter_with_setup(
                || DATA_64,
                |data| {
                    let quads = greedy_mesh_slice_64(black_box(data));
                    black_box(quads);
                },
            );
        })
        .bench_function("32x32, random", |bencher| {
            bencher.iter_with_setup(
                || DATA_32,
                |data| {
                    let quads = greedy_mesh_slice_32(black_box(data));
                    black_box(quads);
                },
            );
        })
        .bench_function("16x16, random", |bencher| {
            bencher.iter_with_setup(
                || DATA_16,
                |data| {
                    let quads = greedy_mesh_slice_16(black_box(data));
                    black_box(quads);
                },
            );
        })
        .bench_function("8x8, random", |bencher| {
            bencher.iter_with_setup(
                || DATA_8,
                |data| {
                    let quads = greedy_mesh_slice_8(black_box(data));
                    black_box(quads);
                },
            );
        })
        .bench_function("yt-reference 32x32, random", |bencher| {
            bencher.iter_with_setup(
                || DATA_32,
                |data| {
                    let quads = greedy_mesh_binary_plane(black_box(data), 32);
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (64x64, random)", |bencher| {
            bencher.iter_with_setup(
                || {
                    DATA_64
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (32x32, random)", |bencher| {
            bencher.iter_with_setup(
                || {
                    DATA_32
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (16x16, random)", |bencher| {
            bencher.iter_with_setup(
                || {
                    DATA_16
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (8x8, random)", |bencher| {
            bencher.iter_with_setup(
                || {
                    DATA_8
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("64x64, filled 1", |bencher| {
            bencher.iter_with_setup(
                || [1u64; 64],
                |data| {
                    let quads = greedy_mesh_slice_64(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("32x32, filled 1", |bencher| {
            bencher.iter_with_setup(
                || [1u32; 32],
                |data| {
                    let quads = greedy_mesh_slice_32(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("16x16, filled 1", |bencher| {
            bencher.iter_with_setup(
                || [1u16; 16],
                |data| {
                    let quads = greedy_mesh_slice_16(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("8x8, filled 1", |bencher| {
            bencher.iter_with_setup(
                || [1u8; 8],
                |data| {
                    let quads = greedy_mesh_slice_8(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("yt-reference 32x32, filled 1", |bencher| {
            bencher.iter_with_setup(
                || [1u32; 32],
                |data| {
                    let quads = greedy_mesh_binary_plane(black_box(data), 32);
                    black_box(quads);
                },
            )
        })
        .bench_function("dyn (64x64), filled 1", |bencher| {
            bencher.iter_with_setup(
                || {
                    [1u64; 64]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (32x32), filled 1", |bencher| {
            bencher.iter_with_setup(
                || {
                    [1u32; 32]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (16x16), filled 1", |bencher| {
            bencher.iter_with_setup(
                || {
                    [1u16; 16]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (8x8), filled 1", |bencher| {
            bencher.iter_with_setup(
                || {
                    [1u8; 8]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("64x64, empty", |bencher| {
            bencher.iter_with_setup(
                || [0u64; 64],
                |data| {
                    let quads = greedy_mesh_slice_64(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("32x32, empty", |bencher| {
            bencher.iter_with_setup(
                || [0u32; 32],
                |data| {
                    let quads = greedy_mesh_slice_32(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("16x16, empty", |bencher| {
            bencher.iter_with_setup(
                || [0u16; 16],
                |data| {
                    let quads = greedy_mesh_slice_16(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("8x8, empty", |bencher| {
            bencher.iter_with_setup(
                || [0u8; 8],
                |data| {
                    let quads = greedy_mesh_slice_8(black_box(data));
                    black_box(quads);
                },
            )
        })
        .bench_function("yt-reference 32x32, empty", |bencher| {
            bencher.iter_with_setup(
                || [0u32; 32],
                |data| {
                    let quads = greedy_mesh_binary_plane(black_box(data), 32);
                    black_box(quads);
                },
            )
        })
        .bench_function("dyn (64x64), empty", |bencher| {
            bencher.iter_with_setup(
                || {
                    [0u64; 64]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (32x32), empty", |bencher| {
            bencher.iter_with_setup(
                || {
                    [0u32; 32]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (16x16), empty", |bencher| {
            bencher.iter_with_setup(
                || {
                    [0u16; 16]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        })
        .bench_function("dyn (8x8), empty", |bencher| {
            bencher.iter_with_setup(
                || {
                    [0u8; 8]
                        .iter()
                        .map(|&col| BitVec::from_element(col))
                        .collect::<Vec<_>>()
                },
                |mut data| {
                    let quads = greedy_mesh_slice_dyn(black_box(&mut data));
                    black_box(quads);
                },
            );
        });
}
