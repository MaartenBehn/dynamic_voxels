use std::time::Instant;

use octa_force::{OctaResult, anyhow::anyhow, glam::{Mat4, Vec3A}, log::{debug, info}};

use crate::{VOXELS_PER_SHADER_UNIT, gi::gi_pool::GIExecutor, scene::{dag_store::{SceneDAGKey, SceneDAGStore}, debug::ObjectDebug, staging_copies::SceneStagingBuilder, worker::{SceneObjectKey, SceneWorker}}, util::{aabb::AABB3, buddy_allocator::ManualBuddyAllocation, default_types::{LODType, Volume}}, voxel::dag64::entry::{DAG64Entry, DAG64EntryKey}};

#[derive(Debug)]
pub struct SceneObject {
    pub bvh_index: usize,
    pub needs_update: bool,
    pub allocation: ManualBuddyAllocation,
    pub mat: Mat4,
    pub model: Volume,
    pub dag_key: SceneDAGKey,
    pub entry_key: DAG64EntryKey,
    pub entry: DAG64Entry,
    pub debug: ObjectDebug,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SceneObjectData {
    pub mat: Mat4,
    pub inv_mat: Mat4,
    pub node_alloc: u64,
    pub data_alloc: u64,
    pub root_index: u32,
}

#[derive(Debug)]
pub struct SceneAddObject {
    pub mat: Mat4,
    pub model: Volume,
}

impl SceneWorker {
    pub fn add_object(&mut self, add_object: SceneAddObject) -> OctaResult<SceneObjectKey> {
        let dag_key = self.dag_store.active_dag();
        let dag = self.dag_store.get_dag_mut(dag_key);

        let allocation = self.allocator.alloc(size_of::<SceneObjectData>())?;
        
        let gi = GIExecutor::new(&self.gi.gi_pool, allocation.start() as u32);

        let now = Instant::now();
        let entry_key = dag.add_pos_query_volume(&add_object.model, &self.lod, gi);

        let elapsed = now.elapsed();
        info!("Voxel DAG Build took: {:?}", elapsed);

        let entry = dag.get_entry(entry_key);

        let key = self.objects.insert(SceneObject {
            bvh_index: 0,
            needs_update: true,
            allocation,
            mat: add_object.mat,
            model: add_object.model,
            dag_key,
            entry_key,
            entry,
            debug: Default::default(),
        });
        self.dag_store.mark_changed(dag_key);
        
        Ok(key)
    }

    pub fn remove_object(&mut self, key: SceneObjectKey) -> OctaResult<SceneObject> {
        self.needs_bvh_update = true;
        self.objects.remove(key)
            .map(|o| Ok(o))
            .unwrap_or(Err(anyhow!("Scene Object Key invalid")))
    }

    pub fn rebuild_all_dag_objects(&mut self) {
        debug!("rebuild_all_dag_objects");

        for object in self.objects.values_mut() {
            object.rebuild(&mut self.dag_store, &self.lod);
            object.needs_update = true;
        }
    }
}

impl SceneObject {
    pub fn get_aabb(&self) -> AABB3 {
        
        let size = self.entry.get_size();
        let aabb = AABB3::from_min_max(
            &self.mat,
            self.entry.offset.as_vec3a() / VOXELS_PER_SHADER_UNIT as f32, 
            (self.entry.offset.as_vec3a() + Vec3A::splat(size as f32)) / VOXELS_PER_SHADER_UNIT as f32,
        );

        aabb
    }

    pub fn update(&mut self, dag_store: &SceneDAGStore, builder: &mut SceneStagingBuilder) {
        if !self.needs_update {
            return;
        }

        let mat = self.entry.calc_mat(self.mat);
        let inv_mat = mat.inverse();

        let dag = &dag_store.dags[self.dag_key];
        
        let data = SceneObjectData {
            mat,
            inv_mat,
            
            node_alloc: dag.node_alloc.start() as u64,
            data_alloc: dag.data_alloc.start() as u64,
            root_index: self.entry.root_index,
        };

        builder.push(&[data], self.allocation.start());

        self.needs_update = false;
    }

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
}

