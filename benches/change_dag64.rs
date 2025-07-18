use std::hint::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use octa_force::glam::{vec3, UVec3, Vec3};
use reload::{csg::{fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::{SlotMapCSGNode, SlotMapCSGTree}, vec_csg_tree::tree::VecCSGTree}, multi_data_buffer::buddy_buffer_allocator::BuddyBufferAllocator, volume::{VolumeBounds, VolumeQureyAABB}, voxel::{dag64::VoxelDAG64, grid::VoxelGrid}};



fn criterion_benchmark(c: &mut Criterion) { 

    c.bench_with_input(
        BenchmarkId::new("change tree 64", "two spheres"), 
        &(), 
        |b, _| 
        b.iter(|| {
            let mut csg = SlotMapCSGTree::<u8>::new_sphere(Vec3::ZERO, 100.0);
            let buffer_size = 2_usize.pow(30);
            let mut allocator = BuddyBufferAllocator::new(buffer_size, 32);
            let mut tree64: VoxelDAG64 = VoxelDAG64::from_aabb_query(&csg, &mut allocator).unwrap();

            let mut transaction = tree64.create_transaction();
            let last_offset = csg.get_offset();
            let index = csg.append_node_with_remove(
                SlotMapCSGNode::new_sphere(vec3(110.0, 0.0, 0.0), 50.0));
            csg.set_all_aabbs();
            let aabb = csg.nodes[index].aabb;

            transaction.update_aabb(&mut tree64, aabb, last_offset, &csg, &mut allocator).unwrap();
        }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

