use octa_force::{anyhow::anyhow, glam::{IVec3, Mat3, UVec3, Vec3, Vec3A}, OctaResult};
use dot_vox::*;

use crate::{util::{aabb3d::AABB, math::to_1d}, voxel::{grid::{offset::OffsetVoxelGrid, VoxelGrid}, palette::Palette}};

use super::{VolumeBounds, VolumeQureyPosValue};

pub struct MagicaVoxelModel {
    min: IVec3,
    max: IVec3,
    vox: DotVoxData,
    models_and_positions: Vec<(IVec3, UVec3, Rotation, u32)>, 
}

impl MagicaVoxelModel {
    pub fn new(path: &str) -> OctaResult<Self> {
        let vox = load(path)
            .map_err(|e| anyhow!(e))?;

        let mut stack = vec![(0, IVec3::ZERO, dot_vox::Rotation::IDENTITY)];
        let mut models_and_positions = Vec::new();

        while let Some((index, translation, rotation)) = stack.pop() {
            match &vox.scenes[index as usize] {
                dot_vox::SceneNode::Transform { child, frames, .. } => {
                    // In case of a Transform node, the potential translation and rotation is added
                    // to the global transform to all of the nodes children nodes
                    let translation = if let Some(t) = frames[0].attributes.get("_t") {
                        let translation_delta = t
                            .split(" ")
                            .map(|x| x.parse().expect("Not an integer!"))
                            .collect::<Vec<i32>>();
                        debug_assert_eq!(translation_delta.len(), 3);
                        translation
                        + IVec3::new(
                            translation_delta[0],
                            translation_delta[1],
                            translation_delta[2],
                        )
                    } else {
                        translation
                    };
                    let rotation = if let Some(r) = frames[0].attributes.get("_r") {
                        rotation
                        * dot_vox::Rotation::from_byte(
                            r.parse()
                                .expect("Expected valid u8 byte to parse rotation matrix"),
                        )
                    } else {
                        rotation
                    };

                    stack.push((*child, translation, rotation));
                }
                dot_vox::SceneNode::Group { children, .. } => {
                    for child in children {
                        stack.push((*child, translation, rotation));
                    }
                }
                dot_vox::SceneNode::Shape { models, .. } => {
                    let model_id = models[0].model_id;
                    let model = &vox.models[model_id as usize];

                    let channel_reordering =
                    (Mat3::from_cols_array_2d(&rotation.to_cols_array_2d())
                        * Vec3::new(1.0, 2.0, 3.0))
                        .as_ivec3();

                    let size = swizzle(
                        UVec3::new(model.size.x, model.size.y, model.size.z).as_ivec3(),
                        channel_reordering,
                    )
                        .as_uvec3();

                    models_and_positions.push((translation, size, rotation, model_id));
                }
            }
        }

        let mut min = IVec3::splat(i32::MAX);
        let mut max = IVec3::splat(i32::MIN);

        for &(pos, model_size, ..) in &models_and_positions {
            min = min.min(pos - (model_size / 2).as_ivec3());
            max = max.max(pos + (model_size / 2).as_ivec3());
        }

        Ok(Self {
            min, 
            max, 
            vox,
            models_and_positions
        })
    }
}

impl VolumeBounds for MagicaVoxelModel {
    fn calculate_bounds(&mut self) {}

    fn get_bounds(&self) -> AABB {
        AABB::new(self.min.as_vec3(), (self.max + 1).as_vec3())
    }
}

impl MagicaVoxelModel {
    pub fn into_grid<P: Palette>(self, palette: &mut P) -> OctaResult<OffsetVoxelGrid> { 

        let size = (self.max - self.min + 1).as_uvec3();

        let mut array = vec![0; size.x as usize * size.y as usize * size.z as usize];

        for (pos, model_size, rotation, model_id) in self.models_and_positions {
            let offset = pos - self.min - (model_size / 2).as_ivec3();

            let channel_reordering = (Mat3::from_cols_array_2d(&rotation.to_cols_array_2d())
                * Vec3::new(1.0, 2.0, 3.0))
                .as_ivec3();
            let model_size_swizzled = swizzle(model_size.as_ivec3(), channel_reordering.abs());

            let model = &self.vox.models[model_id as usize];
            for voxel in &model.voxels {
                let voxel_pos = IVec3::new(voxel.x as i32, voxel.y as i32, voxel.z as i32);

                let mut voxel_pos_swizzled = swizzle(voxel_pos, channel_reordering);

                for i in 0..3 {
                    if channel_reordering[i] < 0 {
                        voxel_pos_swizzled[i] = model_size_swizzled[i] - 1 - voxel_pos_swizzled[i];
                    }
                }

                let color = self.vox.palette[voxel.i as usize];
                let mat_nr = palette.get_index_simple_color([color.r, color.g, color.b])?;

                let voxel_pos = offset + voxel_pos_swizzled;
                array[to_1d(voxel_pos.as_uvec3(), size)] = mat_nr;
            }
        }

        Ok(OffsetVoxelGrid::from_data(size, array, self.min))
    }
}


fn swizzle(v: IVec3, indices: IVec3) -> IVec3 {
    let indices = indices.abs() - 1;
    IVec3::new(
        v[indices.x as usize],
        v[indices.y as usize],
        v[indices.z as usize],
    )
}
