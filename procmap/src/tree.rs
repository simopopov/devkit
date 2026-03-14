use crate::collector::ProcessInfo;
use crate::network::NetConn;
use std::collections::HashMap;

/// A node in the process tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub info: ProcessInfo,
    pub ports: Vec<NetConn>,
    pub children: Vec<TreeNode>,
    pub depth: usize,
}

/// Build the process tree from a flat list of processes and a network map.
/// If `port_range` is Some, only include processes that own a port in that range
/// (plus their ancestors to keep the tree connected).
pub fn build_tree(
    procs: &[ProcessInfo],
    net: &HashMap<u32, Vec<NetConn>>,
    port_range: Option<(u16, u16)>,
) -> Vec<TreeNode> {
    let proc_map: HashMap<u32, &ProcessInfo> = procs.iter().map(|p| (p.pid, p)).collect();
    let mut children_map: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut roots: Vec<u32> = Vec::new();

    for p in procs {
        if p.ppid == 0 || !proc_map.contains_key(&p.ppid) {
            roots.push(p.pid);
        } else {
            children_map.entry(p.ppid).or_default().push(p.pid);
        }
    }

    // Sort roots by PID for stable ordering.
    roots.sort();
    for v in children_map.values_mut() {
        v.sort();
    }

    let mut forest: Vec<TreeNode> = roots
        .iter()
        .map(|&pid| build_subtree(pid, &proc_map, &children_map, net, 0))
        .collect();

    if let Some(range) = port_range {
        forest = forest
            .into_iter()
            .filter_map(|n| prune_tree(n, range))
            .collect();
    }

    forest
}

fn build_subtree(
    pid: u32,
    proc_map: &HashMap<u32, &ProcessInfo>,
    children_map: &HashMap<u32, Vec<u32>>,
    net: &HashMap<u32, Vec<NetConn>>,
    depth: usize,
) -> TreeNode {
    let info = proc_map[&pid].clone();
    let ports = net.get(&pid).cloned().unwrap_or_default();
    let children = children_map
        .get(&pid)
        .map(|kids| {
            kids.iter()
                .map(|&cpid| build_subtree(cpid, proc_map, children_map, net, depth + 1))
                .collect()
        })
        .unwrap_or_default();
    TreeNode {
        info,
        ports,
        children,
        depth,
    }
}

/// Prune the tree: keep only nodes that either have a matching port
/// or have a descendant with a matching port.
fn prune_tree(node: TreeNode, range: (u16, u16)) -> Option<TreeNode> {
    let has_port = node.ports.iter().any(|c| {
        (c.local_port >= range.0 && c.local_port <= range.1)
            || (c.remote_port >= range.0 && c.remote_port <= range.1)
    });

    let pruned_children: Vec<TreeNode> = node
        .children
        .into_iter()
        .filter_map(|c| prune_tree(c, range))
        .collect();

    if has_port || !pruned_children.is_empty() {
        Some(TreeNode {
            info: node.info,
            ports: node.ports,
            children: pruned_children,
            depth: node.depth,
        })
    } else {
        None
    }
}

/// Flatten the tree into a list of (depth, TreeNode-without-children) for rendering.
#[derive(Debug, Clone)]
pub struct FlatRow {
    pub depth: usize,
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu: f32,
    pub mem: f32,
    pub ports: Vec<NetConn>,
    pub has_children: bool,
    pub is_last_sibling: bool,
}

pub fn flatten_tree(forest: &[TreeNode]) -> Vec<FlatRow> {
    let mut rows = Vec::new();
    let len = forest.len();
    for (i, node) in forest.iter().enumerate() {
        flatten_node(node, &mut rows, i == len - 1);
    }
    rows
}

fn flatten_node(node: &TreeNode, rows: &mut Vec<FlatRow>, is_last: bool) {
    rows.push(FlatRow {
        depth: node.depth,
        pid: node.info.pid,
        name: node.info.name.clone(),
        user: node.info.user.clone(),
        cpu: node.info.cpu_percent,
        mem: node.info.mem_percent,
        ports: node.ports.clone(),
        has_children: !node.children.is_empty(),
        is_last_sibling: is_last,
    });
    let len = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        flatten_node(child, rows, i == len - 1);
    }
}

/// Render a static ASCII tree (for --tree mode).
pub fn render_ascii(forest: &[TreeNode]) -> String {
    let mut out = String::new();
    let len = forest.len();
    for (i, node) in forest.iter().enumerate() {
        render_node(node, &mut out, &[], i == len - 1);
    }
    out
}

fn render_node(node: &TreeNode, out: &mut String, prefix_parts: &[bool], is_last: bool) {
    // Build prefix string from ancestor is_last flags.
    let mut prefix = String::new();
    for &ancestor_is_last in prefix_parts {
        if ancestor_is_last {
            prefix.push_str("    ");
        } else {
            prefix.push_str("\u{2502}   ");
        }
    }

    let connector = if prefix_parts.is_empty() {
        ""
    } else if is_last {
        "\u{2514}\u{2500}\u{2500} "
    } else {
        "\u{251c}\u{2500}\u{2500} "
    };

    // Format ports.
    let port_str = format_ports(&node.ports);
    let port_display = if port_str.is_empty() {
        String::new()
    } else {
        format!(" [{}]", port_str)
    };

    out.push_str(&format!(
        "{}{}{} (PID:{} user:{} cpu:{:.1}% mem:{:.1}%){}\n",
        prefix, connector, node.info.name, node.info.pid, node.info.user,
        node.info.cpu_percent, node.info.mem_percent, port_display,
    ));

    let mut next_prefix = prefix_parts.to_vec();
    next_prefix.push(is_last);

    let len = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        render_node(child, out, &next_prefix, i == len - 1);
    }
}

fn format_ports(conns: &[NetConn]) -> String {
    if conns.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = conns
        .iter()
        .map(|c| {
            if c.state == "LISTEN" {
                format!("{}:{} LISTEN", c.proto, c.local_port)
            } else if !c.remote_addr.is_empty() && c.remote_port > 0 {
                format!(
                    "{}:{}->{}:{}",
                    c.proto, c.local_port, c.remote_addr, c.remote_port
                )
            } else {
                format!("{}:{}", c.proto, c.local_port)
            }
        })
        .collect();
    parts.join(", ")
}
