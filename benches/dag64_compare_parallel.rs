use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use itertools::Itertools;
use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A, vec3, vec3a};
use reload::{csg::csg_tree::tree::CSGTree, util::vector::Ve, volume::{VolumeBounds, VolumeQureyAABB, VolumeQureyPosValue}, voxel::dag64::{VoxelDAG64, lod_heuristic::LODHeuristicNone, parallel::ParallelVoxelDAG64}};


fn build_par<V: Ve<i32, 3>, M: VolumeQureyAABB<V, i32, 3> + Sync + Send>(model: &M) -> ParallelVoxelDAG64 {
    let mut dag = ParallelVoxelDAG64::new(100000, 20000);
    dag.add_aabb_query_volume(model, &LODHeuristicNone {}).unwrap();
    dag
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parallel test");
    group.sample_size(10);

    let mut csg = CSGTree::<u8, IVec3, i32, 3>::new_sphere_float(Vec3A::ZERO, 10.0, 1);

    for _ in 0..500 {
        let pos = vec3a(fastrand::f32(), fastrand::f32(), fastrand::f32()) * 1000.0;
        csg.union_sphere(pos, 10.0, 1);
    }
    csg.calculate_bounds();

    group.bench_with_input(
        BenchmarkId::new("dag 64 from csg", "1000 x sphere 10 pos par"), 
        &csg, 
        |b, csg| 
        b.iter(|| build_par(csg)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

