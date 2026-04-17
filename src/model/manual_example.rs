use std::{cmp::Ordering, iter, time::Instant};

use itertools::{Itertools, iproduct};
use octa_force::{glam::{Mat4, Quat, Vec2, Vec3, Vec3A, vec2, vec3, vec3a}, log::info};

use crate::{csg::{csg_tree::tree::{CSGTree, CSGTreeNode}, primitves::CSGPrimitive}, scene::worker::SceneWorkerSend, util::{aabb::AABB, shader_constants::VOXELS_PER_SHADER_UNIT, vector::{IntoT as _, Ve as _}}, volume::{VolumeBounds, VolumeQureyPosValid}};


pub fn calc_manual(scene: SceneWorkerSend) {
    let now = Instant::now();

    let mut f: CSGTree<u8, Vec3A, f32, 3> = CSGTree::default();
    let b = CSGTree::new_box(Vec2::ZERO, Vec2::splat(20000.0), ());

    let mut seq = quasi_rd::Sequence::new_with_offset(2, fastrand::u64(0..=u64::MAX)); 

    let mut roots = vec![];
    for p in b.get_grid_positions(500.0) {
        let mut circle: CSGTree<(), Vec2, f32, 2> = CSGTree::new_sphere(p, 200.0, ());

        circle.calculate_bounds();
        let aabb = circle.get_bounds();
        let min: Vec2 = AABB::min(&aabb).ve_into();
        let size: Vec2 = aabb.size().ve_into();

        let mut set = vec![];

        let samples = 20;
        let tries = samples * 2;

        let seq_ref = &mut seq;
        let mut fi = iter::from_fn(move || Some(seq_ref.next_f32()));

        let pos_iter = iter::from_fn(move || {
            let vf = Vec2::from_iter(&mut fi);
            let v = vf * size + min; 
            Some(v)
        })
            .take(tries)
            .filter(|v| circle.is_position_valid(*v))
            .take(samples);

        set.extend(pos_iter);

        let set_2 = set.clone();

        let pairs = iproduct!(set, set_2)
            .filter(move |(a, b)| matches!(a.cmp(*b), Ordering::Less) && (*a - *b).length_squared() < (100.0 * 100.0) );

        let mut cut_roots = vec![];
        for (start, end) in pairs {
            let side_variance = vec2(1.0, 1.0);
            let spacing = 10.0;

            let mut current = start;

            let delta = end - current;
            let steps = (delta.length() / spacing) as usize;
            let tries = steps * 2;

            for p in iter::once(start).chain(
                iter::successors(Some((current)), move |current| {
                    let delta = end - *current;
                    let length = delta.length();

                    if length < spacing {
                        return None;
                    }

                    let r = Vec2::from_iter(&mut iter::from_fn(|| Some(fastrand::f32()))) * 2.0 - 1.0;
                    let side = r * side_variance * length;
                    let dir = (delta + side).normalize();
                    let pos = *current + dir * spacing;

                    Some(pos)
                })
            ) {
                roots.push(
                    f.add_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(vec3a(p.x, p.y, 20.0), 10.0, 1))));

                cut_roots.push(circle.add_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(p, 20.0, ()))));
            }
        }

        let root = circle.add_union_node(cut_roots);
        circle.add_cut_node(circle.root, root);

        circle.calculate_bounds();
        let aabb = circle.get_bounds();
        let min: Vec2 = AABB::min(&aabb).ve_into();
        let size: Vec2 = aabb.size().ve_into();

        let samples = 30;
        let tries = samples * 2;

        let seq_ref = &mut seq;
        let mut fi = iter::from_fn(move || Some(seq_ref.next_f32()));

        let pos_iter = iter::from_fn(move || {
            let vf = Vec2::from_iter(&mut fi);
            let v = vf * size + min; 
            Some(v)
        })
            .take(tries)
            .filter(|v| circle.is_position_valid(*v))
            .take(samples);

        for p in pos_iter {
            roots.push(
                f.add_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(vec3a(p.x, p.y, 40.0), 10.0, 1))));

            roots.push(
                f.add_node(CSGTreeNode::new_box(CSGPrimitive::new_box(vec3a(p.x, p.y, 20.0), vec3a(5.0, 5.0, 40.0), 1))));
        }

        roots.push(
                f.add_node(CSGTreeNode::new_sphere(CSGPrimitive::new_disk(vec3a(p.x, p.y, 0.0), 200.0, 10.0, 1))));
    }

    let root = f.add_union_node(roots);
    f.set_root(root);
    
    let elapsed = now.elapsed();
    info!("Manual 1 took: {:?}", elapsed);
    //f.calculate_bounds();

    /*
    let elapsed = now.elapsed();
    info!("Manual 2 took: {:?}", elapsed);
    */

    /*
    scene.add_object(
        Mat4::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::IDENTITY,
            vec3(4000.0, 0.0, 0.0) / VOXELS_PER_SHADER_UNIT as f32
        ), 
        f,
    ).result_blocking();
    */

    let elapsed = now.elapsed();
    info!("Manual 3 took: {:?}", elapsed);
}
