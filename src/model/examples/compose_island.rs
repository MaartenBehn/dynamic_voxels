use std::marker::PhantomData;

use egui_snarl::{NodeId, OutPinId};
use octa_force::{egui, glam::{IVec2, IVec3}, OctaResult};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{build::BS, collapse::collapser::{CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeGroupe, ComposeNodeInput, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex}, volume::VolumeTemplate, ModelComposer}, util::{number::Nu, vector::Ve}};


#[derive(Debug)]
pub struct ComposeIsland {
    pub composer: ModelComposer<IVec2, IVec3, i32, ComposeIslandState>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BuildType {
    Object
}

#[derive(Debug, Clone)]
pub enum TemplateValue<V: Ve<T, 3>, T: Nu> {
    Object(ObjectTemplate<V, T>)
}

#[derive(Debug, Clone)]
pub struct ObjectTemplate<V: Ve<T, 3>, T: Nu> {
    volume: VolumeTemplate<V, T, 3>
}

#[derive(Debug, Clone)]
pub enum CollapseValue {
    Object
}

#[derive(Debug, Clone)]
pub struct Object {}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ComposeIslandState {}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> BS<V2, V3, T> for ComposeIslandState {
    type BuildNodeType = BuildType;
    type TemplateValue = TemplateValue<V3, T>;
    type CollapseValue = CollapseValue;


    fn compose_nodes() -> Vec<ComposeNode<V2, V3, T, Self>> {
        vec![
            ComposeNode { 
                t: ComposeNodeType::Build(BuildType::Object), 
                id: NodeId(usize::MAX),
                group: ComposeNodeGroupe::Build, 
                inputs: vec![
                    ComposeNodeInput { 
                        name: "ammount".to_string(), 
                        data_type: ComposeDataType::Ammount, 
                    },
                    ComposeNodeInput { 
                        name: "volume".to_string(), 
                        data_type: ComposeDataType::Volume3D, 
                    },
                ], 
                outputs: vec![], 
            },
        ]
    }

    fn is_template_node(t: &Self::BuildNodeType) -> bool { 
        true
    }

    fn get_depends_and_value(
        t: &Self::BuildNodeType, 
        node: &ComposeNode<V2, V3, T, Self>, 
        composer: &ModelComposer<V2, V3, T, Self>,
        template: &ComposeTemplate<V2, V3, T, Self>,
    ) -> (SmallVec<[TemplateIndex; 4]>, Self::TemplateValue) {
        match t {
            BuildType::Object => {
                let volume = composer.make_volume(
                    composer.get_input_pin_by_type(node, ComposeDataType::Volume3D), template);

                (
                    volume.get_dependend_template_nodes().collect(),
                    TemplateValue::Object(ObjectTemplate { volume })
                )
            },
        }
    }

    fn from_template(
        t: &Self::TemplateValue,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, Self>) -> Self::CollapseValue {
        CollapseValue::Object
    }

    
}




impl ComposeIsland {
    pub fn new() -> Self {
        Self {
            composer: ModelComposer::new(),
        }
    }

    pub fn update(&mut self) -> OctaResult<()> {
        self.composer.update()
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.composer.render(ctx);
    }
}


