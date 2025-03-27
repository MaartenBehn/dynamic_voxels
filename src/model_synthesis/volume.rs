use std::u64;

use fast_poisson::Poisson;
use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use kiddo::KdTree;
use octa_force::glam::{vec3, vec4, Mat4, Vec3, Vec4Swizzles};

use crate::{vec_csg_tree::{self, tree::{VecCSGNode, VecCSGNodeData, VecCSGTree}}, util::to_3d, volume::Volume};


#[derive(Debug, Clone)]
pub struct PossibleVolume {
    pub points: Vec<Vec3>,
    pub kd_tree: KdTree<f32, 3>,
}

impl PossibleVolume {
    pub fn new(base_volume: VecCSGNode, point_distance: f32) -> Self {

        let mut poisson = Poisson::<3, VecCSGTree>::new()
            .with_radius(point_distance);

        let csg_tree = VecCSGTree::from_node(base_volume);
        poisson.set_validate(|p, csg_tree| { csg_tree.is_position_valid_vec3(Vec3::from_array(p)) }, csg_tree);

        let (kd_tree, points) = poisson.iter()
            .enumerate()
            .fold((KdTree::new(), vec![]), |(mut tree, mut points), (i, p)| {
                tree.add(&p, i as u64);
                points.push(Vec3::from_array(p));
                (tree, points)
            });

        PossibleVolume {
            points,
            kd_tree,
        }
    }
}

