use std::{hint::black_box, marker::PhantomData};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{IVec3, UVec3, Vec3, Vec3A};
use reload::{csg::{csg_tree::tree::CSGTree, sphere::CSGSphere}, util::{number::Nu, vector::Ve}, volume::{VolumeQureyAABB, VolumeQureyPosValue}, voxel::{dag64::{VoxelDAG64, lod_heuristic::LODHeuristicNone}, grid::VoxelGrid}};

fn build_from_pos_query<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_pos_query_volume(model, &LODHeuristicNone {}).unwrap();
    dag
}

fn build_from_aabb_query<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3>>(model: &M) -> VoxelDAG64 {
    let mut dag = VoxelDAG64::new(100000, 64);
    dag.add_aabb_query_volume(model, &LODHeuristicNone {}).unwrap();
    dag
}

fn build_from_pos_query_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3> + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(1000000, 64);
    let mut dag = dag.parallel();
    dag.add_pos_query_volume(model, &LODHeuristicNone {}).unwrap();
    dag.single()
}

fn build_from_aabb_query_par<V: Ve<T, 3>, T: Nu, M: VolumeQureyAABB<V, T, 3> + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(1000000, 64);
    let mut dag = dag.parallel();
    dag.add_aabb_query_volume(model, &LODHeuristicNone {}).unwrap();
    dag.single()
}

fn build_tree64_from_pos_query<V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>>(model: &M) -> tree64::Tree64<u8> {
    tree64::Tree64::new(Tree64Model{
        model,
        p0: PhantomData,
        p1: PhantomData,
    })
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sphere 100");
    group.sample_size(100);

    let mut grid = VoxelGrid::empty(UVec3::ONE * 4_u32.pow(4)); 
    grid.set_example_sphere();

    group.bench_with_input(
        BenchmarkId::new("tree 64 from grid", "pos int"), 
        &grid, 
        |b, grid| b.iter(|| build_tree64_from_pos_query::<UVec3, _, _>(black_box(grid))));
    

    let sphere = CSGSphere::<u8, IVec3, i32, 3>::new_sphere(Vec3A::ZERO, 100.0, 1);

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


    let sphere = CSGSphere::<u8, Vec3A, f32, 3>::new_sphere(Vec3A::ZERO, 100.0, 1);

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
 


    let mut csg = CSGTree::<u8, IVec3, i32, 3>::new_sphere_float(Vec3A::ZERO, 100.0, 1);

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


    let mut csg = CSGTree::<u8, Vec3A, f32, 3>::new_sphere(Vec3A::ZERO, 100.0, 1);

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


struct Tree64Model<'a, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>>{
    model: &'a M,
    p0: PhantomData<V>,
    p1: PhantomData<T>,
}

impl<'a, V: Ve<T, 3>, T: Nu, M: VolumeQureyPosValue<V, T, 3>> tree64::VoxelModel<u8> for Tree64Model<'a, V, T, M> {
    fn dimensions(&self) -> [u32; 3] {
       self.model.get_size().to_uvec3().into() 
    }

    fn access(&self, coord: [usize; 3]) -> Option<u8> {
        let pos = UVec3::new(coord[0] as u32, coord[1] as u32, coord[2] as u32);
        if pos.cmpge(self.model.get_size().to_uvec3()).any() {
            return None;
        }

        let data = self.model.get_value(V::from_uvec3(pos));
        if (data == 0) {
            None
        } else {
            Some(data)
        }
    }
}
