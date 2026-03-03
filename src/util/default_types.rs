use octa_force::glam::{Vec2, Vec3A};
use crate::{csg::csg_tree::tree::CSGTree, voxel::dag64::lod_heuristic::{LODHeuristicNone, PowHeuristicSphere}};

pub type T = f32;
pub type V2 = Vec2;
pub type V3 = Vec3A;
pub type Volume = CSGTree<u8, V3, T, 3>;
pub type LODType = LODHeuristicNone;

