use std::usize;

use itertools::Itertools;
use octa_force::{anyhow::bail, glam::{IVec3, Mat4, Vec3, Vec3A}, OctaResult};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{build::BS, position_space::PositionSpaceTemplate, template::TemplateIndex}, util::{number::Nu, vector::Ve}, volume::VolumeQureyPosValid};

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
    pub positions: SlotMap<CollapseChildKey, V>,
    pub new_children: Vec<CollapseChildKey>,
}

#[derive(Debug, Clone)]
pub struct GridOnPlane<V: Ve<T, 2>, T: Nu> {
    pub volume: CSGTree<(), V, T, 2>,
    pub spacing: T,
    pub height: T,
    pub positions: SlotMap<CollapseChildKey, V>,
    pub new_children: Vec<CollapseChildKey>,
}

#[derive(Debug, Clone)]
pub struct Path<V: Ve<T, 2>, T: Nu> {
    pub spacing: T,
    pub side_variance: V,
    pub start: V,
    pub end: V,
    pub positions: SlotMap<CollapseChildKey, V>,
    pub new_children: Vec<CollapseChildKey>,
}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> PositionSpace<V2, V3, T> {
    pub fn from_template<B: BS<V2, V3, T>>(
        template_space: &PositionSpaceTemplate<V2, V3, T>, 
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, B>
    ) -> Self {
        match &template_space {
            PositionSpaceTemplate::GridInVolume(grid_volume_template) 
            => PositionSpace::GridInVolume(GridVolume { 
                volume: grid_volume_template.volume.get_value(depends, collapser, ()), 
                spacing: grid_volume_template.spacing.get_value(depends, collapser),
                
                positions: Default::default(),
                new_children: Default::default(),
            }),
            PositionSpaceTemplate::GridOnPlane(grid_on_plane_template) 
            => PositionSpace::GridOnPlane(GridOnPlane { 
                volume: grid_on_plane_template.volume.get_value(depends, collapser, ()), 
                spacing: grid_on_plane_template.spacing.get_value(depends, collapser),
                height: grid_on_plane_template.height.get_value(depends, collapser),
                
                positions: Default::default(),
                new_children: Default::default(),
            }),
            PositionSpaceTemplate::Path(path_template) 
            => PositionSpace::Path(Path {
                spacing: path_template.spacing.get_value(depends, collapser),
                side_variance: path_template.side_variance.get_value(depends, collapser),
                start: path_template.start.get_value(depends, collapser),
                end: path_template.end.get_value(depends, collapser),
                
                positions: Default::default(),
                new_children: Default::default(),
            }),
        }
    }

    pub fn get_position<V: Ve<T, D>, const D: usize>(&self, index: CollapseChildKey) -> V {
        match self {
            PositionSpace::GridInVolume(grid_volume) => {
                assert_eq!(D, 3);

                // TODO Maybe unsafe cast?
                let v = grid_volume.positions[index]; 
                V::from_iter(v.to_array().into_iter())
            },
            PositionSpace::GridOnPlane(grid_on_plane) => {
                assert_eq!(D, 2);

                let v = grid_on_plane.positions[index]; 
                V::from_iter(v.to_array().into_iter())
            },
            PositionSpace::Path(path) => {
                assert_eq!(D, 2);

                let v = path.positions[index]; 
                V::from_iter(v.to_array().into_iter())
            },
        }
    }

    pub fn is_child_valid(&self, index: CollapseChildKey) -> bool {
        match self {
            PositionSpace::GridInVolume(grid_volume) => grid_volume.positions.contains_key(index) ,
            PositionSpace::GridOnPlane(grid_on_plane) => grid_on_plane.positions.contains_key(index),
            PositionSpace::Path(path) => path.positions.contains_key(index),
        }
    }

    pub fn update(&mut self) {
        match self {
            PositionSpace::GridInVolume(grid_volume) => {
                let new_positions = grid_volume.volume.get_grid_positions(grid_volume.spacing).collect_vec();
                grid_volume.new_children = update_positions(new_positions, &mut grid_volume.positions);
            },
            PositionSpace::GridOnPlane(grid_on_plane) => {
                let mut new_positions = grid_on_plane.volume.get_grid_positions(grid_on_plane.spacing).collect_vec();
                grid_on_plane.new_children = update_positions(new_positions, &mut grid_on_plane.positions);
            },
            PositionSpace::Path(path) => {
                let mut new_positions = path.get_positions();
                path.new_children = update_positions(new_positions, &mut path.positions);
            },
        }
    }

    pub fn get_new_children(&self) -> &[CollapseChildKey] {
        match self {
            PositionSpace::GridInVolume(grid_volume) => &grid_volume.new_children,
            PositionSpace::GridOnPlane(grid_on_plane) => &grid_on_plane.new_children,
            PositionSpace::Path(path) => &path.new_children,
        }
    }
}

fn update_positions<V: Ve<T, D>, T: Nu, const D: usize>(
    mut new_positions: Vec<V>, 
    positions: &mut SlotMap<CollapseChildKey, V>
) -> Vec<CollapseChildKey> {
    positions.retain(|_, p| {
        if let Some(i) = new_positions.iter().position(|t| *t == *p) {
            new_positions.swap_remove(i);
            true
        } else {
            false
        }
    });

    new_positions.iter()
        .map(|p| positions.insert(*p))
        .collect_vec()

}

impl<V: Ve<T, 2>, T: Nu> Path<V, T> {
    pub fn get_positions(&self) -> Vec<V> {
        let end: V::VectorF = self.end.to_vecf();
        let side_variance: V::VectorF = self.side_variance.to_vecf();
        let spacing = self.spacing.to_f32();
        
        let mut points = vec![self.start];
        let mut current: V::VectorF = self.start.to_vecf();

        loop {
            let delta: V::VectorF = end - current;
            let length = delta.length();
            if length < spacing {
                points.push(self.end);
                return points;
            }

            let r = V::VectorF::new([fastrand::f32(), fastrand::f32()]) * 2.0 - 1.0;
            let side = r * side_variance * length;
            let dir = (delta + side).normalize();
            current = current + dir * spacing;
            points.push(V::from_vecf(current));
        }
    }
}
