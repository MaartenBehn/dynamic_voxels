use std::u64;

use fast_poisson::{Poisson};
use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use kiddo::KdTree;
use octa_force::glam::{vec3, vec4, Mat4, Vec3, Vec4Swizzles};

use crate::{csg_tree::{self, tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE}}, util::to_3d};

use super::builder::Identifier;

#[derive(Debug, Clone)]
pub struct PossibleVolume {
    pub kd_tree: KdTree<f32, 3>,
}

impl PossibleVolume {
    pub fn new(base_volume: CSGNode, point_distance: f32) -> Self {

        let mut poisson = Poisson::<3, CSGTree>::new()
            .with_radius(point_distance);

        let csg_tree = CSGTree::from_node(base_volume);
        poisson.set_validate(|p, csg_tree| { csg_tree.at_pos(Vec3::from_array(p)) }, csg_tree);
 
        let kd_tree = poisson.generate_kd_tree();

        //dbg!(&kd_tree.iter().collect::<Vec<_>>());
        
        PossibleVolume {
            kd_tree,
        }
    }

    pub fn get_pos(&self) -> Option<Vec3> {
        match self.kd_tree.iter().take().next() {
            Some((_, p)) => Some(vec3(p[0], p[1], p[2])),
            None => None,
        }
    }
}

