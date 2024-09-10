use taffy::{
    compute_block_layout, compute_cached_layout, compute_flexbox_layout, compute_grid_layout,
    compute_root_layout, prelude::*, Cache, Display, Layout, LayoutOutput, Style,
};

// taffy location
pub struct TafLocation {
    kind: Display,
    style: Style,
    cache: Cache,
    layout: Layout,
    children: Vec<TafLocation>,
}

impl Clone for TafLocation {
    fn clone(&self) -> Self {
        TafLocation {
            kind: self.kind,
            style: self.style.clone(),
            cache: Cache::default(), // reset cache on clones
            layout: self.layout,
            children: self.children.clone(),
        }
    }
}

impl std::fmt::Debug for TafLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TafLocation")
            .field("kind", &self.kind)
            .field("style", &self.style)
            .field("layout", &self.layout)
            .field("children", &self.children)
            .finish()
    }
}

impl Default for TafLocation {
    fn default() -> Self {
        TafLocation {
            kind: Display::Flex,
            style: Style::default(),
            cache: Cache::default(),
            layout: Layout::with_order(0),
            children: Vec::new(),
        }
    }
}

#[allow(dead_code)]
impl TafLocation {
    pub fn new_block(style: Style) -> TafLocation {
        TafLocation {
            kind: Display::Block,
            style: Style {
                display: Display::Block,
                ..style
            },
            ..TafLocation::default()
        }
    }
    pub fn new_row(style: Style) -> TafLocation {
        TafLocation {
            kind: Display::Flex,
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                ..style
            },
            ..TafLocation::default()
        }
    }
    pub fn new_column(style: Style) -> TafLocation {
        TafLocation {
            kind: Display::Flex,
            style: Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                ..style
            },
            ..TafLocation::default()
        }
    }
    pub fn new_grid(style: Style) -> TafLocation {
        TafLocation {
            kind: Display::Grid,
            style: Style {
                display: Display::Grid,
                ..style
            },
            ..TafLocation::default()
        }
    }
    pub fn append_child(&mut self, node: TafLocation) {
        self.children.push(node);
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_root_layout(self, NodeId::from(usize::MAX), available_space);
    }

    /// The methods on LayoutPartialTree need to be able to access:
    ///
    ///  - The node being laid out
    ///  - Direct children of the node being laid out
    ///
    /// Each must have an ID. For children we simply use it's index. For the node itself
    /// we use usize::MAX on the assumption that there will never be that many children.
    fn node_from_id(&self, node_id: NodeId) -> &TafLocation {
        let idx = usize::from(node_id);
        if idx == usize::MAX {
            self
        } else {
            &self.children[idx]
        }
    }

    fn node_from_id_mut(&mut self, node_id: NodeId) -> &mut TafLocation {
        let idx = usize::from(node_id);
        if idx == usize::MAX {
            self
        } else {
            &mut self.children[idx]
        }
    }
}

pub struct ChildIter(std::ops::Range<usize>);
impl Iterator for ChildIter {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(NodeId::from)
    }
}

impl taffy::TraversePartialTree for TafLocation {
    type ChildIter<'a> = ChildIter;

    fn child_ids(&self, _node_id: NodeId) -> Self::ChildIter<'_> {
        ChildIter(0..self.children.len())
    }

    fn child_count(&self, _node_id: NodeId) -> usize {
        self.children.len()
    }

    fn get_child_id(&self, _node_id: NodeId, index: usize) -> NodeId {
        NodeId::from(index)
    }
}

impl taffy::LayoutPartialTree for TafLocation {
    //type CacheMut<'b> = &'b mut Cache where Self: 'b;

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &Layout) {
        self.node_from_id_mut(node_id).layout = *layout
    }

    fn get_cache_mut(&mut self, node_id: NodeId) -> &mut Cache {
        &mut self.node_from_id_mut(node_id).cache
    }

    fn get_style(&self, node_id: NodeId) -> &Style {
        &self.node_from_id(node_id).style
    }

    fn compute_child_layout(
        &mut self, node_id: NodeId, inputs: taffy::tree::LayoutInput,
    ) -> taffy::tree::LayoutOutput {
        compute_cached_layout(self, node_id, inputs, |parent, node_id, inputs| {
            let node = parent.node_from_id_mut(node_id);

            match node.kind {
                Display::Block => compute_block_layout(node, node_id, inputs),
                Display::Flex => compute_flexbox_layout(node, node_id, inputs),
                Display::Grid => compute_grid_layout(node, node_id, inputs),
                Display::None => LayoutOutput::HIDDEN,
            }
        })
    }
}

//fn main() -> Result<(), taffy::TaffyError> {
//    // Compute layout
//    root.compute_layout(Size::MAX_CONTENT);
//}
