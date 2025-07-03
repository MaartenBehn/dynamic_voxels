use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::UVec3;
use reload::{multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, voxel_dag64::VoxelDAG64, voxel_grid::VoxelGrid};

fn build_from_grid(grid: &VoxelGrid) -> VoxelDAG64 {
    let buffer_size = 2_usize.pow(30);
    let mut allocator = BuddyBufferAllocator::new(buffer_size, 32);
    let tree64: VoxelDAG64 = VoxelDAG64::from_pos_query(grid, &mut allocator).unwrap();
    tree64
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut grid = VoxelGrid::new(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();
    grid.set_corners();


    c.bench_with_input(
        BenchmarkId::new("build tree 64 from grid", "sphere grid ^4"), 
        &grid, 
        |b, grid| 
        b.iter(|| build_from_grid(grid)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
