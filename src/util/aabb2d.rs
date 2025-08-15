use std::iter;

use octa_force::glam::{vec2, Affine2, Vec2};

use super::math::to_3d_ivec3;


#[derive(Copy, Clone, Debug)]
pub struct AABB2 {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB2 {
    pub fn new(min: Vec2, max: Vec2) -> AABB2 { AABB2 { min, max } }

    pub fn get_sampled_positions(self, step: f32) -> impl Iterator<Item = Vec2> {
        let min = (self.min / step).as_ivec2();
        let max = (self.max / step).as_ivec2();

        (min.x..=max.x)
            .flat_map(move |x| iter::repeat(x).zip(min.y..=max.y))
            .map(move |(x, y)| vec2(x as f32, y as f32) * step)
    }

    pub fn pos_in_aabb(self, pos: Vec2) -> bool {
        self.min.x <= pos.x && pos.x <= self.max.x
     && self.min.y <= pos.y && pos.y <= self.max.y
    }

    pub fn mul_affine(&self, affine: &Affine2) -> AABB2 {
        AABB2 {
            min: affine.transform_point2(self.min),
            max: affine.transform_point2(self.max),
        }
    }

    pub fn collides_aabb(self, other: AABB2) -> bool {
        self.min.x <= other.max.x && other.min.x <= self.max.x
     && self.min.y <= other.max.y && other.min.y <= self.max.y
    }

    pub fn contains_aabb(self, other: AABB2) -> bool {
        self.min.x <= other.min.x && other.max.x <= self.max.x
     && self.min.y <= other.min.y && other.max.y <= self.max.y
    }

    pub fn collides_unit_sphere(self) -> (bool, bool) {

        let a = self.min * self.min;
        let b = self.max * self.max;
        let dmax = a.max(b).element_sum();
        let dmin = 
            (Vec2::from(Vec2::ZERO.cmplt(self.min)) * a 
           + Vec2::from(Vec2::ZERO.cmpgt(self.max)) * b).element_sum();

        let min = dmin <= 1.0;
        let max = dmax <= 1.0;

        (min, max)
    }
    
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    pub fn from_circle(mat: &Affine2) -> AABB2 {
        let a = vec2(
            f32::sqrt(mat.x_axis.x.powf(2.0) + mat.x_axis.y.powf(2.0)),
            f32::sqrt(mat.y_axis.x.powf(2.0) + mat.y_axis.y.powf(2.0)),
        );
        let b = vec2(mat.z_axis.x, mat.z_axis.y);

        AABB2 {
            min: b - a,
            max: b + a,
        }
    }

    pub fn from_box(mat: &Affine2) -> AABB2 {
        let corners = [
            vec2(-0.5, -0.5),
            vec2(-0.5, 0.5),
            vec2(0.5, -0.5),
            vec2(0.5, 0.5),
        ];

        let mut min = vec2(f32::INFINITY, f32::INFINITY);
        let mut max = vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for corner in corners {
            let transformed_corner = mat.transform_point2(corner);

            min = min.min(transformed_corner);
            max = max.max(transformed_corner);
        }

        AABB2 {
            min: min,
            max: max,
        }
    }

    pub fn infinte() -> AABB2 {
        AABB2 {
            min: vec2(f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: vec2(f32::INFINITY, f32::INFINITY),
        }
    }

    pub fn union(self, other: AABB2) -> AABB2 {
        AABB2 {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }

    pub fn intersect(self, other: AABB2) -> AABB2 {
        AABB2 {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }
}

impl Default for AABB2 {
    fn default() -> Self {
        AABB2 {
            min: vec2(f32::INFINITY, f32::INFINITY),
            max: vec2(f32::NEG_INFINITY, f32::NEG_INFINITY),
        }
    }
}
