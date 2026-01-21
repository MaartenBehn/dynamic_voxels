use crate::{model::{data_types::{number::{Hook, NumberTemplate}, number_space::NumberSpaceTemplate, position::PositionTemplate, position_pair_set::PositionPairSetTemplate, position_set::PositionSetTemplate, position_space::PositionSpaceTemplate, volume::VolumeTemplate}, template::{Template, value::{TemplateValue, ValueIndex}}}, util::{number::Nu, vector::Ve}};

pub struct ValueHooksIterator<'a> {
    values: &'a mut [TemplateValue],
    pointers: Vec<ValueIndex>,
}

impl Template {
    pub fn iter_hooks<'a>(&'a mut self, value_index: ValueIndex) -> ValueHooksIterator<'a> {
        ValueHooksIterator::new(&mut self.values, value_index)
    }
}

impl<'a> ValueHooksIterator<'a> {
    pub fn new(values: &'a mut [TemplateValue], value_index: ValueIndex) -> Self {
        ValueHooksIterator {
            values,
            pointers: vec![value_index],
        }
    }
}

impl<'a> Iterator for ValueHooksIterator<'a> {
    type Item = &'a mut Hook;

    fn next(&mut self) -> Option<Self::Item> {

        while let Some(i) = self.pointers.pop() {

            let value: *mut TemplateValue = &mut self.values[i];
            let value = unsafe { &mut *value };

            match value {
                TemplateValue::None => {},
                TemplateValue::Number(number_template) => {
                    match number_template {
                        NumberTemplate::Const(_) => {},
                        NumberTemplate::Hook(hook) => {
                            return Some(hook);
                        },
                        NumberTemplate::SplitPosition2D((p, _)) => {
                            self.pointers.push(*p);
                        },
                        NumberTemplate::SplitPosition3D((p, _)) => {
                            self.pointers.push(*p);
                        },
                        NumberTemplate::Position3DTo2D(p) => {
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::NumberSet(number_space_template) => {
                    match number_space_template {
                        NumberSpaceTemplate::NumberRange { min, max, step } => {
                            self.pointers.push(*min);
                            self.pointers.push(*max);
                            self.pointers.push(*step);
                        },
                    }
                },
                TemplateValue::PositionSpace2D(position_space_template)
                | TemplateValue::PositionSpace3D(position_space_template)=> {
                    match position_space_template {
                        PositionSpaceTemplate::Grid(grid_template) => {
                            self.pointers.push(grid_template.volume);
                            self.pointers.push(grid_template.spacing);
                        },
                        PositionSpaceTemplate::LeafSpread(leaf_spread_template) => {
                            self.pointers.push(leaf_spread_template.volume);
                            self.pointers.push(leaf_spread_template.samples);
                        },
                        PositionSpaceTemplate::Path(path_template) => {
                            self.pointers.push(path_template.start);
                            self.pointers.push(path_template.end);
                            self.pointers.push(path_template.spacing);
                            self.pointers.push(path_template.side_variance);
                        },
                    }
                },
                TemplateValue::Position2D(position_template) => {
                    match position_template {
                        PositionTemplate::Const(_) => {},
                        PositionTemplate::FromNumbers(n) => {
                            self.pointers.push(n[0]);
                            self.pointers.push(n[1]);
                        },
                        PositionTemplate::Add((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionTemplate::Sub((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionTemplate::PerPosition(hook) => {
                            return Some(hook);
                        },
                        PositionTemplate::PerPair((hook, _)) => {
                            return Some(hook);
                        },
                        PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                        PositionTemplate::Cam => {},
                        PositionTemplate::Position2DTo3D((p, n)) => {
                            self.pointers.push(*p);
                            self.pointers.push(*n);
                        },
                        PositionTemplate::Position3DTo2D(p) => {
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::Position3D(position_template) => {
                    match position_template {
                        PositionTemplate::Const(_) => {},
                        PositionTemplate::Add((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionTemplate::Sub((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionTemplate::FromNumbers(n) => {
                            self.pointers.push(n[0]);
                            self.pointers.push(n[1]);
                            self.pointers.push(n[2]);
                        },
                        PositionTemplate::PerPosition(hook) => {
                            return Some(hook);
                        },
                        PositionTemplate::PerPair((hook, _)) => {
                            return Some(hook);
                        },
                        PositionTemplate::PhantomData(phantom_data) => unreachable!(),
                        PositionTemplate::Cam => {},
                        PositionTemplate::Position2DTo3D((p, n)) => {
                            self.pointers.push(*p);
                            self.pointers.push(*n);
                        },
                        PositionTemplate::Position3DTo2D(p) => { 
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::PositionSet2D(position_set_template)
                | TemplateValue::PositionSet3D(position_set_template)=> {
                    match position_set_template {
                        PositionSetTemplate::All(space) => {
                            self.pointers.push(*space);
                        },
                    }
                },
                TemplateValue::PositionPairSet2D(position_pair_set_template)
                | TemplateValue::PositionPairSet3D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetTemplate::ByDistance((space, distance)) => {
                            self.pointers.push(*space);
                            self.pointers.push(*distance);
                        },
                    }
                },
                TemplateValue::Volume2D(volume_template)
                | TemplateValue::Volume3D(volume_template)=> {
                    match volume_template {
                        VolumeTemplate::Sphere { pos, size } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                        },
                        VolumeTemplate::Disk { pos, size, height } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                            self.pointers.push(*height);
                        },
                        VolumeTemplate::Box { pos, size } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                        },
                        VolumeTemplate::Union { a, b } => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        VolumeTemplate::Cut { base, cut } => {
                            self.pointers.push(*base);
                            self.pointers.push(*cut);
                        },
                        VolumeTemplate::Material { mat, child } => {
                            self.pointers.push(*child);
                        },
                    }
                },
                TemplateValue::Voxels(voxel_template) => todo!(),
                TemplateValue::Mesh(mesh_template) => todo!(),
            }
        }

        None
    }
}

