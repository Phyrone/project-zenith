#![allow(unused_imports)]

use criterion::{criterion_group, criterion_main, Criterion};
use rayon::prelude::*;
use std::hint::black_box;

use mesher::b16::build_mesh16;
use utils::*;

#[cfg(test)]
mod utils;

criterion_group!(benches, bench_greedy_meshing);
criterion_main!(benches);

const SIZE: usize = 16;
const VOLUME: usize = SIZE * SIZE * SIZE;

fn bench_greedy_meshing(criterion: &mut Criterion) {
    criterion
        .benchmark_group("b16")
        //almost worst case but insignoficant difference
        .bench_function("bad case, existing occl-matrix", |bencher| {
            let cube = build_bad_case_voxel_cube::<256, VOLUME>();
            let matrix = build_occlusion_matrix16::<false>(&cube);
            bencher.iter(|| {
                let quads = build_mesh16(black_box(&matrix), |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box(quads);
            });
        })
        .bench_function("bad case, sync occl-matrix", |bencher| {
            let cube = build_bad_case_voxel_cube::<256, VOLUME>();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box(&cube);
                let matrix = build_occlusion_matrix16::<false>(&cube);
                let quads = build_mesh16(&matrix, |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix));
            });
        })
        .bench_function("bad case, par occl-matrix", |bencher| {
            let cube = build_bad_case_voxel_cube::<256, VOLUME>();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box(&cube);
                let matrix = build_occlusion_matrix16::<false>(cube);
                let quads = build_mesh16(&matrix, |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix))
            });
        })
        .bench_function("full, existing occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube::<VOLUME>();
            let matrix = build_occlusion_matrix16::<false>(&cube);

            bencher.iter(|| {
                let (cube, matrix) = black_box((&cube, &matrix));
                let quads = build_mesh16(matrix, |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box(quads)
            });
        })
        .bench_function("full, sync occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube::<VOLUME>();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box(&cube);
                let matrix = build_occlusion_matrix16::<false>(cube);
                let quads = build_mesh16(&matrix, |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix))
            });
        })
        .bench_function("full, par occl-matrix", |bencher| {
            let cube = build_filled_voxel_cube::<VOLUME>();
            bencher.iter_with_large_drop(|| {
                let (cube) = black_box((&cube));
                let matrix = build_occlusion_matrix16::<false>(&cube);
                let quads = build_mesh16(&matrix, |x, y, z, _| {
                    let voxel = cube[x + y * SIZE + z * SIZE * SIZE];
                    if voxel.id != 0 {
                        Some(voxel.id)
                    } else {
                        None
                    }
                });
                black_box((quads, matrix))
            });
        });
}
