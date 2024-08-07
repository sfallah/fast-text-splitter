use crate::splitter::split_node::SplitNode;
use std::io;
use termtree::{GlyphPalette, Tree};

fn label<N: AsRef<SplitNode>>(node: N) -> String {
    format!("{:?}", node.as_ref())
}
fn text_label<N: AsRef<SplitNode>>(node: N, data: &[u8]) -> String {
    format!("Text: {:?}", node.as_ref().reconstruct(data).to_owned())
}

pub fn term_tree<N: AsRef<SplitNode>>(node: N, data: &[u8], node_text: bool) -> io::Result<Tree<String>> {
    let result =
        node.as_ref()
            .children
            .iter()
            .fold(Tree::new(label(node.as_ref())), |mut root, child| {
                if !child.children.is_empty() {
                    let mut child_tree = term_tree(child, data, node_text).unwrap();
                    if node_text {
                        let mut text_glyph = GlyphPalette::new();
                        text_glyph.item_indent = "────── ";
                        let child_text = Tree::new(text_label(child, data)).with_glyphs(text_glyph);
                        child_tree.push(child_text);
                    }
                    root.push(child_tree);
                } else {
                    let mut child_leaf = Tree::new(label(child));
                    child_leaf.clone().with_multiline(true);
                    let child_text = Tree::new(text_label(child, data));
                    child_leaf.push(child_text);
                    root.push(child_leaf);
                }
                root
            });
    Ok(result)
}
