use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{UVec3, Vec3};
use reload::{csg::{csg_tree::tree::CSGTree, fast_query_csg_tree::tree::FastQueryCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, volume::{magica_voxel::MagicaVoxelModel, VolumeQureyAABB, VolumeQureyAABBI, VolumeQureyPosValueI}, voxel::{dag64::VoxelDAG64, grid::{shared::SharedVoxelGrid, VoxelGrid}, renderer::palette::Palette}};

fn build_from_pos_query_par<M: VolumeQureyPosValueI + Sync + Send>(model: &M) -> VoxelDAG64 {
    let dag = VoxelDAG64::new(1000000, 1000000);
    let mut dag = dag.run_worker(10);
    dag.add_pos_query_volume(model).unwrap();
    dag.stop()
}

fn pos_query_par(c: &mut Criterion) {
    let mut group = c.benchmark_group("Sample 10");
    group.sample_size(10);

    let mut palette = Palette::new();
    let tree_model = MagicaVoxelModel::new("./assets/Fall_Tree.vox").unwrap();
    let tree_grid: SharedVoxelGrid = tree_model.into_grid(&mut palette).unwrap().into();

    group.bench_with_input(
        BenchmarkId::new("parallel build dag 64 from grid pos query", "Fall_Tree"), 
        &tree_grid, 
        |b, tree_grid | 
        b.iter(|| build_from_pos_query_par(tree_grid)));

}

criterion_group!(benches, pos_query_par);
criterion_main!(benches);

