use octa_force::{OctaResult, glam::{IVec3, Mat4, Vec3, Vec3A}};
use slotmap::{Key, SlotMap, new_key_type};

use crate::{csg::csg_tree::tree::CSGTree, gi::gi_pool::GI_PROBE_MIN_LEVEL, scene::{gi::SceneGI, object::{SceneAddObject, SceneObject}, worker::{SceneObjectKey, SceneWorker}}, util::shader_constants::{MATERIAL_ID_PROBE, VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT}, volume::VolumeBounds, voxel::palette::palette::MATERIAL_ID_DEBUG};

new_key_type! { pub struct DebugShowProbesKey; }

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectDebug {
    show_probes_key: DebugShowProbesKey,
}

#[derive(Debug, Default)]
pub struct SceneDebugger {
    pub probe_location: SlotMap<DebugShowProbesKey, SceneProbeLocations>,
    pub probe_data: Option<SceneProbeData>, 
}

#[derive(Debug)]
pub struct SceneProbeLocations {
    object: SceneObjectKey,
    probe_object: SceneObjectKey,
    level: usize,
}

#[derive(Debug)]
pub struct SceneProbeData {
    probe_object: SceneObjectKey,
    pub active_index: usize,
    pub probe_position: Vec3,
}

impl SceneWorker {
    pub fn show_probe_location(
        &mut self, 
        key: SceneObjectKey, 
    ) -> OctaResult<()> {
        let object = &mut self.objects[key]; 

        if !object.debug.show_probes_key.is_null() {
            return Ok(());
        }

        let level = 3;

        let mat = object.mat;
        let start = object.allocation.start() as u32;
        let offset = object.entry.offset;

        let mut csg = CSGTree::default();
        let mut csg_children = vec![];

        let size =  object.entry.get_size() as f32;
        let offset = object.entry.offset.as_vec3() / size; 
        for pos in self.iter_probe_level(start, level) {
            let world_pos = (pos - 1.0 + offset) * size;

            csg_children.push(csg.add_sphere(Vec3A::from(world_pos), level as f32 * 1.0, MATERIAL_ID_DEBUG));
        }
        csg.root = csg.add_union_node(csg_children);
        csg.calculate_bounds();

        let probe_object = self.add_object(SceneAddObject {
            mat: mat,
            model: csg,
        }, false)?;

        self.objects[key].debug.show_probes_key = self.debug.probe_location.insert(SceneProbeLocations { 
            object: key, 
            probe_object, 
            level
        });

        Ok(())
    }

    pub fn show_probe_data(
        &mut self, 
        active_index: usize,
    ) -> OctaResult<()> {
        debug_assert!(active_index < self.gi.active.active_size as usize);

        if let Some(data) = &self.debug.probe_data {
            self.remove_object(data.probe_object);
        }

        let (position, object_offset) = {
            let probe = self.gi.gi_pool.pools[0].get(active_index).unwrap();
            (probe.position, probe.object_offset)
        };

        let object = self.objects.values()
            .find(|o| o.allocation.start() == object_offset as usize)
            .unwrap();

        let offset = (object.entry.offset.as_vec3() / object.entry.get_size() as f32); 
        let world_pos = (position - 1.0 + offset) * object.entry.get_size() as f32;

        let radius = 5;
        let mat = object.mat;

        let mut csg = CSGTree::new_sphere(
            Vec3A::from(world_pos), 
            radius as f32, 
            MATERIAL_ID_PROBE as u8);

        csg.calculate_bounds();

        let probe_object = self.add_object(SceneAddObject {
            mat,
            model: csg,
        }, false)?;

        self.debug.probe_data = Some(SceneProbeData {
            probe_object,
            active_index,
            probe_position: world_pos / VOXELS_PER_METER as f32,
        });

        Ok(()) 
    }

    fn iter_probes(&mut self, start: u32) -> impl Iterator<Item = (Vec3, usize)> {
        self.gi.gi_pool.pools.iter_mut()
            .enumerate()
            .flat_map(move |(level, gi_level)| {
                gi_level.unique_iter()
                    .filter_map(move |probe| {
                        if probe.object_offset == start {
                            Some((probe.position, level))
                        } else {
                            None
                        }
                    })
            })
    }

    fn iter_probe_level(&mut self, start: u32, level: usize) -> impl Iterator<Item = Vec3> {
        self.gi.gi_pool.pools[level - GI_PROBE_MIN_LEVEL as usize].unique_iter()
            .filter_map(move |probe| {
                if probe.object_offset == start {
                    Some(probe.position)
                } else {
                    None
                }
            })
    }
}

