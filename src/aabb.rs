use octa_force::glam::{vec3, vec4, Mat4, Vec3, Vec4Swizzles};


#[derive(Copy, Clone, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn from_box(mat: &Mat4, padding: f32) -> AABB {
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
            f32::sqrt(mat.z_axis.x.powf(2.0) + mat.z_axis.y.powf(2.0) + mat.z_axis.z.powf(2.0)));
        let b = vec3(mat.w_axis.x, mat.w_axis.y, mat.w_axis.z);

        AABB {
            min: b - a - padding,
            max: b + a + padding,
        }
    }

    pub fn merge(self, other: AABB) -> AABB {
        AABB {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
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