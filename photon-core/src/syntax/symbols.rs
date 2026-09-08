//! Syntax-based document symbols fallback.
//!
//! The outline tree is normally fed by the language server, which means it
//! stays empty for every language without a running server. This module
//! builds equivalent symbols from the tree-sitter parse tree instead, so the
//! tree works for every language with a grammar and zero setup.
//!
//! Patterns are compiled and run independently per pattern: if one pattern
//! does not match a grammar version, only that construct is lost, never the
//! whole language.

use lapce_xi_rope::Rope;
use lsp_types::{DocumentSymbol, Range, SymbolKind};
use tree_sitter::{Query, QueryCursor};

use super::util::RopeProvider;
use crate::{
    language::PhotonLanguage, rope_text_pos::RopeTextPosition,
};

/// Patterns per language. Each captures:
/// - `@def` the definition node,
/// - `@name` the node the display name is read from,
/// - one of `@function` `@method` `@class` `@struct` `@interface` `@enum`
///   `@module` `@const` `@type` for the symbol kind.
fn patterns(language: PhotonLanguage) -> &'static [&'static str] {
    match language {
        PhotonLanguage::Rust => &[
            r#"(function_item name: (identifier) @name) @def @function"#,
            r#"(struct_item name: (type_identifier) @name) @def @struct"#,
            r#"(enum_item name: (type_identifier) @name) @def @enum"#,
            r#"(trait_item name: (type_identifier) @name) @def @interface"#,
            r#"(mod_item name: (identifier) @name) @def @module"#,
            r#"(const_item name: (identifier) @name) @def @const"#,
            r#"(static_item name: (identifier) @name) @def @const"#,
            r#"(type_item name: (type_identifier) @name) @def @type"#,
            r#"(impl_item type: (type_identifier) @name) @def @class"#,
        ],
        PhotonLanguage::Go => &[
            r#"(function_declaration name: (identifier) @name) @def @function"#,
            r#"(method_declaration name: (field_identifier) @name) @def @method"#,
            r#"(type_declaration (type_spec name: (type_identifier) @name)) @def @type"#,
        ],
        PhotonLanguage::Python => &[
            r#"(function_definition name: (identifier) @name) @def @function"#,
            r#"(class_definition name: (identifier) @name) @def @class"#,
        ],
        PhotonLanguage::Javascript => &[
            r#"(function_declaration name: (identifier) @name) @def @function"#,
            r#"(class_declaration name: (identifier) @name) @def @class"#,
            r#"(method_definition name: [(identifier) (property_identifier) (private_property_identifier)] @name) @def @method"#,
        ],
        PhotonLanguage::Typescript => &[
            r#"(function_declaration name: (identifier) @name) @def @function"#,
            r#"(class_declaration name: (identifier) @name) @def @class"#,
            r#"(method_definition name: [(identifier) (property_identifier) (private_property_identifier)] @name) @def @method"#,
            r#"(interface_declaration name: (type_identifier) @name) @def @interface"#,
            r#"(type_alias_declaration name: (type_identifier) @name) @def @type"#,
            r#"(enum_declaration name: (identifier) @name) @def @enum"#,
        ],
        PhotonLanguage::C => &[
            r#"(function_definition declarator: (function_declarator declarator: (identifier) @name)) @def @function"#,
            r#"(struct_specifier name: (type_identifier) @name) @def @struct"#,
            r#"(enum_specifier name: (type_identifier) @name) @def @enum"#,
            r#"(union_specifier name: (type_identifier) @name) @def @struct"#,
        ],
        PhotonLanguage::Cpp => &[
            r#"(function_definition declarator: (function_declarator declarator: (identifier) @name)) @def @function"#,
            r#"(struct_specifier name: (type_identifier) @name) @def @struct"#,
            r#"(enum_specifier name: (type_identifier) @name) @def @enum"#,
            r#"(union_specifier name: (type_identifier) @name) @def @struct"#,
            r#"(class_specifier name: (type_identifier) @name) @def @class"#,
            r#"(namespace_definition name: (namespace_identifier) @name) @def @module"#,
        ],
        PhotonLanguage::Lua => {
            &[r#"(function_declaration name: (_) @name) @def @function"#]
        }
        PhotonLanguage::Java => &[
            r#"(class_declaration name: (identifier) @name) @def @class"#,
            r#"(method_declaration name: (identifier) @name) @def @method"#,
            r#"(interface_declaration name: (identifier) @name) @def @interface"#,
            r#"(enum_declaration name: (identifier) @name) @def @enum"#,
            r#"(constructor_declaration name: (identifier) @name) @def @method"#,
        ],
        PhotonLanguage::Ruby => &[
            r#"(method name: (identifier) @name) @def @method"#,
            r#"(class name: (constant) @name) @def @class"#,
            r#"(module name: (constant) @name) @def @module"#,
        ],
        PhotonLanguage::Php => &[
            r#"(function_definition name: (name) @name) @def @function"#,
            r#"(class_declaration name: (name) @name) @def @class"#,
            r#"(method_declaration name: (name) @name) @def @method"#,
            r#"(interface_declaration name: (name) @name) @def @interface"#,
        ],
        PhotonLanguage::Swift => &[
            r#"(function_declaration name: (simple_identifier) @name) @def @function"#,
            r#"(class_declaration name: (type_identifier) @name) @def @class"#,
            r#"(struct_declaration name: (type_identifier) @name) @def @struct"#,
            r#"(enum_declaration name: (type_identifier) @name) @def @enum"#,
            r#"(protocol_declaration name: (type_identifier) @name) @def @interface"#,
        ],
        PhotonLanguage::Kotlin => &[
            r#"(function_declaration (simple_identifier) @name) @def @function"#,
            r#"(class_declaration (type_identifier) @name) @def @class"#,
        ],
        PhotonLanguage::Scala => &[
            r#"(function_definition (identifier) @name) @def @function"#,
            r#"(class_definition (identifier) @name) @def @class"#,
            r#"(object_definition (identifier) @name) @def @class"#,
            r#"(trait_definition (identifier) @name) @def @interface"#,
        ],
        PhotonLanguage::Zig => &[
            r#"(Decl (FnProto function: (IDENTIFIER) @name)) @def @function"#,
            r#"(TestDecl (STRINGLITERALSINGLE) @name) @def @function"#,
        ],
        PhotonLanguage::Csharp => &[
            r#"(class_declaration name: (identifier) @name) @def @class"#,
            r#"(method_declaration name: (identifier) @name) @def @method"#,
            r#"(interface_declaration name: (identifier) @name) @def @interface"#,
            r#"(struct_declaration name: (identifier) @name) @def @struct"#,
            r#"(enum_declaration name: (identifier) @name) @def @enum"#,
        ],
        PhotonLanguage::Bash => {
            &[r#"(function_definition name: (word) @name) @def @function"#]
        }
        _ => &[],
    }
}

fn kind_of(capture_names: &[&str]) -> Option<SymbolKind> {
    if capture_names.contains(&"function") {
        Some(SymbolKind::FUNCTION)
    } else if capture_names.contains(&"method") {
        Some(SymbolKind::METHOD)
    } else if capture_names.contains(&"class") {
        Some(SymbolKind::CLASS)
    } else if capture_names.contains(&"struct") {
        Some(SymbolKind::STRUCT)
    } else if capture_names.contains(&"interface") {
        Some(SymbolKind::INTERFACE)
    } else if capture_names.contains(&"enum") {
        Some(SymbolKind::ENUM)
    } else if capture_names.contains(&"module") {
        Some(SymbolKind::MODULE)
    } else if capture_names.contains(&"const") {
        Some(SymbolKind::CONSTANT)
    } else if capture_names.contains(&"type") {
        Some(SymbolKind::TYPE_PARAMETER)
    } else {
        None
    }
}

struct RawSymbol {
    name: String,
    kind: SymbolKind,
    start: usize,
    end: usize,
    name_start: usize,
    name_end: usize,
}

fn text_of(text: &str, start: usize, end: usize) -> Option<String> {
    if start > end || end > text.len() {
        return None;
    }
    if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return None;
    }
    Some(text[start..end].to_string())
}

/// Build outline symbols from an already-parsed tree.
///
/// Never panics: stale trees (offsets past the buffer, split characters)
/// simply yield fewer symbols.
pub fn document_symbols(
    language: PhotonLanguage,
    tree: &tree_sitter::Tree,
    rope: &Rope,
) -> Vec<DocumentSymbol> {
    let patterns = patterns(language);
    if patterns.is_empty() {
        return Vec::new();
    }

    let root = tree.root_node();
    // Contiguous text for name extraction (queries themselves run zero-copy
    // straight on the rope). Stale trees may disagree with the buffer, so
    // every offset below is validated.
    let text = rope.slice_to_cow(..);
    let mut raw: Vec<RawSymbol> = Vec::new();

    for pattern_str in patterns {
        let query = match Query::new(&tree.language(), pattern_str) {
            Ok(query) => query,
            Err(_) => continue,
        };
        let mut cursor = QueryCursor::new();
        for (m, _) in cursor.captures(&query, root, RopeProvider(rope)) {
            let mut def = None;
            let mut name_node = None;
            let mut kinds = Vec::new();
            for capture in m.captures {
                let name =
                    query.capture_names()[capture.index as usize].to_string();
                match name.as_str() {
                    "def" => def = Some(capture.node),
                    "name" => name_node = Some(capture.node),
                    _ => kinds.push(name),
                }
            }
            let (Some(def), Some(name_node)) = (def, name_node) else {
                continue;
            };
            let Some(kind) = kind_of(
                &kinds.iter().map(String::as_str).collect::<Vec<_>>(),
            ) else {
                continue;
            };
            let (start, end) = (def.start_byte(), def.end_byte());
            let (name_start, name_end) =
                (name_node.start_byte(), name_node.end_byte());
            let Some(mut name) = text_of(&text, name_start, name_end)
            else {
                continue;
            };
            name = name.trim().to_string();
            // Test names and the like come quoted; show them bare.
            if name.len() >= 2
                && name.starts_with('"')
                && name.ends_with('"')
            {
                name = name[1..name.len() - 1].to_string();
            }
            if name.is_empty() {
                continue;
            }
            // Same span twice (overlapping patterns): keep the first.
            if raw.iter().any(|s| s.start == start && s.end == end) {
                continue;
            }
            raw.push(RawSymbol {
                name,
                kind,
                start,
                end,
                name_start,
                name_end,
            });
        }
    }

    // Document order, parents (larger span) before children.
    raw.sort_by(|a, b| a.start.cmp(&b.start).then(b.end.cmp(&a.end)));

    // Nest by strict containment.
    let mut children: Vec<Vec<usize>> =
        (0..raw.len()).map(|_| Vec::new()).collect();
    let mut roots = Vec::new();
    for (i, symbol) in raw.iter().enumerate() {
        let mut parent = None;
        let mut parent_span = None;
        for (j, other) in raw.iter().enumerate().take(i) {
            if other.start <= symbol.start
                && symbol.end <= other.end
                && (other.start < symbol.start || other.end > symbol.end)
            {
                let span = other.end - other.start;
                if parent_span.is_none_or(|best| span < best) {
                    parent = Some(j);
                    parent_span = Some(span);
                }
            }
        }
        match parent {
            Some(j) => children[j].push(i),
            None => roots.push(i),
        }
    }

    fn build(
        raw: &[RawSymbol],
        children: &[Vec<usize>],
        rope: &Rope,
        index: usize,
    ) -> DocumentSymbol {
        let symbol = &raw[index];
        let position = |offset: usize| {
            let offset = offset.min(rope.len());
            rope.offset_to_position(offset)
        };
        #[allow(deprecated)]
        DocumentSymbol {
            name: symbol.name.clone(),
            detail: None,
            kind: symbol.kind,
            tags: None,
            deprecated: None,
            range: Range {
                start: position(symbol.start),
                end: position(symbol.end),
            },
            selection_range: Range {
                start: position(symbol.name_start),
                end: position(symbol.name_end),
            },
            children: if children[index].is_empty() {
                None
            } else {
                Some(
                    children[index]
                        .iter()
                        .map(|&child| build(raw, children, rope, child))
                        .collect(),
                )
            },
        }
    }

    roots
        .into_iter()
        .map(|index| build(&raw, &children, rope, index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn grammars_dir() -> Option<PathBuf> {
        std::env::var("PHOTON_TEST_GRAMMARS_DIR")
            .ok()
            .map(PathBuf::from)
    }

    fn load_language(grammar: &str) -> Option<tree_sitter::Language> {
        let dir = grammars_dir()?;
        let path = dir.join(format!(
            "libtree-sitter-{grammar}.{}",
            std::env::consts::DLL_EXTENSION
        ));
        let library = unsafe { libloading::Library::new(&path).ok()? };
        let symbol_name = format!("tree_sitter_{}", grammar.replace('-', "_"));
        let func: unsafe extern "C" fn() -> tree_sitter::Language = unsafe {
            *library
                .get::<unsafe extern "C" fn() -> tree_sitter::Language>(
                    symbol_name.as_bytes(),
                )
                .ok()?
        };
        // The library must outlive the language; leak it for tests.
        std::mem::forget(library);
        unsafe { Some(func()) }
    }

    fn check(
        language: PhotonLanguage,
        grammar: &str,
        source: &str,
        expected_roots: &[&str],
    ) {
        let Some(grammar_lang) = load_language(grammar) else {
            println!("SKIP {grammar}: no grammar library");
            return;
        };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar_lang).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let rope = Rope::from(source);
        let symbols = document_symbols(language, &tree, &rope);
        let names: Vec<&str> =
            symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            expected_roots,
            "roots for {grammar}\nqueries present: {}",
            !patterns(language).is_empty()
        );
    }

    fn children_of<'a>(
        symbols: &'a [DocumentSymbol],
        name: &str,
        kind: SymbolKind,
    ) -> Vec<&'a str> {
        symbols
            .iter()
            .find(|s| s.name == name && s.kind == kind)
            .map(|s| {
                s.children
                    .as_ref()
                    .map(|c| c.iter().map(|s| s.name.as_str()).collect())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    #[test]
    fn syntax_symbols_match_grammars() {
        if grammars_dir().is_none() {
            println!("SKIP: set PHOTON_TEST_GRAMMARS_DIR to verify queries");
            return;
        }

        let rust_src = "mod m {\nfn f() {}\n}\nstruct S;\nimpl S {\nfn g(&self) {}\n}\nconst C: u8 = 1;\n";
        let Some(grammar_lang) = load_language("rust") else {
            println!("SKIP rust: no grammar library");
            return;
        };
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&grammar_lang).unwrap();
        let tree = parser.parse(rust_src, None).unwrap();
        let rope = Rope::from(rust_src);
        let symbols = document_symbols(PhotonLanguage::Rust, &tree, &rope);
        for s in &symbols {
            println!("rust: {} ({:?})", s.name, s.kind);
            if let Some(children) = &s.children {
                for c in children {
                    println!("  child: {} ({:?})", c.name, c.kind);
                }
            }
        }
        let names: Vec<&str> =
            symbols.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["m", "S", "S", "C"]);
        assert_eq!(children_of(&symbols, "m", SymbolKind::MODULE), ["f"]);
        assert_eq!(
            children_of(&symbols, "S", SymbolKind::CLASS),
            ["g"]
        );

        check(
            PhotonLanguage::Go,
            "go",
            "package main\nfunc F() {}\ntype T struct{}\nfunc (t T) M() {}\n",
            &["F", "T", "M"],
        );
        check(
            PhotonLanguage::Python,
            "python",
            "def f():\n pass\nclass C:\n def m(self):\n  pass\n",
            &["f", "C"],
        );
        check(
            PhotonLanguage::Javascript,
            "javascript",
            "function f() {}\nclass C {\nm() {}\n}\n",
            &["f", "C"],
        );
        check(
            PhotonLanguage::Typescript,
            "tsx",
            "interface I {}\ntype A = string;\nenum E {}\nfunction f() {}\n",
            &["I", "A", "E", "f"],
        );
        check(
            PhotonLanguage::C,
            "c",
            "int f() { return 0; }\nstruct S { int x; };\nenum E { A };\n",
            &["f", "S", "E"],
        );
        check(PhotonLanguage::Lua, "lua", "function f() end\n", &["f"]);
        check(
            PhotonLanguage::Java,
            "java",
            "class C {\nvoid m() {}\nC() {}\n}\ninterface I {}\n",
            &["C", "I"],
        );
        check(
            PhotonLanguage::Ruby,
            "ruby",
            "def f\nend\nclass C\ndef m\nend\nend\nmodule M\nend\n",
            &["f", "C", "M"],
        );
        check(
            PhotonLanguage::Php,
            "php",
            "<?php\nfunction f() {}\nclass C {\nfunction m() {}\n}\n",
            &["f", "C"],
        );
        check(
            PhotonLanguage::Swift,
            "swift",
            "func f() {}\nclass C {\nfunc m() {}\n}\nstruct S {}\nprotocol P {}\n",
            &["f", "C", "S", "P"],
        );
        check(
            PhotonLanguage::Kotlin,
            "kotlin",
            "fun f() {}\nclass C {\nfun m() {}\n}\n",
            &["f", "C"],
        );
        check(
            PhotonLanguage::Scala,
            "scala",
            "def f = 1\nclass C {\ndef m = 2\n}\nobject O\ntrait T\n",
            &["f", "C", "O", "T"],
        );
        check(
            PhotonLanguage::Zig,
            "zig",
            "fn f() void {}\ntest \"t\" {}\n",
            &["f", "t"],
        );
        check(
            PhotonLanguage::Csharp,
            "c-sharp",
            "class C {\nvoid M() {}\n}\ninterface I {}\n",
            &["C", "I"],
        );
        check(PhotonLanguage::Bash, "bash", "foo() {\necho hi\n}\n", &["foo"]);
        check(
            PhotonLanguage::Cpp,
            "cpp",
            "class C {\n};\nnamespace N {\n}\nint f() { return 0; }\n",
            &["C", "N", "f"],
        );
    }
}
