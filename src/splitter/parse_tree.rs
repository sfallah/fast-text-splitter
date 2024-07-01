use aho_corasick::Span;

pub struct Node {
    pub parent: Option<usize>,
    pub children: Option<Vec<usize>>,
    pub lvl: usize,
    pub split_data_span: Span,
    pub pattern_found: usize,
}
// implement debug
impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("parent", &self.parent)
            .field("children", &self.children)
            .field("lvl", &self.lvl)
            .field("split_data_span", &self.split_data_span)
            .field("pattern_found", &self.pattern_found)
            .finish()
    }
}

pub fn print_tree(nodes: &Vec<Node>, start_index: usize, indent: usize) {
    if start_index >= nodes.len() {
        return;
    }
    
    // Print the current node with the appropriate indentation
    println!("{:indent$}{:?}", "", nodes[start_index], indent = indent);

    // Check if the current node has children and recursively print them
    if let Some(children) = nodes[start_index].children.as_ref() {
        for &child_idx in children {
            print_tree(nodes, child_idx, indent + 3); // Increase indentation for children
        }
    }
}