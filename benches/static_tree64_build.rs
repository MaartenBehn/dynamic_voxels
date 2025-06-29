use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::UVec3;
use reload::{static_voxel_dag64::StaticVoxelDAG64, voxel_grid::VoxelGrid};

fn build_from_grid(grid: &VoxelGrid) -> StaticVoxelDAG64 {
    let tree64: StaticVoxelDAG64 = grid.into();
    tree64
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();
    grid.set_corners();

    c.bench_with_input(
        BenchmarkId::new("build static tree 64 from grid", "sphere grid ^4"), 
        &grid, 
        |b, grid| b.iter(|| build_from_grid(black_box(grid))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
