use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{UVec3, Vec3};
use reload::{csg::{csg_tree::tree::CSGTree, fast_query_csg_tree::tree::FastQueryCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, volume::{magica_voxel::MagicaVoxelModel, VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyPosValueI}, voxel::{dag64::VoxelDAG64, grid::{shared::SharedVoxelGrid, VoxelGrid}, renderer::palette::Palette}};

fn build_from_pos_query<M: VolumeQureyPosValueI>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_pos_query_volume(model).unwrap();
    dag
}

fn build_from_aabb_query<M: VolumeQureyAABBI>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_aabb_query_volume(model).unwrap();
    dag
}

fn build_from_pos_query_par<M: VolumeQureyPosValueI + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(100000, 64);
    let mut dag = dag.run_worker(10);
    dag.add_pos_query_volume(model).unwrap();
    dag.stop()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut grid = VoxelGrid::empty(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();
    grid.set_corners();

    c.bench_with_input(
        BenchmarkId::new("build dag 64 from grid", "sphere ^4"), 
        &grid, 
        |b, grid| 
        b.iter(|| build_from_pos_query(grid)));


    let csg = CSGTree::<u8>::new_sphere(Vec3::ZERO, 100.0);

    c.bench_with_input(
        BenchmarkId::new("build dag 64 from slotmap csg pos query", "sphere 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    c.bench_with_input(
        BenchmarkId::new("build dag 64 from slotmap csg aabb query", "sphere 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));

    c.bench_with_input(
        BenchmarkId::new("parallel build dag 64 from slotmap csg pos query", "sphere 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query_par(csg)));


    let csg: FastQueryCSGTree<u8> = csg.into();

    c.bench_with_input(
        BenchmarkId::new("build dag 64 from fast csg abb query", "sphere 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    /*
    c.bench_with_input(
        BenchmarkId::new("build dag 64 from fast csg abb query", "sphere 100"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));
    */   
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

