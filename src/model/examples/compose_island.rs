use std::marker::PhantomData;

use egui_snarl::{NodeId, OutPinId};
use octa_force::{egui, glam::{IVec2, IVec3}, OctaResult};
use smallvec::SmallVec;

use crate::{csg::csg_tree::tree::CSGTree, model::composer::{build::{CollapseValueTrait, ComposeTypeTrait, GetCollapseValueArgs, GetTemplateValueArgs, OnCollapseArgs, OnDeleteArgs, TemplateValueTrait, BS}, collapse::collapser::{CollapseNodeKey, Collapser}, data_type::ComposeDataType, nodes::{ComposeNode, ComposeNodeGroupe, ComposeNodeInput, ComposeNodeType}, template::{ComposeTemplate, TemplateIndex}, volume::VolumeTemplate, ModelComposer}, util::{number::Nu, vector::Ve}};


// Compose Type

#[derive(Debug)]
pub struct ComposeIsland {
    pub composer: ModelComposer<IVec2, IVec3, i32, ComposeIslandState>,
    pub state: ComposeIslandState,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ComposeType {
    Object
}
impl ComposeTypeTrait for ComposeType {}


// Template Value

#[derive(Debug, Clone)]
pub enum TemplateValue<V: Ve<T, 3>, T: Nu> {
    Object(ObjectTemplate<V, T>)
}

#[derive(Debug, Clone)]
pub struct ObjectTemplate<V: Ve<T, 3>, T: Nu> {
    volume: VolumeTemplate<V, T, 3>
}

impl<V: Ve<T, 3>, T: Nu> TemplateValueTrait for TemplateValue<V, T> {
    fn get_dependend_template_nodes(&self) -> SmallVec<[TemplateIndex; 4]> {
        match self {
            TemplateValue::Object(object_template) => {
                object_template.volume.get_dependend_template_nodes().collect()
            },
        }
    }
}

// Collapse Value

#[derive(Debug, Clone)]
pub enum CollapseValue {
    Object
}

#[derive(Debug, Clone)]
pub struct Object {}

impl CollapseValueTrait for CollapseValue {}


#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ComposeIslandState {}

impl<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu> BS<V2, V3, T> for ComposeIslandState {
    type ComposeType = ComposeType;
    type TemplateValue = TemplateValue<V3, T>;
    type CollapseValue = CollapseValue;

    fn compose_nodes() -> Vec<ComposeNode<V2, V3, T, Self>> {
        vec![
            ComposeNode { 
                t: ComposeNodeType::Build(ComposeType::Object), 
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

    fn is_template_node(t: &Self::ComposeType) -> bool { 
        true
    }

    fn get_template_value(args: GetTemplateValueArgs<V2, V3, T, Self>) -> Self::TemplateValue {
        match args.compose_type {
            ComposeType::Object => {
                let volume = args.composer.make_volume(
                    args.composer.get_input_pin_by_type(args.composer_node, ComposeDataType::Volume3D), args.template);
 
                TemplateValue::Object(ObjectTemplate { volume }) 
            },
        }
    }

    fn get_collapse_value(args: GetCollapseValueArgs<V2, V3, T, Self>) -> Self::CollapseValue {
        CollapseValue::Object
    }

    fn on_collapse(args: OnCollapseArgs<V2, V3, T, Self>) {
    
    }

    fn on_delete(args: OnDeleteArgs<V2, V3, T, Self>) {
    
    }
}

impl ComposeIsland {
    pub fn new() -> Self {
        Self {
            composer: ModelComposer::new(),
            state: ComposeIslandState::default(),
        }
    }

    pub fn update(&mut self) -> OctaResult<()> {
        self.composer.update()
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        self.composer.render(ctx, &mut self.state);
    }
}


