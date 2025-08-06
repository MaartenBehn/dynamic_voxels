use core::fmt;
use std::{borrow::Cow, marker::PhantomData, time::Instant};

use egui_node_graph2::{DataTypeTrait, Graph, GraphEditorState, InputParamKind, NodeDataTrait, NodeId, NodeResponse, NodeTemplateIter, NodeTemplateTrait, UserResponseTrait, WidgetValueTrait};
use octa_force::{controls::Controls, egui, log::debug, OctaResult};

use crate::csg::{csg_tree_2d::tree::CSGTree2D, fast_query_csg_tree::tree::FastQueryCSGTree, slot_map_csg_tree::tree::SlotMapCSGTreeKey, vec_csg_tree::tree::VecCSGTree};

use super::{examples::islands::{Identifier, IslandKey}, generation::{collapse::{CollapseNodeKey, Collapser}, traits::{ModelGenerationTypes, BU, IT}}};

#[derive(Clone, Copy, Debug, Default)]
pub struct GenerationTypes {}
impl ModelGenerationTypes for GenerationTypes {
    type Identifier = Identifier;
    type UndoData = IslandKey;
    type Volume = FastQueryCSGTree<()>;
    type Volume2D = CSGTree2D<()>;
}

type UserState<GenerationTypes> = Collapser<GenerationTypes>;

const SHOW_COOLDOWN_TIME: f32 = 0.2;

/// Additional (besides inputs and outputs) state to be stored inside each node.
#[derive(Debug)]
pub struct NodeData<T: ModelGenerationTypes> {
    p: PhantomData<T>,
    pub collapse_key: CollapseNodeKey,
}

// Connection variant. Equal DataType means input port is compatible with output port.
// Typically an enum, but this example has only one connection type (any output can be connected to any input),
// so this type is dummied out.
#[derive(PartialEq, Eq, Debug, Default)]
pub struct DataType;

/// Type of the editable value that is used as a fallback for unconnected input node,
/// i.e. when some input to a node can be either constant or taken from another node,
/// this defines how to store that constant.
///
/// This example does not feature editable content within nodes, so this type is dummy.
#[derive(Copy, Clone, Debug, Default)]
pub struct ValueType<T: ModelGenerationTypes> {
    p: PhantomData<T>
}


/// Typically an enum that lists node types.
/// In this example there is only one node type ("Node"),
/// so no this type is dummy.
#[derive(Clone, Copy, Default)]
pub struct DummyNodeTemplate<T: ModelGenerationTypes> {
    p: PhantomData<T>
}

/// Additional events that bubble up from `NodeDataTrait::bottom_ui` back to your app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DummyResponse;


/// Defines how to render edges (connections) between nodes
impl<T: ModelGenerationTypes> DataTypeTrait<UserState<T>> for DataType {
    fn data_type_color(&self, _user_state: &mut UserState<T>) -> egui::Color32 {
        egui::Color32::from_rgb(238, 207, 60)
    }

    fn name(&self) -> Cow<'_, str> {
        "edge".into()
    }
}

/// Defines how to name and construct each node variant and what inputs and
/// outputs each node variant has.
impl<T: ModelGenerationTypes> NodeTemplateTrait for DummyNodeTemplate<T> {
    type NodeData = NodeData<T>;
    type DataType = DataType;
    type ValueType = ValueType<T>;
    type UserState = UserState<T>;
    type CategoryType = &'static str;

    fn node_finder_label(&self, _user_state: &mut Self::UserState) -> Cow<'_, str> {
        "".into()
    }

    fn node_graph_label(&self, _user_state: &mut Self::UserState) -> String {
        "Node".to_owned()
    }

    fn user_data(&self, _user_state: &mut Self::UserState) -> Self::NodeData {
        todo!()
    }

    fn build_node(
        &self,
        graph: &mut Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
        node_id: NodeId,
    ) {
        todo!()
    }
}

/// Enumeration of all node variants to populate the context menu that allows creating nodes
#[derive(Clone, Copy, Default)]
pub struct AllNodeTemplates<T: ModelGenerationTypes>{
    p: PhantomData<T>
}

impl<T: ModelGenerationTypes> NodeTemplateIter for AllNodeTemplates<T> {
    type Item = DummyNodeTemplate<T>;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![]
    }
}

/// Defines how to render input's GUI when it is not connected.
impl<T: ModelGenerationTypes> WidgetValueTrait for ValueType<T> {
    type Response = DummyResponse;
    type UserState = UserState<T>;
    type NodeData = NodeData<T>;

    fn value_widget(
        &mut self,
        _param_name: &str,
        _node_id: NodeId,
        ui: &mut egui::Ui,
        _user_state: &mut Self::UserState,
        _node_data: &NodeData<T>,
    ) -> Vec<DummyResponse> {
        ui.label(_param_name);
        Vec::new()
    }
}

impl UserResponseTrait for DummyResponse {}

/// Defines how to render node window (besides inputs and output ports)
impl<T: ModelGenerationTypes> NodeDataTrait for NodeData<T> {
    type Response = DummyResponse;
    type UserState = UserState<T>;
    type DataType = DataType;
    type ValueType = ValueType<T>;

    fn bottom_ui(
        &self,
        _ui: &mut egui::Ui,
        _node_id: NodeId,
        _graph: &Graph<NodeData<T>, Self::DataType, Self::ValueType>,
        _user_state: &mut Self::UserState,
    ) -> Vec<NodeResponse<Self::Response, NodeData<T>>>
where
        DummyResponse: UserResponseTrait,
    {
        vec![]
    }
}

/// Main graph editor type
type MyEditorState<I> = GraphEditorState<
NodeData<I>,
DataType,
ValueType<I>,
DummyNodeTemplate<I>,
UserState<I>,
>;

pub struct ModelDebugRenderer<T: ModelGenerationTypes> {
    state: MyEditorState<T>,
    level_counter: Vec<usize>,
    show: bool, 
    last_show_change: Instant,
}

impl<T: ModelGenerationTypes> ModelDebugRenderer<T> {
    pub fn render(&mut self, ctx: &egui::Context, collapser: &mut UserState<T>) {
        if !self.show {
            return;
        }

        // Add main panel with the interactive graph
        egui::CentralPanel::default().show(ctx, |ui| {
            // Triger graph display and obtain user interaction events, if any.
            let ret = self.state.draw_graph_editor(
                ui,
                AllNodeTemplates::default(),
                collapser,
                Vec::default(),
            );
        });
    }

    pub fn update_show(&mut self, controls: &Controls) {
        if self.last_show_change.elapsed().as_secs_f32() > SHOW_COOLDOWN_TIME && controls.f2 {
            self.show = !self.show;
            self.last_show_change = Instant::now();
        }
    }

    pub fn update(&mut self, collapser: &mut UserState<T>) {
       
        self.state.graph.nodes.clear();
        self.state.node_order.clear();
        self.level_counter.clear();
        self.state.node_positions.clear();
        self.state.graph.inputs.clear();
        self.state.graph.outputs.clear();
        self.state.graph.connections.clear();

        let keys = collapser.nodes.keys().collect::<Vec<_>>();
        for key in keys.iter() {
            self.add_node(*key, collapser);
        }

        for key in keys.iter() {
            self.add_child_connections(*key, collapser);
        }
    }

    fn add_node(&mut self, node_index: CollapseNodeKey, collapser: &mut UserState<T>){
        let collapser_node = &collapser.nodes[node_index];

        let id =
        self.state
            .graph
            .add_node(
                format!("{:?}", collapser_node.identifier),
                NodeData { 
                    collapse_key: node_index, 
                    p: PhantomData::default(),
                },
                |_g, _id| {

                });

        // Supplement z-order for the node (panic if missing)
        self.state.node_order.push(id);

        while self.level_counter.len() <= collapser_node.level {
            self.level_counter.push(0);
        }

        let y = self.level_counter[collapser_node.level];
        self.level_counter[collapser_node.level] += 1;

        let pos = egui::Pos2{ x: (collapser_node.level - 1) as f32 * 500.0, y: y as f32 * 300.0  } * self.state.pan_zoom.zoom;

        // Position the node within editor area (panic if missing)
        self.state
            .node_positions
            .insert(id, pos);

        for (_, index) in collapser_node.depends.iter() {
            let other_node = &collapser.nodes[*index];
            self.state.graph.add_input_param(
                id,
                format!("{:?}", other_node.identifier),
                DataType,
                ValueType { p: PhantomData::default() },
                InputParamKind::ConnectionOnly,
                true,
            );
        }

        for index in collapser_node.children.iter()
            .map(|(_, c)| c) 
            .flatten() {
            let other_node = &collapser.nodes[*index];
            self.state.graph.add_output_param(
                id,
                format!("{:?}", other_node.identifier),
                DataType,
            );
        }
    }

    pub fn add_child_connections(&mut self, node_index: CollapseNodeKey, collapser: &mut UserState<T>) {
        let collapser_node = &collapser.nodes[node_index];
        let graph_node = self.state.graph.nodes.iter()
            .find(|(_, data)| data.user_data.collapse_key == node_index)
            .map(|(_, data)| data)
            .expect("Graph did not have node with child index");

        for (i, (index, output)) in collapser_node.children.iter()
            .map(|(_, c)| c) 
            .flatten()
            .zip(graph_node.output_ids().collect::<Vec<_>>().into_iter())
            .enumerate(){
            let other_node = &collapser.nodes[*index];
            let depends_nr = other_node.depends.iter()
                .position(|(_, k)| *k == node_index)
                .expect("Child did not have a depends entry of node");

            let other_graph_node = self.state.graph.nodes.iter()
                .find(|(_, data)| data.user_data.collapse_key == *index)
                .map(|(_, data)| data)
                .expect("Graph did not have node with child index");

            let input = &other_graph_node.input_ids().nth(depends_nr)
                .expect("Graph node did not have enough Inputs");

            self.state.graph.add_connection(output, *input, 0);
        }

    }
}

impl<T: ModelGenerationTypes> Default for ModelDebugRenderer<T> {
    fn default() -> Self {
        Self { 
            state: Default::default(), 
            level_counter: Default::default(), 
            show: Default::default(), 
            last_show_change: Instant::now() 
        }
    }
}

impl<T: ModelGenerationTypes> fmt::Debug for ModelDebugRenderer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelDebugRenderer").field("state", &()).field("level_counter", &self.level_counter).field("show", &self.show).field("last_show_change", &self.last_show_change).finish()
    }
}
