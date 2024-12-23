use octa_force::glam::{Mat4, Quat, Vec3};

use crate::{
    cgs_tree::tree::{CSGNode, CSGNodeData, CSGTree, MATERIAL_NONE},
    wfc::builder::{NumberRangeDefinesType, WFCBuilder},
};

// cargo test wfc::test::test_builder  -- --nocapture
#[test]
pub fn test_builder() {
    let wfc_builder = WFCBuilder::new()
        .node((), |b| {
            b.number_range(0..=5, |b| {
                b.defines(NumberRangeDefinesType::Amount { of_node: 2 })
                    .identifier(1)
            })
            .volume(|b| {
                b.identifier(4)
                    .csg_node(CSGNodeData::Box(Mat4::default(), MATERIAL_NONE))
                })
            .identifier(0)
        })
        .node((), |b| {
            b.identifier(2)
                .number_range(1..=2, |b| {
                    b.defines(NumberRangeDefinesType::Link { to_node: 2 })
                        .identifier(6)
                })
                .pos(|b| {
                    b.in_volume(4).identifier(7)
                })
                .on_collapse_modify_volume_with_pos_attribute(4, 7, |mut csg, pos| {
                    let mut tree = CSGTree::new();
                    tree.nodes.push(CSGNode::new(CSGNodeData::Sphere(
                        Mat4::from_scale_rotation_translation(
                            Vec3::ONE * 0.1,
                            Quat::from_euler(octa_force::glam::EulerRot::XYZ, 0.0, 0.0, 0.0),
                            pos,
                        ),
                        MATERIAL_NONE,
                    )));

                    csg.append_tree_with_remove(tree);
                    csg.set_all_aabbs(0.0);
                    csg
                })
        });

    dbg!(&wfc_builder);

    let mut wfc = wfc_builder.build();

    dbg!(&wfc);

    wfc.collapse(0);

    dbg!(&wfc);
}
