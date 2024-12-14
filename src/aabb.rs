use std::iter;

use gcd::Gcd;
use octa_force::glam::{ivec3, vec3, vec4, Mat4, Vec3, Vec4Swizzles};

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

    // https://www.geeksforgeeks.org/check-number-divisible-prime-divisors-another-number/
    fn is_divisable_by_prime_factors(x: u32, y: u32) -> bool {
        if y == 1 {
            return true;
        }

        let z = x.gcd(y);

        if z == 1 {
            return false;
        }

        Self::is_divisable_by_prime_factors(x, y / 2)
    }

    pub fn find_a(m: u32) -> u32 {
        let mut a_s = vec![];
        for i in 0..m {
            if Self::is_divisable_by_prime_factors(i, m) && (m % 4 == 0) == (i % 4 == 0) {
                a_s.push(i + 1);
            }
        }

        dbg!(&a_s);

        *a_s.last().unwrap()
    }

    pub fn get_random_sampled_positions(self, step: f32) -> impl IntoIterator<Item = Vec3> {
        /*
        let min = (self.min / step).as_ivec3();
        let max = (self.max / step).as_ivec3();
        let size = max - min;
        let m = size.element_product() as u32;


                // https://en.wikipedia.org/wiki/Linear_congruential_generator
                let a = Self::find_a(m);
                let c = 1;
                let mut x = 0;

                let iter = (0..m).scan(x, |x, _| {
                    *x = (a * *x + c) % m;
                    Some(*x)
                });

                for i in iter {
                    dbg!(i);
                }
        */

        let mut positions: Vec<_> = self.get_sampled_positions(step).into_iter().collect();

        fastrand::shuffle(&mut positions);

        positions
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
