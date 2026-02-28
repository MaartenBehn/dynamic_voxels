use crate::model::{collapse::collapser::CollapseValueT, composer::output_state::OutputState, data_types::data_type::CollapseValue};

#[derive(Debug, Clone, Copy, Default)]
pub struct NoneCollapserValue {}

impl CollapseValueT for NoneCollapserValue {
    fn on_delete(&self, state: &mut OutputState) {
    }
}
