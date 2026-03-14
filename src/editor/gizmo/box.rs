use itertools::Itertools;
use octa_force::glam::{Mat3, Mat4, Vec2, Vec3, vec3a, vec4};

use crate::{csg::csg_tree::tree::CSGTree, util::{matrix::Ma as _, vector::{CastInto, Ve}}, voxel::palette::palette::MATERIAL_ID_BASE};

pub struct BoxGizmo<V: Ve<f32, D>, const D: usize> {
    mat: V::Matrix,
    csg: CSGTree<u8, V, f32, D>,
}

impl<V: Ve<f32, D>, const D: usize> BoxGizmo<V, D> {
    pub fn new(mat: V::Matrix) -> Self {
        let csg = Self::make_csg(mat);

        Self { 
            mat,
            csg
        }
    }

    fn make_csg(mat: V::Matrix) -> CSGTree<u8, V, f32, D> {
        match D {
            2 => {
                let min = Vec2::splat(-0.5);
                let max = Vec2::splat(0.5);

                Self::make_csg_inner([
                    vec3a(min[0],min[1], 1.0),
                    vec3a(min[0],max[1], 1.0),
                    vec3a(max[0],min[1], 1.0),
                    vec3a(max[0],max[1], 1.0),
                ].map(|p| {
                        let mat: Mat3 = mat.cast_into();
                        V::ve_from(mat.mul_vec3a(p))
                    }), [(0, 1), (2, 3), (0, 2), (1, 3)])
            }
            3 => {
                let min = Vec3::splat(-0.5);
                let max = Vec3::splat(0.5);

                Self::make_csg_inner([
                    vec4(min[0],min[1], min[2], 1.0),
                    vec4(min[0],min[1], max[2], 1.0),
                    vec4(min[0],max[1], min[2], 1.0),
                    vec4(min[0],max[1], max[2], 1.0),
                    vec4(max[0],min[1], min[2], 1.0),
                    vec4(max[0],min[1], max[2], 1.0),
                    vec4(max[0],max[1], min[2], 1.0),
                    vec4(max[0],max[1], max[2], 1.0),
                ].map(|p| {
                        let mat: Mat4 = mat.cast_into();
                        V::ve_from(mat.mul_vec4(p))
                    }), [
                        (0, 1), (2, 3), (4, 5), (6, 7), 
                        (0, 2), (1, 3), (4, 6), (5, 7), 
                        (0, 4), (1, 5), (2, 6), (3, 7)
                    ])
            }
            _ => unreachable!()
        }
    }

    fn make_csg_inner<const NC: usize, const NE: usize>(corners: [V; NC], edges: [(usize, usize); NE]) -> CSGTree<u8, V, f32, D> {
        let mut csg = CSGTree::default();

        let keys = corners.map(|v| csg.add_sphere(v.to_vecf(), 10.0, MATERIAL_ID_BASE));

        csg.add_union_node(keys.to_vec());

        csg
    }
}


