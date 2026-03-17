
use std::{rc::Rc, sync::Arc, time::Instant};

use octa_force::{OctaResult, anyhow::anyhow, glam::{Mat4, Quat, Vec3, Vec3A, Vec3Swizzles, Vec4, Vec4Swizzles, vec3, vec4}, log::{debug, info}, vulkan::{Buffer, Context, ash::vk, gpu_allocator::MemoryLocation}};
use parking_lot::Mutex;
use slotmap::{new_key_type, SlotMap};

use crate::{VOXELS_PER_METER, VOXELS_PER_SHADER_UNIT, csg::csg_tree::tree::CSGTree, scene::{object::SceneObjectType, staging_copies::SceneStagingBuilder, worker::{SceneObjectKey, SceneWorker}}, util::{aabb::AABB3, buddy_allocator::ManualBuddyAllocation, default_types::{LODType, Volume}}, volume::VolumeQureyAABB, voxel::dag64::{entry::{DAG64Entry, DAG64EntryKey}, lod_heuristic::LODHeuristicNone, parallel::ParallelVoxelDAG64}};

use super::{dag_store::{SceneDAG, SceneDAGKey, SceneDAGStore}};

#[derive(Debug)]
pub struct SceneDAGObject {
    pub allocation: ManualBuddyAllocation,
    pub mat: Mat4,
    pub model: Volume,
    pub dag_key: SceneDAGKey,
    pub entry_key: DAG64EntryKey,
    pub entry: DAG64Entry,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SceneDAGObjectData {
    pub mat: Mat4,
    pub inv_mat: Mat4,
    pub node_alloc: u64,
    pub data_alloc: u64,
    pub root_index: u32,
}

#[derive(Debug)]
pub struct SceneAddDAGObject {
    pub mat: Mat4,
    pub model: Volume,
}

impl SceneWorker {
    pub fn add_dag64_object(
        &mut self,
        add_dag_object: SceneAddDAGObject,
    ) -> OctaResult<SceneObjectKey> {
        let dag_key = self.dag_store.active_dag();
        let dag = self.dag_store.get_dag_mut(dag_key);

        let now = Instant::now();
        let entry_key = dag.add_aabb_query_volume(&add_dag_object.model, &self.lod);

        let elapsed = now.elapsed();
        info!("Voxel DAG Build took: {:?}", elapsed);

        let entry = dag.get_entry(entry_key);

        let allocation = self.allocator.alloc(size_of::<SceneDAGObjectData>() )?;

        let key = self.add_object(SceneObjectType::DAG64(SceneDAGObject {
            allocation,
            mat: add_dag_object.mat,
            model: add_dag_object.model,
            dag_key,
            entry_key,
            entry,
        }));
        self.dag_store.mark_changed(dag_key);

        Ok(key)
    }

    pub fn rebuild_all_dag_objects(&mut self) {
        debug!("rebuild_all_dag_objects");

        for dag_object in self.objects.values_mut() {
            match &mut dag_object.data {
                SceneObjectType::DAG64(o) => {
                    o.rebuild(&mut self.dag_store, &self.lod);
                },
            }
            dag_object.needs_update = true;
        }

    }
}

impl SceneDAGObject {   
    pub fn rebuild(&mut self, store: &mut SceneDAGStore, lod: &LODType) {
        let dag = store.get_dag_mut(self.dag_key);

        let old_key = self.entry_key;
        
        let now = Instant::now();
        self.entry_key = dag.add_aabb_query_volume(&self.model, lod);

        let elapsed = now.elapsed();
        info!("Voxel DAG Build took: {:?}", elapsed);

        self.entry = dag.get_entry(self.entry_key); 
        dag.remove_entry(old_key);
        
        store.mark_changed(self.dag_key);
    }

    pub fn rebuild_changed(&mut self, store: &mut SceneDAGStore, lod: &LODType) {
        let dag = store.get_dag_mut(self.dag_key);
        
        let old_key = self.entry_key;

        let now = Instant::now();
        self.entry_key = dag.update_aabb_query_volume(&self.model, lod, self.entry_key);

        let elapsed = now.elapsed();
        info!("Voxel DAG Update took: {:?}", elapsed);

        self.entry = dag.get_entry(self.entry_key); 
        dag.remove_entry(old_key);

        store.mark_changed(self.dag_key);
    } 

    pub fn update(&self, dag_store: &SceneDAGStore, builder: &mut SceneStagingBuilder) {
        let mat = self.entry.calc_mat(self.mat);
        let inv_mat = mat.inverse();

        let dag = &dag_store.dags[self.dag_key];
        
        let data = SceneDAGObjectData {
            mat,
            inv_mat,
            
            node_alloc: dag.node_alloc.start() as u64,
            data_alloc: dag.data_alloc.start() as u64,
            root_index: self.entry.root_index,
        };

        builder.push(&[data], self.allocation.start());
    }

    pub fn get_aabb(&self) -> AABB3 {
        
        let size = self.entry.get_size();
        let aabb = AABB3::from_min_max(
            &self.mat,
            self.entry.offset.as_vec3a() / VOXELS_PER_SHADER_UNIT as f32, 
            (self.entry.offset.as_vec3a() + Vec3A::splat(size as f32)) / VOXELS_PER_SHADER_UNIT as f32,
        );

        aabb
    }
}

