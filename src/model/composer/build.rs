use core::fmt;

use smallvec::SmallVec;

use crate::util::{number::Nu, vector::Ve};

use super::{collapse::collapser::{CollapseNodeKey, Collapser}, nodes::ComposeNode, template::{ComposeTemplate, TemplateIndex}, ModelComposer};

pub trait BS<V2: Ve<T, 2>, V3: Ve<T, 3>, T: Nu>: fmt::Debug + Clone + Default {
    type BuildNodeType: fmt::Debug + Clone + serde::Serialize + serde::de::DeserializeOwned;
    type TemplateValue: fmt::Debug + Clone; 
    type CollapseValue: fmt::Debug + Clone; 

    fn compose_nodes() -> Vec<ComposeNode<V2, V3, T, Self>>;
    fn is_template_node(t: &Self::BuildNodeType) -> bool;
    
    fn get_depends_and_value(
        t: &Self::BuildNodeType, 
        composer_node: &ComposeNode<V2, V3, T, Self>, 
        composer: &ModelComposer<V2, V3, T, Self>,
        template: &ComposeTemplate<V2, V3, T, Self>,
    ) -> (SmallVec<[TemplateIndex; 4]>, Self::TemplateValue);

    fn from_template(
        t: &Self::TemplateValue,
        depends: &[(TemplateIndex, Vec<CollapseNodeKey>)], 
        collapser: &Collapser<V2, V3, T, Self>) -> Self::CollapseValue;

    fn on_collapse(t: &Self::CollapseValue);
}
