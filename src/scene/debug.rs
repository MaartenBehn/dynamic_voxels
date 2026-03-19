use octa_force::{OctaResult, glam::{IVec3, Mat4, Vec3, Vec3A}};
use slotmap::{Key, SlotMap, new_key_type};

use crate::{VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, gi::gi_pool::GI_PROBE_MIN_LEVEL, scene::{gi::SceneGI, object::{SceneAddObject, SceneObject}, worker::{SceneObjectKey, SceneWorker}}, volume::VolumeBounds, voxel::palette::palette::MATERIAL_ID_DEBUG};

new_key_type! { pub struct DebugShowProbesKey; }

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectDebug {
    show_probes_key: DebugShowProbesKey,
}

#[derive(Debug, Default)]
pub struct SceneDebugger {
    show_probes: SlotMap<DebugShowProbesKey, SceneProbeVisulator>,
}

#[derive(Debug)]
pub struct SceneProbeVisulator {
    object: SceneObjectKey,
    probe_object: SceneObjectKey,
    level: usize,
}

impl SceneWorker {
    pub fn show_probes(
        &mut self, 
        key: SceneObjectKey, 
    ) -> OctaResult<()> {
        let object = &mut self.objects[key]; 

        if !object.debug.show_probes_key.is_null() {
            return Ok(());
        }

        let level = 2;

        let mat = object.mat;
        let start = object.allocation.start() as u32;
        let offset = object.entry.offset;

        let mut csg = CSGTree::default();
        let mut csg_children = vec![];

        let size =  object.entry.get_size() as f32;
        for pos in self.iter_probe_level(start, level) {
            let world_pos = (pos - 1.0) * size;

            csg_children.push(csg.add_sphere(Vec3A::from(world_pos), level as f32 * 1.0, MATERIAL_ID_DEBUG));
        }
        csg.root = csg.add_union_node(csg_children);
        csg.calculate_bounds();
       
        let probe_object = self.add_object(SceneAddObject {
            mat: mat,
            model: csg,
        }, false)?;

        self.objects[key].debug.show_probes_key = self.debug.show_probes.insert(SceneProbeVisulator { 
            object: key, 
            probe_object, 
            level
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

