use crate::{model::{data_types::{data_type::TemplateValue, number::{Hook, NumberValue}, number_space::NumberSpaceValue, position::PositionValue, position_pair_set::PositionPairSetValue, position_set::PositionSetValue, position_space::PositionSpaceValue, volume::VolumeValue}, template::{Template, value::ValueIndex}}, util::{number::Nu, vector::Ve}};

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
                        NumberValue::Const(_) => {},
                        NumberValue::Hook(hook) => {
                            return Some(hook);
                        },
                        NumberValue::SplitPosition2D((p, _)) => {
                            self.pointers.push(*p);
                        },
                        NumberValue::SplitPosition3D((p, _)) => {
                            self.pointers.push(*p);
                        },
                        NumberValue::Position3DTo2D(p) => {
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::NumberSet(number_space_template) => {
                    match number_space_template {
                        NumberSpaceValue::NumberRange { min, max, step } => {
                            self.pointers.push(*min);
                            self.pointers.push(*max);
                            self.pointers.push(*step);
                        },
                    }
                },
                TemplateValue::PositionSpace2D(position_space_template)
                | TemplateValue::PositionSpace3D(position_space_template)=> {
                    match position_space_template {
                        PositionSpaceValue::Grid(grid_template) => {
                            self.pointers.push(grid_template.volume);
                            self.pointers.push(grid_template.spacing);
                        },
                        PositionSpaceValue::LeafSpread(leaf_spread_template) => {
                            self.pointers.push(leaf_spread_template.volume);
                            self.pointers.push(leaf_spread_template.samples);
                        },
                        PositionSpaceValue::Path(path_template) => {
                            self.pointers.push(path_template.start);
                            self.pointers.push(path_template.end);
                            self.pointers.push(path_template.spacing);
                            self.pointers.push(path_template.side_variance);
                        },
                    }
                },
                TemplateValue::Position2D(position_template) => {
                    match position_template {
                        PositionValue::Const(_) => {},
                        PositionValue::FromNumbers(n) => {
                            self.pointers.push(n[0]);
                            self.pointers.push(n[1]);
                        },
                        PositionValue::Add((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionValue::Sub((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionValue::PerPosition(hook) => {
                            return Some(hook);
                        },
                        PositionValue::PerPair((hook, _)) => {
                            return Some(hook);
                        },
                        PositionValue::PhantomData(phantom_data) => unreachable!(),
                        PositionValue::Cam => {},
                        PositionValue::Position2DTo3D((p, n)) => {
                            self.pointers.push(*p);
                            self.pointers.push(*n);
                        },
                        PositionValue::Position3DTo2D(p) => {
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::Position3D(position_template) => {
                    match position_template {
                        PositionValue::Const(_) => {},
                        PositionValue::Add((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionValue::Sub((a, b)) => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        PositionValue::FromNumbers(n) => {
                            self.pointers.push(n[0]);
                            self.pointers.push(n[1]);
                            self.pointers.push(n[2]);
                        },
                        PositionValue::PerPosition(hook) => {
                            return Some(hook);
                        },
                        PositionValue::PerPair((hook, _)) => {
                            return Some(hook);
                        },
                        PositionValue::PhantomData(phantom_data) => unreachable!(),
                        PositionValue::Cam => {},
                        PositionValue::Position2DTo3D((p, n)) => {
                            self.pointers.push(*p);
                            self.pointers.push(*n);
                        },
                        PositionValue::Position3DTo2D(p) => { 
                            self.pointers.push(*p);
                        },
                    }
                },
                TemplateValue::PositionSet2D(position_set_template)
                | TemplateValue::PositionSet3D(position_set_template)=> {
                    match position_set_template {
                        PositionSetValue::All(space) => {
                            self.pointers.push(*space);
                        },
                    }
                },
                TemplateValue::PositionPairSet2D(position_pair_set_template)
                | TemplateValue::PositionPairSet3D(position_pair_set_template) => {
                    match position_pair_set_template {
                        PositionPairSetValue::ByDistance((space, distance)) => {
                            self.pointers.push(*space);
                            self.pointers.push(*distance);
                        },
                    }
                },
                TemplateValue::Volume2D(volume_template)
                | TemplateValue::Volume3D(volume_template)=> {
                    match volume_template {
                        VolumeValue::Sphere { pos, size } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                        },
                        VolumeValue::Disk { pos, size, height } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                            self.pointers.push(*height);
                        },
                        VolumeValue::Box { pos, size } => {
                            self.pointers.push(*pos);
                            self.pointers.push(*size);
                        },
                        VolumeValue::Union { a, b } => {
                            self.pointers.push(*a);
                            self.pointers.push(*b);
                        },
                        VolumeValue::Cut { base, cut } => {
                            self.pointers.push(*base);
                            self.pointers.push(*cut);
                        },
                        VolumeValue::Material { mat, child } => {
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

