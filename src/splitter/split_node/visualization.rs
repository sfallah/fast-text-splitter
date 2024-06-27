use crate::splitter::split_node::SplitNode;
use std::io;
use termtree::Tree;

fn label<N: AsRef<SplitNode>>(node: N) -> String {
    format!("{:?}", node.as_ref())
}
fn text_label<N: AsRef<SplitNode>>(node: N, data: &[u8]) -> String {
    format!("Text: {:?}", node.as_ref().reconstruct(data).to_owned())
}

pub fn term_tree<N: AsRef<SplitNode>>(node: N, data: &[u8]) -> io::Result<Tree<String>> {
    let result = node.as_ref().children.iter().fold(
        Tree::new(label(node.as_ref())),
        |mut root, child| {
            if !child.children.is_empty() {
                root.push(term_tree(child, data).unwrap());
            } else {
                let mut child_tree = Tree::new(label(child));
                child_tree.clone().with_multiline(true);
                child_tree.push(Tree::new(text_label(child, data)));
                root.push(child_tree);
            }
            root
        },
    );
    Ok(result)
}
