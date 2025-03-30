use std::iter;

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::{ivec3, vec3, vec4, Mat4, Vec3, Vec4Swizzles};

use crate::{util::to_3d_i, voxel::grid::VoxelGrid};

#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn from_box(mat: &Mat4, padding: f32) -> AABB {
        let corners = [
            vec4(-0.5, -0.5, -0.5, 1.0),
            vec4(-0.5, -0.5, 0.5, 1.0),
            vec4(-0.5, 0.5, -0.5, 1.0),
            vec4(-0.5, 0.5, 0.5, 1.0),
            vec4(0.5, -0.5, -0.5, 1.0),
            vec4(0.5, -0.5, 0.5, 1.0),
            vec4(0.5, 0.5, -0.5, 1.0),
            vec4(0.5, 0.5, 0.5, 1.0),
        ];

        let mut min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for corner in corners {
            let transformed_corner = mat.mul_vec4(corner).xyz();

            min = min.min(transformed_corner);
            max = max.max(transformed_corner);
        }

        

        AABB {
            min: min - padding,
            max: max + padding,
        }
    }

    pub fn from_sphere(mat: &Mat4, padding: f32) -> AABB {
        let a = vec3(
            f32::sqrt(mat.x_axis.x.powf(2.0) + mat.x_axis.y.powf(2.0) + mat.x_axis.z.powf(2.0)),
            f32::sqrt(mat.y_axis.x.powf(2.0) + mat.y_axis.y.powf(2.0) + mat.y_axis.z.powf(2.0)),
            f32::sqrt(mat.z_axis.x.powf(2.0) + mat.z_axis.y.powf(2.0) + mat.z_axis.z.powf(2.0)),
        );
        let b = vec3(mat.w_axis.x, mat.w_axis.y, mat.w_axis.z);

        AABB {
            min: b - a - padding,
            max: b + a + padding,
        }
    }

    pub fn from_voxel_gird(mat: &Mat4, grid: &VoxelGrid) -> AABB {

        let size = vec4(grid.size.x as f32, grid.size.y as f32, grid.size.z as f32, 1.0);

        let corners = [
            vec4(-0.5, -0.5, -0.5, 1.0),
            vec4(-0.5, -0.5, 0.5, 1.0),
            vec4(-0.5, 0.5, -0.5, 1.0),
            vec4(-0.5, 0.5, 0.5, 1.0),
            vec4(0.5, -0.5, -0.5, 1.0),
            vec4(0.5, -0.5, 0.5, 1.0),
            vec4(0.5, 0.5, -0.5, 1.0),
            vec4(0.5, 0.5, 0.5, 1.0),
        ];

        let mut min = vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for corner in corners {
            let transformed_corner = mat.mul_vec4(size * corner).xyz();

            min = min.min(transformed_corner);
            max = max.max(transformed_corner);
        }

        AABB {
            min: min,
            max: max,
        }
    }

    pub fn union(self, other: AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn intersect(self, other: AABB) -> AABB {
        AABB {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }

    pub fn pos_in_aabb(self, pos: Vec3) -> bool {
        self.min.x <= pos.x
            && pos.x <= self.max.x
            && self.min.y <= pos.y
            && pos.y <= self.max.y
            && self.min.z <= pos.z
            && pos.z <= self.max.z
    }

    pub fn get_sampled_positions(self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let min = (self.min / step).as_ivec3();
        let max = (self.max / step).as_ivec3();
        (min.x..=max.x)
            .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
            .flat_map(move |(x, y)| iter::repeat((x, y)).zip(min.z..=max.z))
            .map(move |((x, y), z)| ivec3(x, y, z).as_vec3() * step)
    }

    pub fn get_random_sampled_positions(self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let min = (self.min / step).as_ivec3();
        let max = (self.max / step).as_ivec3();
        let size = max - min;
        let n = size.element_product();

        let seed = fastrand::u64(0..1000);
        let perm = Permutation::new(n as _, seed, DefaultBuildHasher::new());

        perm.into_iter()
            .map(move |i| (to_3d_i(i as usize, size) + min).as_vec3() * step)
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min: vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }
}
