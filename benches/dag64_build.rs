use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A};
use reload::{csg::{csg_tree::tree::CSGTree, sphere::CSGSphere}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, util::{math_config::{Float3D, Int3D}, number::Nu, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyPosValue}, voxel::{dag64::VoxelDAG64, grid::{shared::SharedVoxelGrid, VoxelGrid}, static_dag64::StaticVoxelDAG64}};

fn build_from_pos_query<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_pos_query_volume(model).unwrap();
    dag
}

fn build_from_aabb_query<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_aabb_query_volume(model).unwrap();
    dag
}

fn build_from_pos_query_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(1000000, 64);
    let mut dag = dag.parallel();
    dag.add_pos_query_volume(model).unwrap();
    dag.single()
}

fn build_from_aabb_query_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(1000000, 64);
    let mut dag = dag.parallel();
    dag.add_aabb_query_volume(model).unwrap();
    dag.single()
}

fn build_from_grid(grid: &VoxelGrid) -> StaticVoxelDAG64 {
    let tree64: StaticVoxelDAG64 = grid.into();
    tree64
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sphere 100");
    group.sample_size(100);

    let mut grid = VoxelGrid::empty(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();

    group.bench_with_input(
        BenchmarkId::new("tree 64 from grid", "pos int"), 
        &grid, 
        |b, grid| b.iter(|| build_from_grid(black_box(grid))));
    

    let sphere = CSGSphere::<u8, Int3D, 3>::new_sphere(Vec3A::ZERO, 100.0);

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "pos int"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_pos_query(sphere)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "aabb int"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_aabb_query(sphere)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "par pos int"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_pos_query_par(sphere)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "par aabb int"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_aabb_query_par(sphere)));


    let sphere = CSGSphere::<u8, Float3D, 3>::new_sphere(Vec3A::ZERO, 100.0);

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "pos float"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_pos_query(sphere)));
    
    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "aabb float"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_aabb_query(sphere)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "par pos float"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_pos_query_par(sphere)));
    
    group.bench_with_input(
        BenchmarkId::new("dag 64 from sphere", "par aabb float"), 
        &sphere, 
        |b, sphere| 
        b.iter(|| build_from_aabb_query_par(sphere)));





    let mut grid = VoxelGrid::empty(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();

    group.bench_with_input(
        BenchmarkId::new("dag 64 from grid", "pos int"), 
        &grid, 
        |b, grid| 
        b.iter(|| build_from_pos_query::<IVec3, i32, _>(grid)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from grid", "pos float"), 
        &grid, 
        |b, grid| 
        b.iter(|| build_from_pos_query::<Vec3A, f32, _>(grid)));
 


    let mut csg = CSGTree::<u8, Int3D, 3>::new_sphere(Vec3A::ZERO, 100.0);

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "pos int"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "aabb int"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "par pos int"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query_par(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "par aabb int"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query_par(csg)));


    let mut csg = CSGTree::<u8, Float3D, 3>::new_sphere(Vec3A::ZERO, 100.0);

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "pos float"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "aabb float"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "par pos float"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query_par(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "par aabb float"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_aabb_query_par(csg)));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

