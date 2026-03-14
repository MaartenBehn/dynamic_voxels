use std::any::TypeId;

use octa_force::glam::{vec3, Mat4, Quat, Vec3};

use crate::{csg::{Base, csg_tree::union, primitves::{CSGPrimitive, PrimitiveType, r#box::CSGBox, cylinder::CSGCylinder, sphere::CSGSphere}}, util::{number::Nu, vector::Ve}, voxel::grid::shared::SharedVoxelGrid};

use super::{remove::CSGTreeRemove, tree::{CSGTree, CSGTreeIndex, CSGTreeNode, CSGTreeNodeData, CSG_TREE_INDEX_INVALID}, union::CSGTreeUnion};


impl<M: Base + Send + Sync, V: Ve<T, D>, T: Nu, const D: usize> CSGTree<M, V, T, D> {
    pub fn new_primitive<P: PrimitiveType + 'static>(p: CSGPrimitive<P, M, V::VectorF, D>) -> Self {
        Self::from_node(CSGTreeNode::new_primitive(p))
    }

    pub fn new_sphere(center: V, radius: T, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(center.to_vecf(), radius.to_f32(), mat)))
    }

    pub fn new_sphere_float(center: V::VectorF, radius: f32, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(CSGPrimitive::new_sphere(center, radius, mat)))
    }

    pub fn new_box(center: V, size: V, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_box(CSGPrimitive::new_box(center.to_vecf(), size.to_vecf(), mat)))
    }
    
    pub fn new_box_float(center: V::VectorF, size: V::VectorF, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_box(CSGPrimitive::new_box(center, size, mat)))
    }
}

impl<M: Base + Send + Sync, V: Ve<T, 3>, T: Nu> CSGTree<M, V, T, 3> {
    pub fn new_disk(center: V::VectorF, radius: f32, height: f32, mat: M) -> Self {
        Self::from_node(CSGTreeNode::new_sphere(CSGPrimitive::new_disk(center, radius, height, mat)))
    }

    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        Self::from_node(CSGTreeNode::new_shared_grid(grid))
    }
}

union CSGPrimitiveUnion<PA: PrimitiveType, PB: PrimitiveType, M: Copy, V: Ve<f32, D>, const D: usize> {
    a: CSGPrimitive<PA, M, V, D>,
    b: CSGPrimitive<PB, M, V, D>
}

impl <M: Base, V: Ve<T, D>, T: Nu, const D: usize> CSGTreeNode<M, V, T, D> {
    pub fn new_none() -> Self {
        CSGTreeNode::new(CSGTreeNodeData::None, CSG_TREE_INDEX_INVALID)
    }

    pub fn new_primitive<P: PrimitiveType + 'static>(p: CSGPrimitive<P, M, V::VectorF, D>) -> Self {
        
        let data = if TypeId::of::<P>() == TypeId::of::<CSGSphere>() {
             CSGTreeNodeData::Sphere(unsafe { CSGPrimitiveUnion { a: p }.b })
        } else if TypeId::of::<P>() == TypeId::of::<CSGBox>() {
             CSGTreeNodeData::Box(unsafe { CSGPrimitiveUnion { a: p }.b })
        } else if TypeId::of::<P>() == TypeId::of::<CSGCylinder>() {
             CSGTreeNodeData::Cylinder(unsafe { CSGPrimitiveUnion { a: p }.b })
        } else {
            unreachable!()
        };

        CSGTreeNode::new(data, CSG_TREE_INDEX_INVALID)
    }

    pub fn new_sphere(p: CSGPrimitive<CSGSphere, M, V::VectorF, D>) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Sphere(p), CSG_TREE_INDEX_INVALID)
    }

    pub fn new_box(p: CSGPrimitive<CSGBox, M, V::VectorF, D>) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Box(p), CSG_TREE_INDEX_INVALID)
    }
    
    pub fn new_union(nodes: Vec<CSGTreeIndex>) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Union(CSGTreeUnion::new(nodes)), CSG_TREE_INDEX_INVALID)
    }
    
    pub fn new_cut(base: CSGTreeIndex, cut: CSGTreeIndex) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::Cut(CSGTreeRemove::new(base, cut)), CSG_TREE_INDEX_INVALID)
    }
}

impl <M: Base, V: Ve<T, 3>, T: Nu> CSGTreeNode<M, V, T, 3> {
    pub fn new_shared_grid(grid: SharedVoxelGrid) -> Self {
        CSGTreeNode::new(CSGTreeNodeData::SharedVoxelGrid(grid), CSG_TREE_INDEX_INVALID)
    }
} 
