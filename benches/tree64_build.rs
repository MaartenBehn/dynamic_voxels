use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{UVec3, Vec3};
use reload::{csg::{fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::SlotMapCSGTree, vec_csg_tree::tree::VecCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, volume::VolumeQureyAABB, voxel::{dag64::VoxelDAG64, grid::VoxelGrid}};

fn build_from_grid(grid: &VoxelGrid) -> VoxelDAG64 {
    let tree64: VoxelDAG64 = VoxelDAG64::from_pos_query(grid).unwrap();
    tree64
}

fn build_from_aabb_query<M: VolumeQureyAABB>(model: &M) -> VoxelDAG64 {
    let tree64: VoxelDAG64 = VoxelDAG64::from_aabb_query(model).unwrap();
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

    let csg: FastQueryCSGTree<u8> = VecCSGTree::new_sphere(Vec3::ZERO, 100.0).into();

    c.bench_with_input(
        BenchmarkId::new("build tree 64 from fast csg", "sphere grid 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));

    let csg = SlotMapCSGTree::<u8>::new_sphere(Vec3::ZERO, 100.0);

    c.bench_with_input(
        BenchmarkId::new("build tree 64 from slotmap csg", "sphere grid 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

