use building_blocks_core::prelude::*;
use building_blocks_storage::prelude::*;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn array_for_each_stride(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_for_each_stride");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_array(size),
                |(array, iter_extent)| {
                    array.for_each(&iter_extent, |stride: Stride, value| {
                        black_box((stride, value));
                    });
                },
            );
        });
    }
    group.finish();
}

fn array_for_each_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_for_each_point");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_array(size),
                |(array, iter_extent)| {
                    array.for_each(&iter_extent, |p: Point3i, value| {
                        black_box((p, value));
                    });
                },
            );
        });
    }
    group.finish();
}

fn array_for_each_point_and_stride(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_for_each_point_and_stride");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_array(size),
                |(array, iter_extent)| {
                    array.for_each(&iter_extent, |(p, stride): (Point3i, Stride), value| {
                        black_box((p, stride, value));
                    });
                },
            );
        });
    }
    group.finish();
}

fn chunk_map_for_each_point(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_map_for_each_point");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_chunk_map(size),
                |(chunk_map, iter_extent)| {
                    let local_cache = LocalChunkCache3::new();
                    let reader = ChunkMapReader3::new(&chunk_map, &local_cache);

                    reader.for_each(&iter_extent, |p: Point3i, value| {
                        black_box((p, value));
                    });
                },
            );
        });
    }
    group.finish();
}

fn array_point_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_point_indexing");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_array(size),
                |(array, iter_extent)| {
                    for p in iter_extent.iter_points() {
                        black_box(array.get(&p));
                    }
                },
            );
        });
    }
    group.finish();
}

fn chunk_map_point_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_map_point_indexing");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || set_up_chunk_map(size),
                |(chunk_map, iter_extent)| {
                    let local_cache = LocalChunkCache3::new();
                    let reader = ChunkMapReader3::new(&chunk_map, &local_cache);

                    for p in iter_extent.iter_points() {
                        black_box(reader.get(&p));
                    }
                },
            );
        });
    }
    group.finish();
}

fn array_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("array_copy");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let array_extent =
                        Extent3::from_min_and_shape(PointN([0; 3]), PointN([size; 3]));
                    let array_src = Array3::fill(array_extent, 1);
                    let array_dst = Array3::fill(array_extent, 0);

                    let cp_extent = array_extent.padded(-1);

                    (array_src, array_dst, cp_extent)
                },
                |(src, mut dst, cp_extent)| {
                    copy_extent(&cp_extent, &src, &mut dst);
                },
            );
        });
    }
    group.finish();
}

fn chunk_map_copy(c: &mut Criterion) {
    let mut group = c.benchmark_group("chunk_map_copy");
    for size in ARRAY_SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let cp_extent = Extent3::from_min_and_shape(PointN([0; 3]), PointN([size; 3]));
                    let mut src = default_chunk_map();
                    src.fill_extent(&cp_extent, 1);

                    let dst = default_chunk_map();

                    (src, dst, cp_extent)
                },
                |(src, mut dst, cp_extent)| {
                    let local_cache = LocalChunkCache3::new();
                    let src_reader = ChunkMapReader3::new(&src, &local_cache);
                    copy_extent(&cp_extent, &src_reader, &mut dst);
                },
            );
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    array_for_each_stride,
    array_for_each_point,
    array_for_each_point_and_stride,
    array_point_indexing,
    array_copy,
    chunk_map_for_each_point,
    chunk_map_point_indexing,
    chunk_map_copy
);
criterion_main!(benches);

const ARRAY_SIZES: [i32; 3] = [16, 32, 64];

fn set_up_array(size: i32) -> (Array3<i32>, Extent3i) {
    let array_extent = Extent3::from_min_and_shape(PointN([0; 3]), PointN([size; 3]));
    let array = Array3::fill(array_extent, 1);

    let iter_extent = array_extent.padded(-1);

    (array, iter_extent)
}

fn set_up_chunk_map(size: i32) -> (ChunkMap3<i32, ()>, Extent3i) {
    let mut map = default_chunk_map();
    let iter_extent = Extent3i::from_min_and_shape(PointN([0; 3]), PointN([size; 3]));
    map.fill_extent(&iter_extent, 1);

    (map, iter_extent)
}

fn default_chunk_map() -> ChunkMap3<i32, ()> {
    let chunk_shape = PointN([16; 3]);
    let ambient_value = 0;

    ChunkMap3::new(chunk_shape, ambient_value, (), Lz4 { level: 10 })
}
