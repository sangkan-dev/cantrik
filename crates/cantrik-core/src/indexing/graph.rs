use std::collections::HashMap;
use std::path::Path;

use tree_sitter::Node;

use super::chunk::parse_tree;
use super::{CallEdge, SourceChunk};

pub(crate) fn extract_edges(project_root: &Path, chunks: &[SourceChunk]) -> Vec<CallEdge> {
    let mut by_path: HashMap<String, Vec<&SourceChunk>> = HashMap::new();
    for c in chunks {
        by_path.entry(c.path.clone()).or_default().push(c);
    }

    let mut edges = Vec::new();

    for (rel, path_chunks) in by_path {
        let full = project_root.join(&rel);
        let Ok(bytes) = std::fs::read(&full) else {
            continue;
        };
        let Ok(src) = std::str::from_utf8(&bytes) else {
            continue;
        };
        let Some((tree, _lang)) = parse_tree(&rel, src) else {
            continue;
        };
        let root = tree.root_node();

        for ch in path_chunks {
            let Some(node) = root.descendant_for_byte_range(ch.start_byte, ch.end_byte) else {
                continue;
            };
            collect_call_edges(node, src, &rel, &ch.symbol, &mut edges);
        }
    }

    edges.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.caller.cmp(&b.caller))
            .then_with(|| a.callee.cmp(&b.callee))
    });
    edges
}

fn collect_call_edges(
    root: Node<'_>,
    source: &str,
    path: &str,
    caller: &str,
    out: &mut Vec<CallEdge>,
) {
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        if matches!(
            node.kind(),
            "call_expression"
                | "call"
                | "function_call_expression"
                | "command_call"
                | "decorated_call"
        ) && let Some(callee) = callee_from_invocation(node, source)
        {
            let line = node.start_position().row + 1;
            out.push(CallEdge {
                path: path.to_string(),
                caller: caller.to_string(),
                callee,
                line,
            });
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
}

fn callee_from_invocation(call: Node<'_>, source: &str) -> Option<String> {
    let callee = call
        .child_by_field_name("function")
        .or_else(|| call.child_by_field_name("method"))
        .or_else(|| call.child_by_field_name("name"))?;
    callee_label(callee, source)
}

fn callee_label(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "identifier"
        | "type_identifier"
        | "property_identifier"
        | "field_identifier"
        | "private_property_identifier" => {
            Some(node.utf8_text(source.as_bytes()).ok()?.to_string())
        }
        "field_expression" => {
            let field = node.child_by_field_name("field")?;
            callee_label(field, source)
        }
        "scoped_identifier" => {
            let name = node.child_by_field_name("name")?;
            callee_label(name, source)
        }
        "parenthesized_expression" | "subscript_expression" => {
            node.named_child(0).and_then(|c| callee_label(c, source))
        }
        "member_expression" => {
            let prop = node.child_by_field_name("property")?;
            callee_label(prop, source)
        }
        "attribute" => {
            let attr = node.child_by_field_name("attribute")?;
            callee_label(attr, source)
        }
        _ => {
            let t = node.utf8_text(source.as_bytes()).ok()?;
            let short: String = t.chars().take(64).collect();
            if short.is_empty() { None } else { Some(short) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexing::chunk::extract_chunks;

    #[test]
    fn edges_found_in_rust_fn() {
        let src = "fn a() {}\nfn b() { a(); }\n";
        let dir = std::env::temp_dir().join(format!(
            "cantrik-graph-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("t.rs"), src).unwrap();
        let chunks = extract_chunks("t.rs", src.as_bytes()).unwrap().1;
        let edges = extract_edges(&dir, &chunks);
        assert!(
            edges.iter().any(|e| e.callee == "a" && e.caller == "b"),
            "{edges:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
