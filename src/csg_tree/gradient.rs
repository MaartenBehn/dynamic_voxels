use octa_force::glam::{vec3, vec4, Vec3, Vec4, Vec4Swizzles};

use super::tree::{CSGNodeData, CSGTree};


impl CSGTree {
    pub fn get_gradient_at_pos(&self, pos: Vec3) -> Vec3 {
        self.get_gradient_at_pos_internal(vec4(pos.x, pos.y, pos.z, 1.0), 0)
    } 

    fn get_gradient_at_pos_internal(&self, pos: Vec4, i: usize) -> Vec3 {
        match self.nodes[i].data {
            CSGNodeData::Union(i1, i2) => {
                let g1 = self.get_gradient_at_pos_internal(pos, i1);
                let g2 = self.get_gradient_at_pos_internal(pos, i2);

                g1 + g2
            },
            CSGNodeData::Remove(i1, i2) => {
                let g1 = self.get_gradient_at_pos_internal(pos, i1);
                let g2 = self.get_gradient_at_pos_internal(pos, i2);

                g1 - g2
            },
            CSGNodeData::Intersect(i1, i2) => {
                let g1 = self.get_gradient_at_pos_internal(pos, i1);
                let g2 = self.get_gradient_at_pos_internal(pos, i2);

                g1 * g2
            },
            CSGNodeData::Box(mat, _) => {
                let t_point = mat.inverse().mul_vec4(pos);

                get_gradient_of_unit_box(t_point.xyz())
            },
            CSGNodeData::Sphere(mat, _) => {
                let t_point = mat.inverse().mul_vec4(pos);

                get_gradient_of_unit_box(t_point.xyz())
            },
            CSGNodeData::VoxelGrid(..)
            | CSGNodeData::All(_) => vec3(0.0, 0.0, 0.0),
        }
    }
}

// Just the pos it self
pub fn get_gradient_of_uint_sphere(to_pos: Vec3) -> Vec3 {
    to_pos
}

/**
*          |
*  x---------------x
*  |       |       |
*  |       q --------> p
*  |       |       |
*  |       x       |
*  |       |       |
*  |       |       |
*  |       |       |
*  x---------------x
*          |
*
* From: https://github.com/MaartenBehn/distance3d/blob/master/distance3d/distance/_plane.py
*    t = np.dot(plane_normal, point - plane_point)
*    closest_point_plane = point - t * plane_normal
**/
pub fn get_gradient_of_unit_box(to_pos: Vec3) -> Vec3 {
    let normal = to_pos.signum();

    let t = normal.dot(to_pos);
    // let q = to_pos - t * normal;
    // let v = q - to_pos;
    
    -t * normal
}
