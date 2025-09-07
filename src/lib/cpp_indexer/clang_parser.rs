use clang::{Clang, EntityKind, Index};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SemanticInfo {
    pub symbol_name: String,
    pub symbol_kind: EntityKind,
    pub fully_qualified_name: String,
    pub location: SourceLocation,
    pub type_info: Option<String>,
    pub access_specifier: Option<AccessSpecifier>,
    pub is_definition: bool,
    pub is_declaration: bool,
    pub references: Vec<SourceLocation>,
    pub template_info: Option<TemplateInfo>,
    pub inheritance_info: Option<InheritanceInfo>,
}

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub file_path: PathBuf,
    pub line: u32,
    pub column: u32,
    pub offset: u32,
}

#[derive(Debug, Clone)]
pub enum AccessSpecifier {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone)]
pub struct TemplateInfo {
    pub template_parameters: Vec<String>,
    pub specializations: Vec<String>,
    pub is_template: bool,
    pub is_specialization: bool,
}

#[derive(Debug, Clone)]
pub struct InheritanceInfo {
    pub base_classes: Vec<String>,
    pub derived_classes: Vec<String>,
    pub virtual_inheritance: bool,
}

#[derive(Debug)]
pub struct ClangParser {
    compile_flags: Vec<String>,
}

impl ClangParser {
    pub fn new(compile_flags: Option<Vec<String>>) -> Result<Self, Box<dyn std::error::Error>> {
        let default_flags = vec![
            "-std=c++17".to_string(),
        ];
        
        let flags = compile_flags.unwrap_or(default_flags);
        
        Ok(Self {
            compile_flags: flags,
        })
    }

    pub fn parse_file(&self, file_path: &Path) -> Result<SemanticParseResult, Box<dyn std::error::Error>> {
        let clang = Clang::new().map_err(|e| format!("Failed to initialize Clang: {:?}", e))?;
        let index = Index::new(&clang, false, false);
        
        let translation_unit = index
            .parser(file_path)
            .arguments(&self.compile_flags)
            .parse()
            .map_err(|e| format!("Failed to parse file: {:?}", e))?;

        let mut symbols = Vec::new();
        let mut references = HashMap::new();
        let mut type_hierarchy = HashMap::new();

        let entity = translation_unit.get_entity();
        self.visit_entity_recursive(&entity, &mut symbols, &mut references, &mut type_hierarchy)?;

        Ok(SemanticParseResult {
            file_path: file_path.to_path_buf(),
            symbols,
            references,
            type_hierarchy,
        })
    }

    fn visit_entity_recursive(
        &self,
        entity: &clang::Entity,
        symbols: &mut Vec<SemanticInfo>,
        references: &mut HashMap<String, Vec<SourceLocation>>,
        type_hierarchy: &mut HashMap<String, InheritanceInfo>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(location_info) = self.get_location_info(entity) {
            match entity.get_kind() {
                EntityKind::ClassDecl | 
                EntityKind::StructDecl | 
                EntityKind::UnionDecl |
                EntityKind::FunctionDecl | 
                EntityKind::Method |
                EntityKind::Constructor |
                EntityKind::Destructor |
                EntityKind::FieldDecl |
                EntityKind::VarDecl |
                EntityKind::EnumDecl |
                EntityKind::EnumConstantDecl |
                EntityKind::Namespace |
                EntityKind::TypedefDecl => {
                    let semantic_info = self.extract_semantic_info(entity, location_info)?;
                    
                    if let Some(ref name) = entity.get_name() {
                        references.entry(name.clone()).or_insert_with(Vec::new);
                    }
                    
                    if matches!(entity.get_kind(), EntityKind::ClassDecl | EntityKind::StructDecl) {
                        if let Some(inheritance) = self.extract_inheritance_info(entity)? {
                            if let Some(ref name) = entity.get_name() {
                                type_hierarchy.insert(name.clone(), inheritance);
                            }
                        }
                    }
                    
                    symbols.push(semantic_info);
                }
                _ => {}
            }
        }

        for child in entity.get_children() {
            self.visit_entity_recursive(&child, symbols, references, type_hierarchy)?;
        }

        Ok(())
    }

    fn get_location_info(&self, entity: &clang::Entity) -> Option<SourceLocation> {
        if let Some(location) = entity.get_location() {
            let file_location = location.get_file_location();
            if let Some(file) = file_location.file {
                return Some(SourceLocation {
                    file_path: file.get_path(),
                    line: file_location.line,
                    column: file_location.column,
                    offset: file_location.offset,
                });
            }
        }
        None
    }

    fn extract_semantic_info(
        &self,
        entity: &clang::Entity,
        location: SourceLocation,
    ) -> Result<SemanticInfo, Box<dyn std::error::Error>> {
        let symbol_name = entity.get_name().unwrap_or_default();
        let symbol_kind = entity.get_kind();
        let fully_qualified_name = entity.get_display_name().unwrap_or(symbol_name.clone());
        
        let type_info = entity.get_type().map(|t| t.get_display_name());
        
        let access_specifier = match entity.get_accessibility() {
            Some(clang::Accessibility::Public) => Some(AccessSpecifier::Public),
            Some(clang::Accessibility::Protected) => Some(AccessSpecifier::Protected),
            Some(clang::Accessibility::Private) => Some(AccessSpecifier::Private),
            _ => None,
        };

        let is_definition = entity.is_definition();
        let is_declaration = !is_definition;

        let template_info = self.extract_template_info(entity)?;

        Ok(SemanticInfo {
            symbol_name,
            symbol_kind,
            fully_qualified_name,
            location,
            type_info,
            access_specifier,
            is_definition,
            is_declaration,
            references: Vec::new(),
            template_info,
            inheritance_info: None,
        })
    }

    fn extract_template_info(
        &self,
        entity: &clang::Entity,
    ) -> Result<Option<TemplateInfo>, Box<dyn std::error::Error>> {
        // Check if entity is a template
        let is_template = matches!(
            entity.get_kind(), 
            EntityKind::ClassTemplate | EntityKind::FunctionTemplate
        );
        
        if is_template {
            let mut template_parameters = Vec::new();
            
            for child in entity.get_children() {
                match child.get_kind() {
                    EntityKind::TemplateTypeParameter |
                    EntityKind::NonTypeTemplateParameter |
                    EntityKind::TemplateTemplateParameter => {
                        if let Some(name) = child.get_name() {
                            template_parameters.push(name);
                        }
                    }
                    _ => {}
                }
            }
            
            Ok(Some(TemplateInfo {
                template_parameters,
                specializations: Vec::new(),
                is_template: true,
                is_specialization: false,
            }))
        } else {
            Ok(None)
        }
    }

    fn extract_inheritance_info(
        &self,
        entity: &clang::Entity,
    ) -> Result<Option<InheritanceInfo>, Box<dyn std::error::Error>> {
        if !matches!(entity.get_kind(), EntityKind::ClassDecl | EntityKind::StructDecl) {
            return Ok(None);
        }

        let mut base_classes = Vec::new();
        let mut virtual_inheritance = false;

        for child in entity.get_children() {
            if child.get_kind() == EntityKind::BaseSpecifier {
                if let Some(base_type) = child.get_type() {
                    base_classes.push(base_type.get_display_name());
                }
                
                if child.is_virtual_base() {
                    virtual_inheritance = true;
                }
            }
        }

        if base_classes.is_empty() {
            Ok(None)
        } else {
            Ok(Some(InheritanceInfo {
                base_classes,
                derived_classes: Vec::new(),
                virtual_inheritance,
            }))
        }
    }

    pub fn find_definition(
        &self,
        _file_path: &Path,
        _line: u32,
        _column: u32,
    ) -> Result<Option<SourceLocation>, Box<dyn std::error::Error>> {
        // Simplified implementation - would need more complex logic
        Ok(None)
    }

    pub fn find_references(
        &self,
        _file_path: &Path,
        _line: u32,
        _column: u32,
    ) -> Result<Vec<SourceLocation>, Box<dyn std::error::Error>> {
        // Simplified implementation - would need more complex logic
        Ok(Vec::new())
    }
}

#[derive(Debug)]
pub struct SemanticParseResult {
    pub file_path: PathBuf,
    pub symbols: Vec<SemanticInfo>,
    pub references: HashMap<String, Vec<SourceLocation>>,
    pub type_hierarchy: HashMap<String, InheritanceInfo>,
}

impl SemanticParseResult {
    pub fn get_symbols_by_kind(&self, kind: EntityKind) -> Vec<&SemanticInfo> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.symbol_kind == kind)
            .collect()
    }
    
    pub fn get_definitions(&self) -> Vec<&SemanticInfo> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.is_definition)
            .collect()
    }
    
    pub fn get_declarations(&self) -> Vec<&SemanticInfo> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.is_declaration)
            .collect()
    }
    
    pub fn get_template_symbols(&self) -> Vec<&SemanticInfo> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.template_info.is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parser_creation() {
        let parser = ClangParser::new(None);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_custom_compile_flags() {
        let flags = vec!["-std=c++20".to_string(), "-O2".to_string()];
        let parser = ClangParser::new(Some(flags.clone()));
        assert!(parser.is_ok());
        
        let parser = parser.unwrap();
        assert_eq!(parser.compile_flags, flags);
    }
}