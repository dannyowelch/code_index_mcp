use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

extern "C" {
    fn tree_sitter_cpp() -> Language;
}

#[derive(Debug, Clone)]
pub struct ParsedNode {
    pub kind: String,
    pub name: Option<String>,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub text: String,
}

pub struct TreeSitterParser {
    parser: Parser,
    query_cursor: QueryCursor,
    symbols_query: Query,
    includes_query: Query,
}

impl TreeSitterParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let language = unsafe { tree_sitter_cpp() };
        let mut parser = Parser::new();
        parser.set_language(language)?;

        let symbols_query = Query::new(
            language,
            r#"
            (class_specifier
              name: (type_identifier) @class.name) @class.definition

            (struct_specifier
              name: (type_identifier) @struct.name) @struct.definition

            (function_definition
              declarator: [
                (function_declarator
                  declarator: (identifier) @function.name)
                (function_declarator
                  declarator: (qualified_identifier
                    name: (identifier) @function.name))
              ]) @function.definition

            (declaration
              declarator: [
                (function_declarator
                  declarator: (identifier) @function.name)
                (function_declarator
                  declarator: (qualified_identifier
                    name: (identifier) @function.name))
              ]) @function.declaration

            (field_declaration
              declarator: (field_declarator
                declarator: (identifier) @field.name)) @field.definition

            (declaration
              declarator: (init_declarator
                declarator: (identifier) @variable.name)) @variable.definition

            (enum_specifier
              name: (type_identifier) @enum.name) @enum.definition

            (enumerator
              name: (identifier) @enum.member.name) @enum.member.definition

            (namespace_definition
              name: (identifier) @namespace.name) @namespace.definition

            (using_declaration
              (qualified_identifier
                name: (identifier) @using.name)) @using.declaration

            (type_definition
              declarator: (type_identifier) @typedef.name) @typedef.definition

            (template_declaration
              [
                (class_specifier
                  name: (type_identifier) @template.class.name)
                (function_definition
                  declarator: (function_declarator
                    declarator: (identifier) @template.function.name))
              ]) @template.definition
            "#,
        )?;

        let includes_query = Query::new(
            language,
            r#"
            (preproc_include
              path: [
                (string_literal) @include.path
                (system_lib_string) @include.system_path
              ]) @include.directive
            "#,
        )?;

        Ok(Self {
            parser,
            query_cursor: QueryCursor::new(),
            symbols_query,
            includes_query,
        })
    }

    pub async fn parse_file(&mut self, file_path: &Path) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path).await?;
        self.parse_content(&content, file_path)
    }

    pub fn parse_content(&mut self, content: &str, file_path: &Path) -> Result<ParseResult, Box<dyn std::error::Error>> {
        let tree = self.parser.parse(content, None).ok_or("Failed to parse content")?;
        
        let symbols = self.extract_symbols(&tree, content)?;
        let includes = self.extract_includes(&tree, content)?;
        
        Ok(ParseResult {
            file_path: file_path.to_path_buf(),
            symbols,
            includes,
            tree: Some(tree),
            content: content.to_string(),
        })
    }

    fn extract_symbols(&mut self, tree: &Tree, content: &str) -> Result<Vec<ParsedNode>, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        let captures = self.query_cursor.matches(&self.symbols_query, tree.root_node(), content.as_bytes());

        for match_ in captures {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = &self.symbols_query.capture_names()[capture.index as usize];
                
                let text = node.utf8_text(content.as_bytes()).unwrap_or("");
                
                let symbol = ParsedNode {
                    kind: capture_name.to_string(),
                    name: Some(text.to_string()),
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_row: node.start_position().row,
                    start_col: node.start_position().column,
                    end_row: node.end_position().row,
                    end_col: node.end_position().column,
                    text: text.to_string(),
                };
                
                symbols.push(symbol);
            }
        }
        
        Ok(symbols)
    }

    fn extract_includes(&mut self, tree: &Tree, content: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut includes = Vec::new();
        let captures = self.query_cursor.matches(&self.includes_query, tree.root_node(), content.as_bytes());

        for match_ in captures {
            for capture in match_.captures {
                let node = capture.node;
                let text = node.utf8_text(content.as_bytes()).unwrap_or("");
                
                let include_path = text.trim_matches('"').trim_matches('<').trim_matches('>');
                includes.push(include_path.to_string());
            }
        }
        
        Ok(includes)
    }

    pub fn get_node_at_position(&self, tree: &Tree, content: &str, line: usize, column: usize) -> Option<ParsedNode> {
        let byte_offset = self.position_to_byte_offset(content, line, column)?;
        let node = tree.root_node().descendant_for_byte_range(byte_offset, byte_offset)?;
        
        let text = node.utf8_text(content.as_bytes()).ok()?;
        
        Some(ParsedNode {
            kind: node.kind().to_string(),
            name: None,
            start_byte: node.start_byte(),
            end_byte: node.end_byte(),
            start_row: node.start_position().row,
            start_col: node.start_position().column,
            end_row: node.end_position().row,
            end_col: node.end_position().column,
            text: text.to_string(),
        })
    }

    fn position_to_byte_offset(&self, content: &str, line: usize, column: usize) -> Option<usize> {
        let mut current_line = 0;
        
        for (i, ch) in content.char_indices() {
            if current_line == line {
                if column == 0 {
                    return Some(i);
                }
                let mut current_col = 0;
                for (j, _) in content[i..].char_indices() {
                    if current_col == column {
                        return Some(i + j);
                    }
                    current_col += 1;
                    if content.chars().nth((i + j) / 4).unwrap_or('\0') == '\n' {
                        break;
                    }
                }
            }
            
            if ch == '\n' {
                current_line += 1;
            }
        }
        
        None
    }
}

#[derive(Debug)]
pub struct ParseResult {
    pub file_path: std::path::PathBuf,
    pub symbols: Vec<ParsedNode>,
    pub includes: Vec<String>,
    pub tree: Option<Tree>,
    pub content: String,
}

impl ParseResult {
    pub fn get_symbols_by_type(&self, symbol_type: &str) -> Vec<&ParsedNode> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.kind.contains(symbol_type))
            .collect()
    }
    
    pub fn get_symbol_count(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for symbol in &self.symbols {
            let base_type = symbol.kind.split('.').next().unwrap_or(&symbol.kind);
            *counts.entry(base_type.to_string()).or_insert(0) += 1;
        }
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_parser_creation() {
        let parser = TreeSitterParser::new();
        assert!(parser.is_ok());
    }

    #[tokio::test]
    async fn test_parse_simple_class() {
        let mut parser = TreeSitterParser::new().expect("Failed to create parser");
        let content = r#"
class TestClass {
public:
    int member_var;
    void test_method();
};
"#;
        
        let result = parser.parse_content(content, &PathBuf::from("test.cpp"));
        assert!(result.is_ok());
        
        let parse_result = result.unwrap();
        assert!(!parse_result.symbols.is_empty());
        
        let classes = parse_result.get_symbols_by_type("class");
        assert!(!classes.is_empty());
    }

    #[tokio::test]
    async fn test_parse_includes() {
        let mut parser = TreeSitterParser::new().expect("Failed to create parser");
        let content = r#"
#include <iostream>
#include "local_header.h"

int main() {
    return 0;
}
"#;
        
        let result = parser.parse_content(content, &PathBuf::from("test.cpp"));
        assert!(result.is_ok());
        
        let parse_result = result.unwrap();
        assert_eq!(parse_result.includes.len(), 2);
        assert!(parse_result.includes.contains(&"iostream".to_string()));
        assert!(parse_result.includes.contains(&"local_header.h".to_string()));
    }
}