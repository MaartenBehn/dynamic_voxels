use std::iter;

use feistel_permutation_rs::{DefaultBuildHasher, Permutation};
use octa_force::glam::{self, ivec3, vec4, Mat4, Vec3, Vec3A, Vec4, Vec4Swizzles};

use super::math::to_3d_ivec3;


#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Vec4,
    pub max: Vec4,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        AABB {
            min: vec4(min.x, min.y, min.z, 1.0),
            max: vec4(max.x, max.y, max.z, 1.0)
        }
    }

    pub fn new_a(min: Vec3A, max: Vec3A) -> AABB {
        AABB {
            min: vec4(min.x, min.y, min.z, 1.0),
            max: vec4(max.x, max.y, max.z, 1.0)
        }
    }

    pub fn from_box(mat: &Mat4) -> AABB {
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

        let mut min = vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
        let mut max = vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
        for corner in corners {
            let transformed_corner = mat.mul_vec4(corner);

            min = min.min(transformed_corner);
            max = max.max(transformed_corner);
        }

        AABB {
            min: min,
            max: max,
        }
    }

    pub fn from_sphere(mat: &Mat4) -> AABB {
        let a = vec4(
            f32::sqrt(mat.x_axis.x.powf(2.0) + mat.x_axis.y.powf(2.0) + mat.x_axis.z.powf(2.0)),
            f32::sqrt(mat.y_axis.x.powf(2.0) + mat.y_axis.y.powf(2.0) + mat.y_axis.z.powf(2.0)),
            f32::sqrt(mat.z_axis.x.powf(2.0) + mat.z_axis.y.powf(2.0) + mat.z_axis.z.powf(2.0)),
            0.0
        );
        let b = vec4(mat.w_axis.x, mat.w_axis.y, mat.w_axis.z, 1.0);

        AABB {
            min: b - a,
            max: b + a,
        }
    }

    pub fn from_centered_size(mat: &Mat4, size: Vec3) -> AABB {
        let size = vec4(size.x as f32, size.y as f32, size.z as f32, 1.0);

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

        let mut min = vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
        let mut max = vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
        for corner in corners {
            let transformed_corner = mat.mul_vec4(size * corner);

            min = min.min(transformed_corner);
            max = max.max(transformed_corner);
        }

        AABB {
            min: min,
            max: max,
        }
    }

    pub fn from_size(mat: &Mat4, size: Vec3) -> AABB {
        let corners = [
            vec4(0.0, 0.0, 0.0, 1.0),
            vec4(0.0, 0.0, size.z, 1.0),
            vec4(0.0, size.y, 0.0, 1.0),
            vec4(0.0, size.y, size.z, 1.0),
            vec4(size.x, 0.0, 0.0, 1.0),
            vec4(size.x, 0.0, size.z, 1.0),
            vec4(size.x, size.y, 0.0, 1.0),
            vec4(size.x, size.y, size.z, 1.0),
        ];

        let mut min = vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0);
        let mut max = vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0);
        for corner in corners {
            let transformed_corner = mat.mul_vec4(corner);

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

    pub fn pos_in_aabb(self, pos: Vec4) -> bool {
        self.min.x <= pos.x && pos.x <= self.max.x
     && self.min.y <= pos.y && pos.y <= self.max.y
     && self.min.z <= pos.z && pos.z <= self.max.z
    }

    pub fn collides_aabb(self, other: AABB) -> bool {
        self.min.x <= other.max.x && other.min.x <= self.max.x
     && self.min.y <= other.max.y && other.min.y <= self.max.y
     && self.min.z <= other.max.z && other.min.z <= self.max.z
    }

    pub fn in_aabb(self, other: AABB) -> bool {
        self.min.x <= other.min.x && other.max.x <= self.max.x
     && self.min.y <= other.min.y && other.max.y <= self.max.y
     && self.min.z <= other.min.z && other.max.z <= self.max.z
    }

    pub fn in_unit_sphere(self) -> bool {
        self.min.length_squared() <= 1.0 && self.min.length_squared() <= 1.0 
    }

    pub fn collides_unit_sphere(self) -> (bool, bool) {

        /*
        let mut dmax = 0.0;
        let mut dmin = 0.0;
        for i in 0..3 {
            let a = self.min[i].powi(2);
            let b = self.max[i].powi(2);
            dmax += a.max(b);
            if( 0.0 < self.min[i] ) { dmin += a; } else 
            if( 0.0 > self.max[i] ) { dmin += b; }
        }
        */

        let a = self.min * self.min;
        let b = self.max * self.max;
        let dmax = a.max(b).element_sum();
        let dmin = (Vec4::from(Vec4::ZERO.cmplt(self.min)) * a 
        + Vec4::from(Vec4::ZERO.cmpgt(self.max)) * b).element_sum();

        let min = dmin <= 1.0;
        let max = dmax <= 1.0;

        (min, max)
    }

    pub fn get_sampled_positions(self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let min = (self.min / step).xyz().as_ivec3();
        let max = (self.max / step).xyz().as_ivec3();

        (min.x..=max.x)
            .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
            .flat_map(move |(x, y)| iter::repeat((x, y)).zip(min.z..=max.z))
            .map(move |((x, y), z)| ivec3(x, y, z).as_vec3() * step)
    }

    pub fn get_random_sampled_positions(self, step: f32) -> impl IntoIterator<Item = Vec3> {
        let min = (self.min / step).xyz().as_ivec3();
        let max = (self.max / step).xyz().as_ivec3();
        let size = max - min;
        let n = size.element_product();

        let seed = fastrand::u64(0..1000);
        let perm = Permutation::new(n as _, seed, DefaultBuildHasher::new());

        perm.into_iter()
            .map(move |i| (to_3d_ivec3(i as usize, size) + min).as_vec3() * step)
    }

    pub fn size(&self) -> Vec4 {
        self.max - self.min
    }

    pub fn mul_mat(&self, mat: Mat4) -> AABB {
        AABB {
            min: mat.mul_vec4(self.min),
            max: mat.mul_vec4(self.max),
        }
    }

    pub fn infinte() -> AABB {
        AABB {
            min: vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0),
            max: vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0),
        }
    }
}

impl Default for AABB {
    fn default() -> Self {
        AABB {
            min: vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0),
            max: vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0),
        }
    }
}

impl Into<bvh::aabb::Aabb<f32, 3>> for AABB {
    fn into(self) -> bvh::aabb::Aabb<f32, 3> {
        bvh::aabb::Aabb::with_bounds(
            nalgebra::Point3::new(self.min.x, self.min.y, self.min.z),
            nalgebra::Point3::new(self.max.x, self.max.y, self.max.z))
    }
}

impl From<bvh::aabb::Aabb<f32, 3>> for AABB {
    fn from(value: bvh::aabb::Aabb<f32, 3>) -> Self {
        AABB {
            min: vec4(value.min.x, value.min.y, value.min.z, 1.0),
            max: vec4(value.max.x, value.max.y, value.max.z, 1.0),
        }
    }
}

impl From<&bvh::aabb::Aabb<f32, 3>> for AABB {
    fn from(value: &bvh::aabb::Aabb<f32, 3>) -> Self {
        AABB {
            min: vec4(value.min.x, value.min.y, value.min.z, 1.0),
            max: vec4(value.max.x, value.max.y, value.max.z, 1.0),
        }
    }
}
