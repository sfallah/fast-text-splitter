use std::io;
use std::ops::Deref;
use termtree::{Tree};
use crate::splitter::parse_tree::Node;

fn label<N: AsRef<Node>>(node: N) -> String {
    format!("{:?}", node.as_ref())
}

pub fn term_tree<N: AsRef<Node>>(node: N, nodes: &Vec<Node>) -> io::Result<Tree<String>> {
    let result =
        node.as_ref().children.clone().map_or_else( || Vec::new(), |children| {
                let nodes: Vec<_> = children.iter()
                    .map(|child| nodes.get(*child).unwrap().clone())
                    .collect();
                nodes
            })
            .iter()
            .fold(Tree::new(label(node)), |mut root, child| {
                if !child.children.is_none() && !child.children.as_ref().unwrap().is_empty() {
                    let mut child_tree = term_tree(child, nodes).unwrap();
                    root.push(child_tree);
                } else {
                    let mut child_leaf = Tree::new(label(child));
                    root.push(child_leaf);
                }
                root
            });
    Ok(result)
}

