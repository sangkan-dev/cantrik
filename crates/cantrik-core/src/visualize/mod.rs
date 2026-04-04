//! Mermaid diagrams from index / repo layout / cargo tree (Sprint 18, PRD §4.17).

mod mermaid;

pub use mermaid::{
    VisualizeError, VisualizeKind, mermaid_architecture_top_dirs, mermaid_callgraph,
    mermaid_dependencies_from_cargo_tree_stdout, render_visualize_kind,
};
