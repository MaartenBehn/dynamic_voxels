use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use octa_force::glam::{vec3, vec3a, UVec3, Vec3, Vec3A};
use reload::{csg::union::tree::{CSGUnion, CSGUnionNode}, util::{math_config::Int3D, vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValue}, voxel::dag64::VoxelDAG64};

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
    let mut group = c.benchmark_group("Sample 10");
    group.sample_size(10);

    //let mut csg = CSGTree::<u8>::new_sphere(Vec3::ZERO, 10.0);
    let mut union = CSGUnion::<u8, Int3D, 3>::new();
    union.add_node(CSGUnionNode::new_sphere(Vec3A::ZERO, 10.0));

    for _ in 0..100 {
        let pos = vec3a(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 100.0;
        //csg.append_node_with_union(CSGNode::new_sphere(pos, 10.0));

        union.add_node(CSGUnionNode::new_sphere(pos, 10.0));
    }
    //csg.set_all_aabbs();
    union.calculate_bounds();

    /*
    group.bench_with_input(
        BenchmarkId::new("build dag 64 from slotmap csg pos query", "100 x sphere 10"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query(csg)));

    group.bench_with_input(
        BenchmarkId::new("parallel build dag 64 from slotmap csg pos query", "100 x sphere 10"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_from_pos_query_par(csg)));
    */


    group.bench_with_input(
        BenchmarkId::new("build dag 64 from csg union_i pos query", "100 x sphere 10"), 
        &union, 
        |b, union| 
        b.iter(|| build_from_pos_query(union)));

    group.bench_with_input(
        BenchmarkId::new("parallel build dag 64 from csg union_ pos query", "100 x sphere 10"), 
        &union, 
        |b, union| 
        b.iter(|| build_from_pos_query_par(union))); 


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

