use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A, vec3, vec3a};
use reload::{csg::{csg_tree::tree::CSGTree, union::tree::{Union, UnionNode}}, util::{vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValue}, voxel::dag64::VoxelDAG64};

fn build_from_pos_query<V: Ve<i32, 3>, M: VolumeQureyPosValue<V, i32, 3>>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_pos_query_volume(model).unwrap();
    dag
}

fn build_from_pos_query_par<V: Ve<i32, 3>, M: VolumeQureyPosValue<V, i32, 3> + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(100000, 64);
    let mut dag = dag.parallel();
    dag.add_pos_query_volume(model).unwrap();
    dag.single()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Large Union");
    group.sample_size(10);

    let mut csg = CSGTree::<u8, IVec3, i32, 3>::new_sphere_float(Vec3A::ZERO, 10.0, 1);
    let mut union = Union::<u8, IVec3, i32, 3>::new();
    union.add_node(UnionNode::new_sphere(Vec3A::ZERO, 10.0, 1));

    for _ in 0..100 {
        let pos = vec3a(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 100.0;
        csg.union_sphere(pos, 10.0, 1);
        union.add_node(UnionNode::new_sphere(pos, 10.0, 1));
    }
    csg.calculate_bounds();
    union.calculate_bounds();

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "100 x sphere 10 pos"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "100 x sphere 10 pos par"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query_par(csg)));


    group.bench_with_input(
        BenchmarkId::new("dag 64 from union", "100 x sphere 10 pos "), 
        &union, 
        |b, union| 
        b.iter(|| build_from_pos_query(union)));

    group.bench_with_input(
        BenchmarkId::new("dag 64 from union", "100 x sphere 10 pos par"), 
        &union, 
        |b, union| 
        b.iter(|| build_from_pos_query_par(union))); 


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

