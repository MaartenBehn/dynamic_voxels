use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A, vec3, vec3a};
use reload::{csg::{csg_tree::tree::CSGTree, union::tree::{Union, UnionNode}}, util::{vector::Ve}, volume::{VolumeBounds, VolumeQureyPosValue}, voxel::dag64::VoxelDAG64};


fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Bvh build");
    group.sample_size(100);

    let mut csg = CSGTree::<u8, IVec3, i32, 3>::new_sphere(IVec3::ZERO, 10, 1);
    let mut union = Union::<u8, IVec3, i32, 3>::new();
    union.add_node(UnionNode::new_sphere(Vec3A::ZERO, 10.0, 1));

    for _ in 0..100000 {
        let pos = vec3a(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 1000.0;
        csg.union_sphere(pos, 10.0, 1);
        union.add_node(UnionNode::new_sphere(pos, 10.0, 1));
    }

    group.bench_with_input(
        BenchmarkId::new("Union", "100000 x sphere 10"), 
        &union, 
        |b, union| {
            let mut union = union.to_owned();
            b.iter(|| black_box({
                union.changed = true;
                union.calculate_bounds();
            }));
        });
        

    group.bench_with_input(
        BenchmarkId::new("csg", "100000 x sphere 10"), 
        &csg, 
        |b, csg| {
            let mut csg = csg.to_owned();
            b.iter(|| black_box({
                csg.changed = true;
                csg.calculate_bounds();
            }));
        });

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

