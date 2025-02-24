use std::u64;

use fast_poisson::{Poisson};
use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use kiddo::KdTree;
use octa_force::glam::{vec4, Mat4, Vec3, Vec4Swizzles};

use crate::{csg_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE}, util::to_3d};

#[derive(Debug, Clone)]
pub struct PossibleVolume {
    pub template_index: usize,
    pub kd_tree: KdTree<f32, 3>,
    pub csg_tree: CSGTree,
}

impl PossibleVolume {
    pub fn new(template_index: usize, base_volume: CSGNode, point_distance: f32) -> Self {

        let mut poisson = Poisson::<3, Mat4>::new()
            .with_radius(point_distance);

        match base_volume.data {
            CSGNodeData::Box(mat, _) => {
                poisson.set_validate(|p, mat| {
                    let q = mat.inverse().mul_vec4(vec4(p[0], p[1] as _, p[2] as _, 1.0)).xyz().to_array();
                    q.iter().all( |&v| v >= -0.5 && v <= 0.5 )
                }, mat);
            },
            CSGNodeData::Sphere(mat, _) => {
                poisson.set_validate(|p, mat| {
                    let q = mat.inverse().mul_vec4(vec4(p[0], p[1], p[2], 1.0)).xyz();
                    q.length_squared() < 1.0
                }, mat);
            },
            _ => panic!("Possible Volume can only be build from a Box or Sphere as Base Volume!")
        }

        let kd_tree = poisson.generate_kd_tree();
        
        let csg_tree = CSGTree {
            nodes: vec![
                CSGNode::new(CSGNodeData::Remove(1, 2)),
                CSGNode::new(CSGNodeData::All(MATERIAL_NONE)),
                base_volume
            ],
        };

        PossibleVolume {
            template_index,
            kd_tree,
            csg_tree
        }
    }
}

