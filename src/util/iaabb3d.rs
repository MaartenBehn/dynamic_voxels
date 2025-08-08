use octa_force::glam::IVec3;

use super::aabb3d::AABB;

#[derive(Copy, Clone, Debug)]
pub struct AABBI {
    pub min: IVec3,
    pub max: IVec3,
}

impl AABBI {
    pub fn new(min: IVec3, max: IVec3) -> AABBI {
        AABBI { min, max }
    }

    pub fn size(&self) -> IVec3 {
        self.max - self.min
    }

    pub fn center(&self) -> IVec3 {
        (self.max + self.min) / 2
    }

    pub fn pos_in_aabb(self, pos: IVec3) -> bool {
        self.min.x <= pos.x && pos.x <= self.max.x
     && self.min.y <= pos.y && pos.y <= self.max.y
     && self.min.z <= pos.z && pos.z <= self.max.z
    }

    pub fn collides_aabb(self, other: AABBI) -> bool {
        self.min.x <= other.max.x && other.min.x <= self.max.x
     && self.min.y <= other.max.y && other.min.y <= self.max.y
     && self.min.z <= other.max.z && other.min.z <= self.max.z
    }

    pub fn contains_aabb(self, other: AABBI) -> bool {
        self.min.x <= other.min.x && other.max.x <= self.max.x
     && self.min.y <= other.min.y && other.max.y <= self.max.y
     && self.min.z <= other.min.z && other.max.z <= self.max.z
    }
}

impl Into<AABB> for AABBI {
    fn into(self) -> AABB {
        AABB::new(self.min.as_vec3(), self.max.as_vec3())
    }
}

impl Default for AABBI {
    fn default() -> Self {
        AABBI {
            min: IVec3::MAX,
            max: IVec3::MIN,
        }
    }
}
