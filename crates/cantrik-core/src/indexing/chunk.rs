use std::path::Path;

use tree_sitter::{Parser, Query, QueryCursor, StreamingIterator};

use super::SourceChunk;

#[derive(Clone, Copy)]
pub(super) enum Lang {
    Rust,
    Python,
    Javascript,
    Typescript,
    Tsx,
    Go,
    Java,
    C,
    Cpp,
    Php,
    Ruby,
    Sql,
    Toml,
    Json,
    Yaml,
    Markdown,
    Bash,
    Css,
    Html,
    Makefile,
    Scala,
}

impl Lang {
    fn as_str(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::Javascript => "javascript",
            Lang::Typescript => "typescript",
            Lang::Tsx => "tsx",
            Lang::Go => "go",
            Lang::Java => "java",
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Php => "php",
            Lang::Ruby => "ruby",
            Lang::Sql => "sql",
            Lang::Toml => "toml",
            Lang::Json => "json",
            Lang::Yaml => "yaml",
            Lang::Markdown => "markdown",
            Lang::Bash => "bash",
            Lang::Css => "css",
            Lang::Html => "html",
            Lang::Makefile => "makefile",
            Lang::Scala => "scala",
        }
    }

    pub(super) fn language(self) -> tree_sitter::Language {
        match self {
            Lang::Rust => tree_sitter_rust::LANGUAGE.into(),
            Lang::Python => tree_sitter_python::LANGUAGE.into(),
            Lang::Javascript => tree_sitter_javascript::LANGUAGE.into(),
            Lang::Typescript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Lang::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
            Lang::Go => tree_sitter_go::LANGUAGE.into(),
            Lang::Java => tree_sitter_java::LANGUAGE.into(),
            Lang::C => tree_sitter_c::LANGUAGE.into(),
            Lang::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Lang::Php => tree_sitter_php::LANGUAGE_PHP.into(),
            Lang::Ruby => tree_sitter_ruby::LANGUAGE.into(),
            Lang::Sql => tree_sitter_sequel::LANGUAGE.into(),
            Lang::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
            Lang::Json => tree_sitter_json::LANGUAGE.into(),
            Lang::Yaml => tree_sitter_yaml::LANGUAGE.into(),
            Lang::Markdown => tree_sitter_md::LANGUAGE.into(),
            Lang::Bash => tree_sitter_bash::LANGUAGE.into(),
            Lang::Css => tree_sitter_css::LANGUAGE.into(),
            Lang::Html => tree_sitter_html::LANGUAGE.into(),
            Lang::Makefile => tree_sitter_make::LANGUAGE.into(),
            Lang::Scala => tree_sitter_scala::LANGUAGE.into(),
        }
    }

    fn chunk_query(self) -> &'static str {
        match self {
            Lang::Rust => {
                r"
(function_item name: (identifier) @name) @chunk
(struct_item name: (type_identifier) @name) @chunk
(enum_item name: (type_identifier) @name) @chunk
(union_item name: (type_identifier) @name) @chunk
"
            }
            Lang::Python => {
                r"
(function_definition name: (identifier) @name) @chunk
(class_definition name: (identifier) @name) @chunk
"
            }
            Lang::Javascript => {
                r"
(function_declaration name: (identifier) @name) @chunk
(class_declaration name: (identifier) @name) @chunk
(method_definition name: (property_identifier) @name) @chunk
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(arrow_function) (function_expression)])) @chunk
"
            }
            Lang::Typescript | Lang::Tsx => {
                r"
(function_declaration name: (identifier) @name) @chunk
(class_declaration name: (type_identifier) @name) @chunk
(method_definition name: (property_identifier) @name) @chunk
(method_definition name: (private_property_identifier) @name) @chunk
(lexical_declaration
  (variable_declarator
    name: (identifier) @name
    value: [(arrow_function) (function_expression)])) @chunk
"
            }
            Lang::Go => {
                r"
(function_declaration name: (identifier) @name) @chunk
(method_declaration name: (field_identifier) @name) @chunk
"
            }
            Lang::Java => {
                r"
(method_declaration name: (identifier) @name) @chunk
(class_declaration name: (identifier) @name) @chunk
(interface_declaration name: (identifier) @name) @chunk
(constructor_declaration name: (identifier) @name) @chunk
"
            }
            Lang::C => {
                r"
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @chunk
"
            }
            Lang::Cpp => {
                r"
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @chunk
(function_definition
  declarator: (function_declarator
    declarator: (field_identifier) @name)) @chunk
(function_definition
  declarator: (function_declarator
    declarator: (qualified_identifier name: (identifier) @name))) @chunk
(class_specifier name: (type_identifier) @name) @chunk
(struct_specifier name: (type_identifier) @name) @chunk
"
            }
            Lang::Php => {
                r"
(function_definition name: (name) @name) @chunk
(class_declaration name: (name) @name) @chunk
(method_declaration name: (name) @name) @chunk
"
            }
            Lang::Ruby => {
                r"
(method name: (simple_identifier) @name) @chunk
(singleton_method name: (simple_identifier) @name) @chunk
(class name: (constant) @name) @chunk
(module name: (constant) @name) @chunk
"
            }
            Lang::Sql => {
                // Coarse: one chunk per top-level statement (name = first line / kind).
                r"(statement) @chunk"
            }
            Lang::Toml => {
                r"
(table (bare_key) @name) @chunk
(pair (bare_key) @name) @chunk
(table (dotted_key) @name) @chunk
(pair (dotted_key) @name) @chunk
"
            }
            Lang::Json => r"(pair key: (string) @name) @chunk",
            Lang::Yaml => {
                r"
(block_mapping_pair (flow_node) @name) @chunk
(block_mapping_pair (block_mapping_pair_key) @name) @chunk
"
            }
            Lang::Markdown => {
                r"
(atx_heading (heading_content) @name) @chunk
(setext_heading (heading_content) @name) @chunk
(fenced_code_block) @chunk
"
            }
            Lang::Bash => {
                r"
(function_definition name: (word) @name) @chunk
"
            }
            Lang::Css => {
                r"
(rule_set
  (selectors (class_selector (class_name) @name))) @chunk
(rule_set
  (selectors (id_selector (id_name) @name))) @chunk
"
            }
            Lang::Html => {
                r"
(element (start_tag (tag_name) @name)) @chunk
(script_element (start_tag (tag_name) @name)) @chunk
(style_element (start_tag (tag_name) @name)) @chunk
"
            }
            Lang::Makefile => {
                r"
(rule (targets (word) @name)) @chunk
(variable_assignment name: (word) @name) @chunk
"
            }
            Lang::Scala => {
                r"
(class_definition name: (_) @name) @chunk
(object_definition name: (_) @name) @chunk
(trait_definition name: (_) @name) @chunk
(function_definition name: (_) @name) @chunk
"
            }
        }
    }

    /// `true` if query uses optional `@name` (still may appear in some patterns).
    fn name_capture_optional(self) -> bool {
        matches!(self, Lang::Sql | Lang::Markdown)
    }
}

pub(super) fn detect_language(rel_path: &str) -> Option<Lang> {
    let path = Path::new(rel_path);
    if let Some(base) = path.file_name().and_then(|n| n.to_str())
        && base.eq_ignore_ascii_case("makefile")
    {
        return Some(Lang::Makefile);
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())?
        .to_ascii_lowercase();
    Some(match ext.as_str() {
        "rs" => Lang::Rust,
        "py" => Lang::Python,
        "js" | "mjs" | "cjs" => Lang::Javascript,
        "ts" | "mts" | "cts" => Lang::Typescript,
        "tsx" => Lang::Tsx,
        "go" => Lang::Go,
        "java" => Lang::Java,
        "c" | "h" => Lang::C,
        "cc" | "cpp" | "cxx" | "hpp" | "hxx" => Lang::Cpp,
        "php" => Lang::Php,
        "rb" => Lang::Ruby,
        "sql" => Lang::Sql,
        "toml" => Lang::Toml,
        "json" => Lang::Json,
        "yaml" | "yml" => Lang::Yaml,
        "md" | "markdown" => Lang::Markdown,
        "sh" | "bash" => Lang::Bash,
        "css" => Lang::Css,
        "html" | "htm" => Lang::Html,
        "mk" => Lang::Makefile,
        "scala" | "sc" => Lang::Scala,
        _ => return None,
    })
}

fn capture_u32(query: &Query, name: &str) -> Option<u32> {
    query
        .capture_names()
        .iter()
        .position(|&n| n == name)
        .map(|i| i as u32)
}

/// Returns `(language_id, chunks)` or `None` if file cannot be parsed as UTF-8 or language unsupported.
pub fn extract_chunks(rel_path: &str, bytes: &[u8]) -> Option<(String, Vec<SourceChunk>)> {
    let src = std::str::from_utf8(bytes).ok()?;
    let lang = detect_language(rel_path)?;
    let ts_lang = lang.language();
    let mut parser = Parser::new();
    parser.set_language(&ts_lang).ok()?;
    let tree = parser.parse(src, None)?;
    let root = tree.root_node();

    let mut chunks = Vec::new();

    if let Ok(query) = Query::new(&ts_lang, lang.chunk_query())
        && let Some(capture_chunk) = capture_u32(&query, "chunk")
    {
        let capture_name = capture_u32(&query, "name");
        let name_optional = lang.name_capture_optional() || capture_name.is_none();

        let mut cursor = QueryCursor::new();
        let mut qmatches = cursor.matches(&query, root, src.as_bytes());
        loop {
            qmatches.advance();
            let Some(m) = qmatches.get() else {
                break;
            };
            let mut name_node = None;
            let mut chunk_node = None;
            for c in m.captures {
                if capture_name == Some(c.index) {
                    name_node = Some(c.node);
                } else if c.index == capture_chunk {
                    chunk_node = Some(c.node);
                }
            }
            let Some(chunk_n) = chunk_node else {
                continue;
            };
            if !name_optional && capture_name.is_some() && name_node.is_none() {
                continue;
            }
            let symbol = if let Some(name_n) = name_node {
                name_n
                    .utf8_text(src.as_bytes())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| fallback_symbol(&chunk_n))
            } else {
                fallback_symbol(&chunk_n)
            };
            if symbol.is_empty() {
                continue;
            }
            let kind = chunk_n.kind().to_string();
            let start = chunk_n.start_byte();
            let end = chunk_n.end_byte();
            let pos = chunk_n.start_position();
            let source = src.get(start..end).unwrap_or("").to_string();
            chunks.push(SourceChunk {
                path: rel_path.to_string(),
                language: lang.as_str().to_string(),
                symbol,
                kind,
                start_byte: start,
                end_byte: end,
                start_row: pos.row,
                start_col: pos.column,
                source,
            });
        }
    }

    if chunks.is_empty() {
        let end = src.len();
        chunks.push(SourceChunk {
            path: rel_path.to_string(),
            language: lang.as_str().to_string(),
            symbol: "<file>".to_string(),
            kind: "translation_unit".to_string(),
            start_byte: 0,
            end_byte: end,
            start_row: 0,
            start_col: 0,
            source: src.to_string(),
        });
    }

    Some((lang.as_str().to_string(), chunks))
}

fn fallback_symbol(chunk_n: &tree_sitter::Node<'_>) -> String {
    format!("{}:{}", chunk_n.kind(), chunk_n.start_position().row + 1)
}

/// Parse `src` for call-graph extraction (reuses same languages as chunking).
pub(super) fn parse_tree(rel_path: &str, src: &str) -> Option<(tree_sitter::Tree, Lang)> {
    let lang = detect_language(rel_path)?;
    let ts_lang = lang.language();
    let mut parser = Parser::new();
    parser.set_language(&ts_lang).ok()?;
    let tree = parser.parse(src, None)?;
    Some((tree, lang))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_rust_fn_named_main() {
        let src = "fn main() {}\n";
        let (_, chunks) = extract_chunks("x.rs", src.as_bytes()).unwrap();
        assert!(chunks.iter().any(|c| c.symbol == "main"));
    }

    #[test]
    fn chunk_python_def() {
        let src = "def foo():\n    pass\n";
        let (_, chunks) = extract_chunks("m.py", src.as_bytes()).unwrap();
        assert!(chunks.iter().any(|c| c.symbol == "foo"));
    }

    #[test]
    fn chunk_java_class() {
        let src = "class Foo { void bar() {} }\n";
        let (_, chunks) = extract_chunks("X.java", src.as_bytes()).unwrap();
        assert!(
            chunks
                .iter()
                .any(|c| c.symbol == "Foo" || c.symbol == "bar")
        );
    }

    #[test]
    fn chunk_bash_function() {
        let src = "foo() { echo hi; }\n";
        let (_, chunks) = extract_chunks("x.sh", src.as_bytes()).unwrap();
        assert!(chunks.iter().any(|c| c.symbol == "foo"));
    }

    #[test]
    fn chunk_css_class_rule() {
        let src = ".btn { color: red; }\n";
        let (_, chunks) = extract_chunks("x.css", src.as_bytes()).unwrap();
        assert!(chunks.iter().any(|c| c.symbol == "btn"));
    }

    #[test]
    fn chunk_json_pair() {
        let src = r#"{"a": 1}"#;
        let (_, chunks) = extract_chunks("x.json", src.as_bytes()).unwrap();
        assert!(chunks.iter().any(|c| c.symbol.contains('a')));
    }

    #[test]
    fn chunk_html_tag() {
        let src = "<div><p>hi</p></div>\n";
        let (_, chunks) = extract_chunks("x.html", src.as_bytes()).unwrap();
        assert!(
            chunks.iter().any(|c| c.symbol == "div" || c.symbol == "p"),
            "chunks: {chunks:?}"
        );
    }

    #[test]
    fn chunk_makefile_rule_and_var() {
        let src = "all:\n\t@echo hi\n\nFOO := 1\n";
        let (_, chunks) = extract_chunks("Makefile", src.as_bytes()).unwrap();
        assert!(
            chunks
                .iter()
                .any(|c| c.symbol == "all" || c.symbol == "FOO"),
            "chunks: {chunks:?}"
        );
    }

    #[test]
    fn chunk_scala_object_and_def() {
        let src = "object Foo {\n  def bar(): Unit = ()\n}\n";
        let (_, chunks) = extract_chunks("x.scala", src.as_bytes()).unwrap();
        assert!(
            chunks
                .iter()
                .any(|c| c.symbol == "Foo" || c.symbol == "bar"),
            "chunks: {chunks:?}"
        );
    }
}
