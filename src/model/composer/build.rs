use core::fmt;

use smallvec::SmallVec;

use crate::{model::collapse::{add_nodes::GetValueData, collapser::{CollapseNode, Collapser}}, util::{number::Nu, vector::Ve}};

use super::{nodes::ComposeNode, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

pub trait BS<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu>: fmt::Debug + Clone + Send + Sync + 'static {
    type ComposeType: ComposeTypeTrait;
    type TemplateValue: TemplateValueTrait; 
    type CollapseValue: CollapseValueTrait; 

    fn compose_nodes() -> Vec<ComposeNode<Self::ComposeType>>;
    fn is_template_node(t: &Self::ComposeType) -> bool;
    
    fn get_template_value(args: GetTemplateValueArgs<V2, V3, T, Self>) -> Self::TemplateValue;
    fn on_collapse(args: OnCollapseArgs<V2, V3, T, Self>) -> impl std::future::Future<Output = Self::CollapseValue> + Send;
    fn on_delete(args: OnDeleteArgs<V2, V3, T, Self>);
}

pub trait ComposeTypeTrait: fmt::Debug + Clone {
}

pub trait TemplateValueTrait: fmt::Debug + Clone + Send + Sync {
    fn get_dependend_template_nodes(&self) -> SmallVec<[TemplateIndex; 4]>;
}

pub trait CollapseValueTrait: fmt::Debug + Clone + Send + Sync {

}

pub struct GetTemplateValueArgs<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub building_template_index: TemplateIndex,

    pub compose_type: &'a B::ComposeType, 
    pub composer_node: &'a ComposeNode<B::ComposeType>,

    pub composer: &'a ModelComposer<V2, V3, T, B>,
    pub template: &'a ComposeTemplate<V2, V3, T, B>,
}

pub struct OnCollapseArgs<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub template_value: &'a B::TemplateValue,
    pub get_value_data: GetValueData<'a>,
    pub collapse_node: &'a CollapseNode<V2, V3, T, B>,

    pub collapser: &'a Collapser<V2, V3, T, B>,
    pub template: &'a ComposeTemplate<V2, V3, T, B>,
    pub state: &'a mut B,
}

pub struct OnDeleteArgs<'a, V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu, B: BS<V2, V3, T>> {
    pub collapse_node: &'a CollapseNode<V2, V3, T, B>,

    pub collapser: &'a Collapser<V2, V3, T, B>,
    pub template: &'a ComposeTemplate<V2, V3, T, B>,
    pub state: &'a mut B,
}
