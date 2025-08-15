use std::{iter, marker::PhantomData};

use itertools::Either;
use octa_force::glam::{ivec2, ivec3, vec2, vec3a, vec4};

use super::{math_config::MC, matrix::Ma, number::Nu, vector::Ve};

#[derive(Clone, Copy, Debug)]
pub struct AABB<C: MC<D>, const D: usize> {
    min: C::Vector,
    max: C::Vector,
}

impl<C: MC<D>, const D: usize> AABB<C, D>  {
    pub fn new(min: C::Vector, max: C::Vector) -> Self {
        Self { min, max }
    }

    pub fn min(&self) -> C::Vector {
        self.min
    }

    pub fn max(&self) -> C::Vector {
        self.max
    }

    pub fn size(&self) -> C::Vector {
        self.max - self.min
    }

    pub fn center(&self) -> C::Vector {
        (self.max + self.min) / C::Number::TWO
    }

    pub fn infinte() -> Self {
        Self {
            min: C::Vector::MIN,
            max: C::Vector::MAX,
        }
    }
    
    pub fn pos_in_aabb(&self, pos: C::Vector) -> bool {
        (0..D).all(|i| {
            self.min[i] <= pos[i] && pos[i] <= self.max[i]
        })
    }

    pub fn collides_aabb(&self, other: AABB<C, D>) -> bool {
        (0..D).all(|i| {
            self.min[i] <= other.max[i] && other.min[i] <= self.max[i]
            // self.min.x <= other.max.x && other.min.x <= self.max.x
        })
    }

    pub fn contains_aabb(&self, other: AABB<C, D>) -> bool {
        (0..D).all(|i| {
            self.min[i] <= other.min[i] && other.max[i] <= self.max[i]
            // self.min.x <= other.max.x && other.min.x <= self.max.x
        })
    }

    pub fn from_box(mat: &C::Matrix) -> Self {
        match D {
            2 => {
                let corners = [
                    vec2(-0.5, -0.5),
                    vec2(-0.5,  0.5),
                    vec2( 0.5, -0.5),
                    vec2( 0.5,  0.5),
                ];

                let mut min = C::Vector::from_vec2(vec2(f32::INFINITY, f32::INFINITY));
                let mut max = C::Vector::from_vec2(vec2(f32::NEG_INFINITY, f32::NEG_INFINITY));

                for corner in corners {
                    let transformed_corner = mat.mul_vector(C::Vector::from_vec2(corner));

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                AABB {
                    min,
                    max,
                }
            }
            3 => {
                let corners = [
                    vec4(-0.5, -0.5, -0.5, 1.0),
                    vec4(-0.5, -0.5,  0.5, 1.0),
                    vec4(-0.5,  0.5, -0.5, 1.0),
                    vec4(-0.5,  0.5,  0.5, 1.0),
                    vec4( 0.5, -0.5, -0.5, 1.0),
                    vec4( 0.5, -0.5,  0.5, 1.0),
                    vec4( 0.5,  0.5, -0.5, 1.0),
                    vec4( 0.5,  0.5,  0.5, 1.0),
                ];

                let mut min = C::Vector::from_vec4h(vec4(f32::INFINITY, f32::INFINITY, f32::INFINITY, 1.0));
                let mut max = C::Vector::from_vec4h(vec4(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY, 1.0));

                for corner in corners {
                    let transformed_corner = mat.mul_vector(C::Vector::from_vec4h(corner));

                    min = min.min(transformed_corner);
                    max = max.max(transformed_corner);
                }

                AABB {
                    min,
                    max,
                }
            }
            _ => unreachable!()
        }
    }

    pub fn from_sphere(mat: &C::Matrix) -> Self {
        let a = C::VectorF::from_iter((0..D).map(|i| {
            f32::sqrt((0..D).map(|j| {
                mat.index(i, j).powf(2.0)
            }).sum())
        }));

        let b = C::VectorF::from_iter((0..D).map(|i| mat.index(3, i)));

        Self::new(C::to_vector(b - a), C::to_vector(b + a))
    }

    pub fn collides_unit_sphere(self) -> (bool, bool) {

        let a = self.min * self.min;
        let b = self.max * self.max;
        let dmax = a.max(b).element_sum();
        let dmin = 
            (C::Vector::ZERO.lt(self.min) * a 
           + C::Vector::ZERO.gt(self.max) * b).element_sum();

        let min = dmin <= C::Number::ONE;
        let max = dmax <= C::Number::ONE;

        (min, max)
    }

    pub fn get_sampled_positions(self, step: C::Number) -> impl Iterator<Item = C::Vector> {
        let min = (self.min / step);
        let max = (self.max / step);

        match D {
            2 => {
                let min = min.to_ivec2();
                let max = max.to_ivec2();

                Either::Left((min.x..=max.y)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .map(move |(x, y)| C::Vector::from_ivec2(ivec2(x, y)) * step))
            }
            3 => {
                let min = min.to_ivec3();
                let max = max.to_ivec3();

                Either::Right((min.x..=max.y)
                    .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
                    .flat_map(move |(x, y)| iter::repeat((x, y)).zip(min.z..=max.z))
                    .map(move |((x, y), z)| C::Vector::from_ivec3(ivec3(x, y, z)) * step))
            }
            _ => unreachable!()
        }
    }

    pub fn mul_mat(&self, mat: &C::Matrix) -> Self {
        AABB {
            min: mat.mul_vector(self.min),
            max: mat.mul_vector(self.max),
        }
    }

}

impl<C: MC<D>, const D: usize> Default for AABB<C, D> {
    fn default() -> Self {
        Self { min: C::Vector::MAX, max: C::Vector::MIN }
    }
}

