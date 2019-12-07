// Useful links:
//  * https://www.w3.org/TR/css-display-3/#css-box
//  * https://www.w3.org/TR/2018/WD-css-box-3-20181218/#intro

use crate::dom::tree::NodeRef;
use crate::layout::BoxType::Anonymous;
use crate::style::values::computed::Display;
use std::mem::discriminant;

/// Takes a DOM node and builds the corresponding layout tree of it and its children.
pub fn build_layout_tree(node: NodeRef) -> Option<LayoutBox> {
    let computed_opt = &*node.computed_values();
    let computed_values = computed_opt
        .as_ref()
        .expect("layout called on a node that has not yet acquired computed values");
    let mut layout_box = match computed_values.display {
        Display::Block => LayoutBox::new(BoxType::Block(node.clone())),
        Display::Inline => LayoutBox::new(BoxType::Inline(node.clone())),
        Display::None => {
            return None;
        }
    };

    for child in node.children() {
        let child_computed_opt = &*child.computed_values();
        let child_computed_values = child_computed_opt
            .as_ref()
            .expect("layout called on a node that has not yet acquired computed values");
        match child_computed_values.display {
            Display::Block => match build_layout_tree(child.clone()) {
                // TODO: We don't handle the case where a block-flow child box is added to an inline
                // box.  This current behavior is wrong.  To fix, see: https://www.w3.org/TR/CSS2/visuren.html#box-gen
                // Namely, the paragraph that begins with "When an inline box contains an in-flow block-level box"
                Some(child_box) => layout_box.children.push(child_box),
                None => {}
            },
            Display::Inline => match build_layout_tree(child.clone()) {
                Some(child_box) => layout_box.get_inline_container().children.push(child_box),
                None => {}
            },
            Display::None => {}
        }
    }
    return Some(layout_box);
}

/// https://www.w3.org/TR/2018/WD-css-box-3-20181218/#box-model
#[derive(Clone, Debug, Default)]
struct Dimensions {
    // Position of the content area relative to the document origin:
    content: Rect,

    // Surrounding edges:
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

#[derive(Clone, Debug, Default)]
pub struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug, Default)]
pub struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutBox {
    dimensions: Dimensions,
    box_type: BoxType,
    children: Vec<LayoutBox>,
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: Vec::new(),
        }
    }

    /// If a block box contains inline-children, an anonymous box must be used to contain them.
    ///
    /// If this box is already an inline or anonymous box, we can use ourself to contain the inline
    /// children.  Otherwise, find or create an anonymous box.
    fn get_inline_container(&mut self) -> &mut LayoutBox {
        match self.box_type {
            BoxType::Inline(_) | BoxType::Anonymous => self,
            BoxType::Block(_) => {
                match self.children.last() {
                    Some(last_child)
                        if discriminant(&last_child.box_type)
                            == discriminant(&BoxType::Anonymous) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::Anonymous)),
                }
                self.children
                    .last_mut()
                    .expect("there should've been at least one child")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum BoxType {
    Block(NodeRef),
    Inline(NodeRef),
    Anonymous,
}