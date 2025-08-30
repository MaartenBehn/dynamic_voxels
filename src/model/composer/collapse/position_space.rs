use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{position_space::PositionSpaceTemplate, template::TemplateIndex}, util::{number::Nu, vector::Ve}};

use super::collapser::{CollapseChildKey, CollapseNodeKey, Collapser};

#[derive(Debug, Clone)]
pub enum PositionSpace<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> {
    GridInVolume(GridVolume<V3, T>),
    GridOnPlane(GridOnPlane<V2, T>),
    Path(Path<V2, T>)
}

#[derive(Debug, Clone)]
pub struct GridVolume<V: Ve<T, 3>, T: Nu> {
    pub volume: CSGTree<(), V, T, 3>,
    pub spacing: T,
    pub position: SlotMap<CollapseChildKey, V>,
}

#[derive(Debug, Clone)]
pub struct GridOnPlane<V: Ve<T, 2>, T: Nu> {
    pub volume: CSGTree<(), V, T, 2>,
    pub spacing: T,
    pub height: T,
    pub position: SlotMap<CollapseChildKey, V>,
}

#[derive(Debug, Clone)]
pub struct Path<V: Ve<T, 2>, T: Nu> {
    pub spacing: T,
    pub side_variance: V,
    pub start: V,
    pub end: V,
    pub position: SlotMap<CollapseChildKey, V>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PositionSpace<V2, V3, T> {
    pub fn from_template(
        template_space: &PositionSpaceTemplate<V2, V3, T>, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T>
    ) -> Self {
        match &template_space {
            PositionSpaceTemplate::GridInVolume(grid_volume_template) 
            => PositionSpace::GridInVolume(GridVolume { 
                volume: grid_volume_template.volume.get_value(depends, collapser), 
                spacing: grid_volume_template.spacing.get_value(depends, collapser),
                position: Default::default(),
            }),
            PositionSpaceTemplate::GridOnPlane(grid_on_plane_template) 
            => PositionSpace::GridOnPlane(GridOnPlane { 
                volume: grid_on_plane_template.volume.get_value(depends, collapser), 
                spacing: grid_on_plane_template.spacing.get_value(depends, collapser),
                height: grid_on_plane_template.height.get_value(depends, collapser),
                position: Default::default(),
            }),
            PositionSpaceTemplate::Path(path_template) 
            => PositionSpace::Path(Path {
                spacing: path_template.spacing.get_value(depends, collapser),
                side_variance: path_template.side_variance.get_value(depends, collapser),
                start: path_template.start.get_value(depends, collapser),
                end: path_template.end.get_value(depends, collapser),
                position: Default::default(),
            }),
        }
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, index: CollapseChildKey) -> V {
        match self {
            PositionSpace::GridInVolume(grid_volume) => {
                assert_eq!(D, 3);

                // TODO Maybe unsafe cast?
                let v = grid_volume.position[index]; 
                V::from_iter(v.to_array().into_iter())
            },
            PositionSpace::GridOnPlane(grid_on_plane) => {
                assert_eq!(D, 2);

                let v = grid_on_plane.position[index]; 
                V::from_iter(v.to_array().into_iter())
            },
            PositionSpace::Path(path) => {
                assert_eq!(D, 2);

                let v = path.position[index]; 
                V::from_iter(v.to_array().into_iter())
            },
        }
    }
}
