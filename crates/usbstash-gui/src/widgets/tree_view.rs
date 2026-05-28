use std::collections::HashSet;
use usbstash_core::StashEntry;

/// A node in the directory tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: String,
    pub children: Vec<TreeNode>,
    pub is_file: bool,
}

impl TreeNode {
    /// Create a new directory node.
    pub fn dir(name: String, path: String) -> Self {
        Self {
            name,
            path,
            children: Vec::new(),
            is_file: false,
        }
    }

    /// Create a new file node.
    pub fn file(name: String, path: String) -> Self {
        Self {
            name,
            path,
            children: Vec::new(),
            is_file: true,
        }
    }

    /// Sort children recursively: directories first (alphabetically), then files (alphabetically).
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            // Directories first
            match (a.is_file, b.is_file) {
                (false, true) => std::cmp::Ordering::Less,
                (true, false) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

/// Build a tree structure from flat stash entries.
///
/// Each entry's path is split by path separators to create nested directory nodes.
/// For example, "docs/report.pdf" becomes:
/// - docs/
///   - report.pdf (file)
pub fn build_tree(entries: &[StashEntry]) -> Vec<TreeNode> {
    let mut root = TreeNode::dir("root".to_string(), String::new());

    for entry in entries {
        let path = entry.path();
        insert_path(&mut root, path);
    }

    root.sort_children();
    root.children
}

/// Insert a path into the tree, creating intermediate directories as needed.
fn insert_path(root: &mut TreeNode, path: &str) {
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        return;
    }

    let mut current = root;
    let mut current_path = String::new();

    for (i, part) in parts.iter().enumerate() {
        let is_last = i == parts.len() - 1;

        if !current_path.is_empty() {
            current_path.push('/');
        }
        current_path.push_str(part);

        // Check if child already exists
        let child_idx = current.children.iter().position(|c| c.name == *part);

        if let Some(idx) = child_idx {
            current = &mut current.children[idx];
        } else {
            let new_node = if is_last {
                TreeNode::file(part.to_string(), current_path.clone())
            } else {
                TreeNode::dir(part.to_string(), current_path.clone())
            };
            current.children.push(new_node);
            current = current.children.last_mut().unwrap();
        }
    }
}

/// Recursively collect all file paths from the tree.
#[allow(dead_code)]
pub fn collect_file_paths(nodes: &[TreeNode]) -> Vec<String> {
    let mut paths = Vec::new();
    for node in nodes {
        if node.is_file {
            paths.push(node.path.clone());
        } else {
            paths.extend(collect_file_paths(&node.children));
        }
    }
    paths
}

/// Find a node by its full path.
#[allow(dead_code)]
pub fn find_node<'a>(nodes: &'a [TreeNode], path: &str) -> Option<&'a TreeNode> {
    for node in nodes {
        if node.path == path {
            return Some(node);
        }
        if !node.is_file
            && let Some(found) = find_node(&node.children, path)
        {
            return Some(found);
        }
    }
    None
}

/// Render the tree view widget.
///
/// Displays expandable/collapsible directory nodes and selectable file nodes.
///
/// Returns `true` if the selection or expansion changed.
pub fn show(
    ui: &mut egui::Ui,
    nodes: &[TreeNode],
    expanded: &mut HashSet<String>,
    selected: &mut Option<String>,
) -> bool {
    let mut changed = false;

    for node in nodes {
        render_node(ui, node, expanded, selected, 0, &mut changed);
    }

    changed
}

fn render_node(
    ui: &mut egui::Ui,
    node: &TreeNode,
    expanded: &mut HashSet<String>,
    selected: &mut Option<String>,
    depth: usize,
    changed: &mut bool,
) {
    let indent = depth as f32 * 16.0;

    ui.horizontal(|ui| {
        ui.add_space(indent);

        if node.is_file {
            // File node: clickable
            let is_selected = selected.as_ref() == Some(&node.path);
            let response = ui.selectable_label(is_selected, format!("📄 {}", node.name));
            if response.clicked() {
                *selected = Some(node.path.clone());
                *changed = true;
            }
        } else {
            // Directory node: expandable
            let is_expanded = expanded.contains(&node.path);
            let arrow = if is_expanded { "▼" } else { "▶" };

            let response = ui.button(format!("{}  {}", arrow, node.name));
            if response.clicked() {
                if is_expanded {
                    expanded.remove(&node.path);
                } else {
                    expanded.insert(node.path.clone());
                }
                *changed = true;
            }

            // Render children if expanded
            if is_expanded {
                for child in &node.children {
                    render_node(ui, child, expanded, selected, depth + 1, changed);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(path: &str, size: u64) -> StashEntry {
        StashEntry::new(
            uuid::Uuid::new_v4(),
            path.to_string(),
            0,
            0,
            size,
            "application/octet-stream".to_string(),
            vec![],
        )
    }

    #[test]
    fn test_empty_entries() {
        let tree = build_tree(&[]);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_single_file_at_root() {
        let entries = vec![make_entry("readme.txt", 100)];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "readme.txt");
        assert!(tree[0].is_file);
        assert_eq!(tree[0].path, "readme.txt");
    }

    #[test]
    fn test_file_in_subdirectory() {
        let entries = vec![make_entry("docs/report.pdf", 200)];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "docs");
        assert!(!tree[0].is_file);
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].name, "report.pdf");
        assert!(tree[0].children[0].is_file);
        assert_eq!(tree[0].children[0].path, "docs/report.pdf");
    }

    #[test]
    fn test_multiple_files_same_directory() {
        let entries = vec![
            make_entry("docs/report.pdf", 200),
            make_entry("docs/notes.txt", 50),
        ];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn test_deeply_nested_path() {
        let entries = vec![make_entry("a/b/c/d/file.txt", 10)];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 1); // a
        assert_eq!(tree[0].children.len(), 1); // b
        assert_eq!(tree[0].children[0].children.len(), 1); // c
        assert_eq!(tree[0].children[0].children[0].children.len(), 1); // d
        assert_eq!(
            tree[0].children[0].children[0].children[0].children.len(),
            1
        ); // file.txt
        assert_eq!(
            tree[0].children[0].children[0].children[0].children[0].name,
            "file.txt"
        );
    }

    #[test]
    fn test_shared_parent_not_duplicated() {
        let entries = vec![
            make_entry("docs/a.txt", 10),
            make_entry("docs/b.txt", 20),
            make_entry("docs/c.txt", 30),
        ];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 1); // single "docs" dir
        assert_eq!(tree[0].children.len(), 3); // 3 files
    }

    #[test]
    fn test_multiple_root_items() {
        let entries = vec![
            make_entry("readme.txt", 10),
            make_entry("src/main.rs", 100),
            make_entry("docs/guide.md", 50),
        ];
        let tree = build_tree(&entries);

        assert_eq!(tree.len(), 3); // readme.txt, src, docs
    }

    #[test]
    fn test_sort_directories_first() {
        let entries = vec![
            make_entry("zebra.txt", 10),
            make_entry("alpha/beta.txt", 20),
            make_entry("docs/readme.md", 30),
        ];
        let tree = build_tree(&entries);

        // build_tree already sorts, so directories should come before files
        let names: Vec<&str> = tree.iter().map(|n| n.name.as_str()).collect();
        // "alpha" and "docs" are dirs, "zebra" is a file
        let alpha_pos = names
            .iter()
            .position(|&n| n == "alpha")
            .expect("alpha not found");
        let docs_pos = names
            .iter()
            .position(|&n| n == "docs")
            .expect("docs not found");
        let zebra_pos = names
            .iter()
            .position(|&n| n == "zebra.txt")
            .expect("zebra.txt not found");
        assert!(alpha_pos < zebra_pos, "alpha should come before zebra");
        assert!(docs_pos < zebra_pos, "docs should come before zebra");
    }

    #[test]
    fn test_collect_file_paths() {
        let entries = vec![
            make_entry("a.txt", 10),
            make_entry("docs/b.txt", 20),
            make_entry("docs/c.txt", 30),
        ];
        let tree = build_tree(&entries);
        let paths = collect_file_paths(&tree);

        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&"a.txt".to_string()));
        assert!(paths.contains(&"docs/b.txt".to_string()));
        assert!(paths.contains(&"docs/c.txt".to_string()));
    }

    #[test]
    fn test_find_node_existing() {
        let entries = vec![
            make_entry("docs/report.pdf", 100),
            make_entry("src/main.rs", 50),
        ];
        let tree = build_tree(&entries);

        assert!(find_node(&tree, "docs").is_some());
        assert!(find_node(&tree, "docs/report.pdf").is_some());
        assert!(find_node(&tree, "src/main.rs").is_some());
    }

    #[test]
    fn test_find_node_nonexistent() {
        let entries = vec![make_entry("docs/report.pdf", 100)];
        let tree = build_tree(&entries);

        assert!(find_node(&tree, "nonexistent").is_none());
        assert!(find_node(&tree, "docs/missing.txt").is_none());
    }

    #[test]
    fn test_empty_path_parts_ignored() {
        let entries = vec![make_entry("/leading/slash.txt", 10)];
        let tree = build_tree(&entries);

        // Should not create an empty root node
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "leading");
    }
}
