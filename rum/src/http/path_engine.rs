use super::Handler;
use std::cell::{Ref, RefCell, RefMut};
use std::cmp::{Ordering, PartialOrd};

type NodeId = usize;

pub struct Arena {
    nodes: Vec<RefCell<Node>>,
}

impl<'a> Arena {
    pub fn new() -> Self {
        Arena { nodes: vec![] }
    }

    pub fn add_node(&mut self, part: &str) -> NodeId {
        let id = self.nodes.len();
        let node = RefCell::new(Node::root(id, part));
        self.nodes.push(node);
        id
    }

    pub fn node(&'a self, id: NodeId) -> Ref<'a, Node> {
        self.nodes[id].borrow()
    }

    pub fn node_mut(&'a self, id: NodeId) -> RefMut<'a, Node> {
        self.nodes[id].borrow_mut()
    }
}

#[derive(Default)]
pub struct Node {
    id: NodeId,
    parent: Option<NodeId>,
    left: Option<NodeId>,
    right: Option<NodeId>,
    handler: Option<Handler>,
    part: String,
}

impl Node {
    pub fn root(id: NodeId, part: &str) -> Node {
        Node {
            id,
            part: part.to_owned(),
            ..Default::default()
        }
    }

    pub fn add(&mut self, arena: &mut Arena, part: &str) {
        let id = arena.add_node(part);
        let mut node_ref = arena.node_mut(id);
        node_ref.parent = Some(self.id);

        if node_ref.lt(self) {
            self.left = Some(id);
        } else {
            self.right = Some(id);
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.part.eq(&other.part)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.part.partial_cmp(&other.part)
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_node_arena() {
        let _parts = vec!["one", "two", "three", "four"];
    }
}
