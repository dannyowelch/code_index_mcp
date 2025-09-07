use crate::lib::cpp_indexer::tree_sitter_parser::{TreeSitterParser, ParseResult, ParsedNode};
use crate::lib::cpp_indexer::clang_parser::{ClangParser, SemanticParseResult, SemanticInfo};
use crate::lib::storage::models::code_element::{SymbolType, AccessModifier};
use clang::EntityKind;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct ExtractedSymbol {
    pub name: String,
    pub symbol_type: SymbolType,
    pub visibility: Option<AccessModifier>,
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
    pub content: String,
    pub fully_qualified_name: String,
    pub namespace_path: Vec<String>,
    pub dependencies: Vec<String>,
    pub template_parameters: Vec<String>,
    pub base_classes: Vec<String>,
    pub member_functions: Vec<String>,
    pub member_variables: Vec<String>,
    pub signature: Option<String>,
    pub documentation: Option<String>,
    pub is_definition: bool,
    pub is_declaration: bool,
}

pub struct SymbolExtractor {
    tree_sitter_parser: TreeSitterParser,
    clang_parser: ClangParser,
}

impl SymbolExtractor {
    pub fn new(compile_flags: Option<Vec<String>>) -> Result<Self, Box<dyn std::error::Error>> {
        let tree_sitter_parser = TreeSitterParser::new()?;
        let clang_parser = ClangParser::new(compile_flags)?;
        
        Ok(Self {
            tree_sitter_parser,
            clang_parser,
        })
    }

    pub async fn extract_symbols(&mut self, file_path: &Path) -> Result<ExtractionResult, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        
        let tree_sitter_result = self.tree_sitter_parser.parse_file(file_path).await?;
        let clang_result = self.clang_parser.parse_file(file_path)?;
        
        let symbols = self.merge_parser_results(&tree_sitter_result, &clang_result)?;
        
        let extraction_time = start_time.elapsed();
        
        Ok(ExtractionResult {
            file_path: file_path.to_path_buf(),
            symbols,
            includes: tree_sitter_result.includes,
            extraction_time_ms: extraction_time.as_millis() as u32,
            tree_sitter_symbols: tree_sitter_result.symbols.len(),
            clang_symbols: clang_result.symbols.len(),
        })
    }

    fn merge_parser_results(
        &self,
        tree_sitter_result: &ParseResult,
        clang_result: &SemanticParseResult,
    ) -> Result<Vec<ExtractedSymbol>, Box<dyn std::error::Error>> {
        let mut symbols = Vec::new();
        let mut processed_locations = std::collections::HashSet::new();

        for semantic_info in &clang_result.symbols {
            let extracted_symbol = self.convert_semantic_to_extracted(semantic_info, clang_result)?;
            
            let location_key = format!(
                "{}:{}:{}",
                extracted_symbol.file_path.display(),
                extracted_symbol.start_line,
                extracted_symbol.start_column
            );
            
            if !processed_locations.contains(&location_key) {
                processed_locations.insert(location_key);
                symbols.push(extracted_symbol);
            }
        }

        for parsed_node in &tree_sitter_result.symbols {
            let extracted_symbol = self.convert_parsed_to_extracted(parsed_node, &tree_sitter_result.file_path)?;
            
            let location_key = format!(
                "{}:{}:{}",
                extracted_symbol.file_path.display(),
                extracted_symbol.start_line,
                extracted_symbol.start_column
            );
            
            if !processed_locations.contains(&location_key) {
                processed_locations.insert(location_key);
                symbols.push(extracted_symbol);
            }
        }

        self.enrich_symbols_with_relationships(&mut symbols, clang_result)?;
        
        Ok(symbols)
    }

    fn convert_semantic_to_extracted(
        &self,
        semantic_info: &SemanticInfo,
        clang_result: &SemanticParseResult,
    ) -> Result<ExtractedSymbol, Box<dyn std::error::Error>> {
        let symbol_type = self.entity_kind_to_symbol_type(semantic_info.symbol_kind);
        let visibility = self.access_specifier_to_access_modifier(&semantic_info.access_specifier);
        
        let namespace_path = self.extract_namespace_path(&semantic_info.fully_qualified_name);
        let dependencies = self.extract_dependencies(semantic_info)?;
        
        let template_parameters = semantic_info
            .template_info
            .as_ref()
            .map(|info| info.template_parameters.clone())
            .unwrap_or_default();
        
        let base_classes = semantic_info
            .inheritance_info
            .as_ref()
            .map(|info| info.base_classes.clone())
            .unwrap_or_default();

        let (member_functions, member_variables) = self.extract_class_members(semantic_info, clang_result)?;

        Ok(ExtractedSymbol {
            name: semantic_info.symbol_name.clone(),
            symbol_type,
            visibility,
            file_path: semantic_info.location.file_path.clone(),
            start_line: semantic_info.location.line,
            end_line: semantic_info.location.line,
            start_column: semantic_info.location.column,
            end_column: semantic_info.location.column,
            content: String::new(),
            fully_qualified_name: semantic_info.fully_qualified_name.clone(),
            namespace_path,
            dependencies,
            template_parameters,
            base_classes,
            member_functions,
            member_variables,
            signature: semantic_info.type_info.clone(),
            documentation: None,
            is_definition: semantic_info.is_definition,
            is_declaration: semantic_info.is_declaration,
        })
    }

    fn convert_parsed_to_extracted(
        &self,
        parsed_node: &ParsedNode,
        file_path: &PathBuf,
    ) -> Result<ExtractedSymbol, Box<dyn std::error::Error>> {
        let symbol_type = self.parse_kind_to_symbol_type(&parsed_node.kind);
        
        Ok(ExtractedSymbol {
            name: parsed_node.name.as_ref().unwrap_or(&parsed_node.text).clone(),
            symbol_type,
            visibility: None,
            file_path: file_path.clone(),
            start_line: parsed_node.start_row as u32 + 1,
            end_line: parsed_node.end_row as u32 + 1,
            start_column: parsed_node.start_col as u32,
            end_column: parsed_node.end_col as u32,
            content: parsed_node.text.clone(),
            fully_qualified_name: parsed_node.name.as_ref().unwrap_or(&parsed_node.text).clone(),
            namespace_path: Vec::new(),
            dependencies: Vec::new(),
            template_parameters: Vec::new(),
            base_classes: Vec::new(),
            member_functions: Vec::new(),
            member_variables: Vec::new(),
            signature: None,
            documentation: None,
            is_definition: true,
            is_declaration: false,
        })
    }

    fn entity_kind_to_symbol_type(&self, entity_kind: EntityKind) -> SymbolType {
        match entity_kind {
            EntityKind::ClassDecl => SymbolType::Class,
            EntityKind::StructDecl => SymbolType::Struct,
            EntityKind::UnionDecl => SymbolType::Union,
            EntityKind::FunctionDecl | EntityKind::Method => SymbolType::Function,
            EntityKind::Constructor => SymbolType::Constructor,
            EntityKind::Destructor => SymbolType::Destructor,
            EntityKind::FieldDecl => SymbolType::Field,
            EntityKind::VarDecl => SymbolType::Variable,
            EntityKind::EnumDecl => SymbolType::Enum,
            EntityKind::EnumConstantDecl => SymbolType::EnumConstant,
            EntityKind::Namespace => SymbolType::Namespace,
            EntityKind::TypedefDecl | EntityKind::TypeAliasDecl => SymbolType::Typedef,
            _ => SymbolType::Unknown,
        }
    }

    fn parse_kind_to_symbol_type(&self, parse_kind: &str) -> SymbolType {
        if parse_kind.contains("class") {
            SymbolType::Class
        } else if parse_kind.contains("struct") {
            SymbolType::Struct
        } else if parse_kind.contains("function") {
            SymbolType::Function
        } else if parse_kind.contains("field") {
            SymbolType::Field
        } else if parse_kind.contains("variable") {
            SymbolType::Variable
        } else if parse_kind.contains("enum") && parse_kind.contains("member") {
            SymbolType::EnumConstant
        } else if parse_kind.contains("enum") {
            SymbolType::Enum
        } else if parse_kind.contains("namespace") {
            SymbolType::Namespace
        } else if parse_kind.contains("typedef") {
            SymbolType::Typedef
        } else if parse_kind.contains("template") {
            SymbolType::Template
        } else {
            SymbolType::Unknown
        }
    }

    fn access_specifier_to_access_modifier(
        &self,
        access_specifier: &Option<crate::lib::cpp_indexer::clang_parser::AccessSpecifier>,
    ) -> Option<AccessModifier> {
        access_specifier.as_ref().map(|spec| match spec {
            crate::lib::cpp_indexer::clang_parser::AccessSpecifier::Public => AccessModifier::Public,
            crate::lib::cpp_indexer::clang_parser::AccessSpecifier::Protected => AccessModifier::Protected,
            crate::lib::cpp_indexer::clang_parser::AccessSpecifier::Private => AccessModifier::Private,
        })
    }

    fn extract_namespace_path(&self, fully_qualified_name: &str) -> Vec<String> {
        let parts: Vec<&str> = fully_qualified_name.split("::").collect();
        if parts.len() > 1 {
            parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        }
    }

    fn extract_dependencies(&self, semantic_info: &SemanticInfo) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut dependencies = Vec::new();
        
        if let Some(type_info) = &semantic_info.type_info {
            let type_parts: Vec<&str> = type_info.split_whitespace().collect();
            for part in type_parts {
                if part.contains("::") {
                    dependencies.push(part.to_string());
                }
            }
        }
        
        Ok(dependencies)
    }

    fn extract_class_members(
        &self,
        semantic_info: &SemanticInfo,
        clang_result: &SemanticParseResult,
    ) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
        let mut member_functions = Vec::new();
        let mut member_variables = Vec::new();
        
        for symbol in &clang_result.symbols {
            if symbol.fully_qualified_name.starts_with(&semantic_info.fully_qualified_name) &&
               symbol.fully_qualified_name != semantic_info.fully_qualified_name {
                
                match symbol.symbol_kind {
                    EntityKind::Method | EntityKind::Constructor | EntityKind::Destructor => {
                        member_functions.push(symbol.symbol_name.clone());
                    }
                    EntityKind::FieldDecl => {
                        member_variables.push(symbol.symbol_name.clone());
                    }
                    _ => {}
                }
            }
        }
        
        Ok((member_functions, member_variables))
    }

    fn enrich_symbols_with_relationships(
        &self,
        symbols: &mut Vec<ExtractedSymbol>,
        clang_result: &SemanticParseResult,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for symbol in symbols.iter_mut() {
            if let Some(inheritance_info) = clang_result.type_hierarchy.get(&symbol.name) {
                symbol.base_classes = inheritance_info.base_classes.clone();
            }
            
            if let Some(references) = clang_result.references.get(&symbol.name) {
                symbol.dependencies.extend(
                    references
                        .iter()
                        .map(|loc| format!("{}:{}", loc.file_path.display(), loc.line))
                        .collect::<Vec<_>>()
                );
            }
        }
        
        Ok(())
    }

    pub fn extract_file_dependencies(&self, symbols: &[ExtractedSymbol], includes: &[String]) -> Vec<String> {
        let mut dependencies = std::collections::HashSet::new();
        
        for include in includes {
            dependencies.insert(include.clone());
        }
        
        for symbol in symbols {
            for dep in &symbol.dependencies {
                if dep.contains('.') && (dep.ends_with(".h") || dep.ends_with(".hpp") || dep.ends_with(".hxx")) {
                    dependencies.insert(dep.clone());
                }
            }
        }
        
        dependencies.into_iter().collect()
    }

    pub fn group_symbols_by_type<'a>(&self, symbols: &'a [ExtractedSymbol]) -> HashMap<SymbolType, Vec<&'a ExtractedSymbol>> {
        let mut grouped = HashMap::new();
        
        for symbol in symbols {
            grouped
                .entry(symbol.symbol_type)
                .or_insert_with(Vec::new)
                .push(symbol);
        }
        
        grouped
    }

    pub fn filter_public_api<'a>(&self, symbols: &'a [ExtractedSymbol]) -> Vec<&'a ExtractedSymbol> {
        symbols
            .iter()
            .filter(|symbol| {
                matches!(symbol.visibility, Some(AccessModifier::Public) | None) &&
                matches!(
                    symbol.symbol_type,
                    SymbolType::Class | SymbolType::Struct | SymbolType::Function | 
                    SymbolType::Enum | SymbolType::Typedef
                )
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ExtractionResult {
    pub file_path: PathBuf,
    pub symbols: Vec<ExtractedSymbol>,
    pub includes: Vec<String>,
    pub extraction_time_ms: u32,
    pub tree_sitter_symbols: usize,
    pub clang_symbols: usize,
}

impl ExtractionResult {
    pub fn get_symbol_count_by_type(&self) -> HashMap<SymbolType, usize> {
        let mut counts = HashMap::new();
        for symbol in &self.symbols {
            *counts.entry(symbol.symbol_type).or_insert(0) += 1;
        }
        counts
    }
    
    pub fn get_definitions(&self) -> Vec<&ExtractedSymbol> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.is_definition)
            .collect()
    }
    
    pub fn get_declarations(&self) -> Vec<&ExtractedSymbol> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.is_declaration)
            .collect()
    }
    
    pub fn get_template_symbols(&self) -> Vec<&ExtractedSymbol> {
        self.symbols
            .iter()
            .filter(|symbol| !symbol.template_parameters.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_extractor_creation() {
        let extractor = SymbolExtractor::new(None);
        assert!(extractor.is_ok());
    }

    #[tokio::test]
    async fn test_symbol_type_conversion() {
        let extractor = SymbolExtractor::new(None).expect("Failed to create extractor");
        
        assert_eq!(
            extractor.entity_kind_to_symbol_type(EntityKind::ClassDecl),
            SymbolType::Class
        );
        assert_eq!(
            extractor.entity_kind_to_symbol_type(EntityKind::FunctionDecl),
            SymbolType::Function
        );
    }

    #[tokio::test]
    async fn test_parse_kind_conversion() {
        let extractor = SymbolExtractor::new(None).expect("Failed to create extractor");
        
        assert_eq!(
            extractor.parse_kind_to_symbol_type("class.name"),
            SymbolType::Class
        );
        assert_eq!(
            extractor.parse_kind_to_symbol_type("function.name"),
            SymbolType::Function
        );
    }

    #[tokio::test]
    async fn test_namespace_path_extraction() {
        let extractor = SymbolExtractor::new(None).expect("Failed to create extractor");
        
        let path = extractor.extract_namespace_path("std::vector::iterator");
        assert_eq!(path, vec!["std", "vector"]);
        
        let path = extractor.extract_namespace_path("MyClass");
        assert_eq!(path, Vec::<String>::new());
    }
}