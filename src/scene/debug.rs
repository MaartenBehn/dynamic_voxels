use octa_force::{OctaResult, glam::{IVec3, Vec3A}};
use slotmap::{Key, SlotMap, new_key_type};

use crate::{csg::csg_tree::tree::CSGTree, scene::{gi::SceneGI, object::{SceneAddObject, SceneObject}, worker::{SceneObjectKey, SceneWorker}}, voxel::palette::palette::MATERIAL_ID_DEBUG};

new_key_type! { pub struct DebugShowProbesKey; }

#[derive(Debug, Clone, Copy, Default)]
pub struct ObjectDebug {
    show_probes_key: DebugShowProbesKey,
}

#[derive(Debug)]
pub struct SceneDebugger {
    show_probes: SlotMap<DebugShowProbesKey, SceneProbeVisulator>,
}

#[derive(Debug)]
pub struct SceneProbeVisulator {
    object: SceneObjectKey,
    probe_object: SceneObjectKey,
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

        let mat = object.mat;
        let start = object.allocation.start() as u32;

        let mut csg = CSGTree::default();
        let mut csg_children = vec![];
        for (pos, level) in self.iter_probe_pos_of_object(start) {
            csg_children.push(csg.add_sphere(pos.as_vec3a(), level as f32 * 10.0, MATERIAL_ID_DEBUG));
        }
        csg.root = csg.add_union_node(csg_children);
       
        let probe_object = self.add_object(SceneAddObject {
            mat,
            model: csg,
        })?;

        self.objects[key].debug.show_probes_key = self.debug.show_probes.insert(SceneProbeVisulator { 
            object: key, 
            probe_object, 
        });

        Ok(())
    }

    fn iter_probe_pos_of_object(&self, start: u32) -> impl Iterator<Item = (IVec3, usize)> {
        self.gi.gi_pool.pools.iter()
            .enumerate()
            .flat_map(move |(level, gi_level)| {
                gi_level.debug_iter_used_indices()
                    .filter_map(move |i| {
                        let probe = gi_level.get(i);
                        if probe.object_index == start {
                            Some((probe.position, level))
                        } else {
                            None
                        }
                    })
            })
    }
}

