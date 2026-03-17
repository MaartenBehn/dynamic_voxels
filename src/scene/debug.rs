use octa_force::{OctaResult, glam::{IVec3, Mat4, Vec3A}};
use slotmap::{Key, SlotMap, new_key_type};

use crate::{VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, scene::{gi::SceneGI, object::{SceneAddObject, SceneObject}, worker::{SceneObjectKey, SceneWorker}}, volume::VolumeBounds, voxel::palette::palette::MATERIAL_ID_DEBUG};

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

        let level = 5;

        let mat = object.mat;
        let start = object.allocation.start() as u32;
        let offset = object.entry.offset;

        let mut csg = CSGTree::default();
        let mut csg_children = vec![];
        for pos in self.iter_probe_level(start, level) {
            csg_children.push(csg.add_sphere(pos.as_vec3a(), level as f32 * 1.0, MATERIAL_ID_DEBUG));
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

    fn iter_probes(&mut self, start: u32) -> impl Iterator<Item = (IVec3, usize)> {
        self.gi.gi_pool.pools.iter_mut()
            .enumerate()
            .flat_map(move |(level, gi_level)| {
                gi_level.unique_iter()
                    .filter_map(move |probe| {
                        if probe.object_index == start {
                            Some((probe.position, level))
                        } else {
                            None
                        }
                    })
            })
    }

    fn iter_probe_level(&mut self, start: u32, level: usize) -> impl Iterator<Item = IVec3> {
        self.gi.gi_pool.pools[level].unique_iter()
            .filter_map(move |probe| {
                if probe.object_index == start {
                    Some(probe.position)
                } else {
                    None
                }
            })
    }
}

