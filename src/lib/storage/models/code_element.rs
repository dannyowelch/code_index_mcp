use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Individual C++ code symbols and constructs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeElement {
    /// Unique identifier (auto-increment)
    pub id: Option<i64>,
    /// Foreign key to Code Index
    pub index_id: Uuid,
    /// Name of the symbol (function, class, variable)
    pub symbol_name: String,
    /// Type of symbol
    pub symbol_type: SymbolType,
    /// Relative path from codebase root
    pub file_path: String,
    /// Line number in file (1-based)
    pub line_number: u32,
    /// Column number in file (1-based)
    pub column_number: u32,
    /// Blake3 hash of definition for change detection
    pub definition_hash: String,
    /// Fully qualified scope (e.g., "MyNamespace::MyClass")
    pub scope: Option<String>,
    /// Access modifier
    pub access_modifier: Option<AccessModifier>,
    /// Boolean - true if declaration, false if definition
    pub is_declaration: bool,
    /// Function signature or variable type (optional)
    pub signature: Option<String>,
}

/// Type of C++ symbol
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolType {
    Function,
    Class,
    Struct,
    Variable,
    Macro,
    Namespace,
    Enum,
    Typedef,
    Union,
    Template,
    Constructor,
    Destructor,
    Operator,
    Field,
    EnumConstant,
    Unknown,
}

/// C++ access modifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccessModifier {
    Public,
    Private,
    Protected,
}

impl CodeElement {
    /// Creates a new CodeElement
    pub fn new(
        index_id: Uuid,
        symbol_name: String,
        symbol_type: SymbolType,
        file_path: String,
        line_number: u32,
        column_number: u32,
        definition_hash: String,
    ) -> Self {
        Self {
            id: None,
            index_id,
            symbol_name,
            symbol_type,
            file_path,
            line_number,
            column_number,
            definition_hash,
            scope: None,
            access_modifier: None,
            is_declaration: false,
            signature: None,
        }
    }

    /// Sets the scope for this code element
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Sets the access modifier for this code element
    pub fn with_access_modifier(mut self, access_modifier: AccessModifier) -> Self {
        self.access_modifier = Some(access_modifier);
        self
    }

    /// Sets whether this is a declaration
    pub fn with_declaration(mut self, is_declaration: bool) -> Self {
        self.is_declaration = is_declaration;
        self
    }

    /// Sets the signature for this code element
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Validates the code element fields
    pub fn validate(&self) -> Result<(), String> {
        if self.symbol_name.trim().is_empty() {
            return Err("Symbol name cannot be empty".to_string());
        }

        if self.file_path.trim().is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        // Validate file path is relative and uses forward slashes
        if std::path::Path::new(&self.file_path).is_absolute() {
            return Err("File path must be relative".to_string());
        }

        if self.line_number == 0 {
            return Err("Line number must be positive (1-based)".to_string());
        }

        if self.column_number == 0 {
            return Err("Column number must be positive (1-based)".to_string());
        }

        // Validate Blake3 hash format (64 character hex string)
        if self.definition_hash.len() != 64 {
            return Err("Definition hash must be 64 characters".to_string());
        }

        if !self.definition_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Definition hash must contain only hexadecimal characters".to_string());
        }

        Ok(())
    }

    /// Returns the fully qualified name including scope
    pub fn fully_qualified_name(&self) -> String {
        match &self.scope {
            Some(scope) if !scope.is_empty() => format!("{}::{}", scope, self.symbol_name),
            _ => self.symbol_name.clone(),
        }
    }

    /// Returns true if this symbol is a type definition
    pub fn is_type(&self) -> bool {
        matches!(
            self.symbol_type,
            SymbolType::Class 
            | SymbolType::Struct 
            | SymbolType::Enum 
            | SymbolType::Union 
            | SymbolType::Typedef
        )
    }

    /// Returns true if this symbol is callable
    pub fn is_callable(&self) -> bool {
        matches!(
            self.symbol_type,
            SymbolType::Function 
            | SymbolType::Constructor 
            | SymbolType::Destructor 
            | SymbolType::Operator
        )
    }
}

impl SymbolType {
    /// Returns all symbol types as a slice
    pub fn all() -> &'static [SymbolType] {
        &[
            SymbolType::Function,
            SymbolType::Class,
            SymbolType::Struct,
            SymbolType::Variable,
            SymbolType::Macro,
            SymbolType::Namespace,
            SymbolType::Enum,
            SymbolType::Typedef,
            SymbolType::Union,
            SymbolType::Template,
            SymbolType::Constructor,
            SymbolType::Destructor,
            SymbolType::Operator,
            SymbolType::Field,
            SymbolType::EnumConstant,
            SymbolType::Unknown,
        ]
    }

    /// Returns string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolType::Function => "function",
            SymbolType::Class => "class",
            SymbolType::Struct => "struct",
            SymbolType::Variable => "variable",
            SymbolType::Macro => "macro",
            SymbolType::Namespace => "namespace",
            SymbolType::Enum => "enum",
            SymbolType::Typedef => "typedef",
            SymbolType::Union => "union",
            SymbolType::Template => "template",
            SymbolType::Constructor => "constructor",
            SymbolType::Destructor => "destructor",
            SymbolType::Operator => "operator",
            SymbolType::Field => "field",
            SymbolType::EnumConstant => "enum_constant",
            SymbolType::Unknown => "unknown",
        }
    }
}

impl AccessModifier {
    /// Returns string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessModifier::Public => "public",
            AccessModifier::Private => "private",
            AccessModifier::Protected => "protected",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_element() -> CodeElement {
        CodeElement::new(
            Uuid::new_v4(),
            "testFunction".to_string(),
            SymbolType::Function,
            "src/test.cpp".to_string(),
            10,
            5,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        )
    }

    #[test]
    fn test_code_element_new() {
        let index_id = Uuid::new_v4();
        let element = CodeElement::new(
            index_id,
            "TestClass".to_string(),
            SymbolType::Class,
            "include/test.h".to_string(),
            1,
            1,
            "abcd1234".repeat(8),
        );

        assert_eq!(element.index_id, index_id);
        assert_eq!(element.symbol_name, "TestClass");
        assert_eq!(element.symbol_type, SymbolType::Class);
        assert_eq!(element.file_path, "include/test.h");
        assert_eq!(element.line_number, 1);
        assert_eq!(element.column_number, 1);
        assert_eq!(element.is_declaration, false);
        assert!(element.scope.is_none());
        assert!(element.access_modifier.is_none());
        assert!(element.signature.is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let element = create_test_element()
            .with_scope("MyNamespace".to_string())
            .with_access_modifier(AccessModifier::Public)
            .with_declaration(true)
            .with_signature("void testFunction()".to_string());

        assert_eq!(element.scope, Some("MyNamespace".to_string()));
        assert_eq!(element.access_modifier, Some(AccessModifier::Public));
        assert_eq!(element.is_declaration, true);
        assert_eq!(element.signature, Some("void testFunction()".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut element = create_test_element();
        assert!(element.validate().is_ok());

        // Test empty symbol name
        element.symbol_name = "".to_string();
        assert!(element.validate().is_err());

        // Test empty file path
        element.symbol_name = "test".to_string();
        element.file_path = "".to_string();
        assert!(element.validate().is_err());

        // Test absolute file path
        element.file_path = if cfg!(windows) {
            "C:\\absolute\\path.cpp".to_string()
        } else {
            "/absolute/path.cpp".to_string()
        };
        assert!(element.validate().is_err());

        // Test zero line number
        element.file_path = "relative/path.cpp".to_string();
        element.line_number = 0;
        assert!(element.validate().is_err());

        // Test zero column number
        element.line_number = 1;
        element.column_number = 0;
        assert!(element.validate().is_err());

        // Test invalid hash length
        element.column_number = 1;
        element.definition_hash = "short".to_string();
        assert!(element.validate().is_err());

        // Test invalid hash characters
        element.definition_hash = "g".repeat(64);
        assert!(element.validate().is_err());
    }

    #[test]
    fn test_fully_qualified_name() {
        let mut element = create_test_element();
        assert_eq!(element.fully_qualified_name(), "testFunction");

        element.scope = Some("MyNamespace::MyClass".to_string());
        assert_eq!(element.fully_qualified_name(), "MyNamespace::MyClass::testFunction");

        element.scope = Some("".to_string());
        assert_eq!(element.fully_qualified_name(), "testFunction");
    }

    #[test]
    fn test_symbol_classification() {
        let class_element = create_test_element().with_scope("NS".to_string());
        assert!(!class_element.is_type());
        assert!(class_element.is_callable());

        let mut class_element = create_test_element();
        class_element.symbol_type = SymbolType::Class;
        assert!(class_element.is_type());
        assert!(!class_element.is_callable());

        let mut var_element = create_test_element();
        var_element.symbol_type = SymbolType::Variable;
        assert!(!var_element.is_type());
        assert!(!var_element.is_callable());
    }

    #[test]
    fn test_symbol_type_as_str() {
        assert_eq!(SymbolType::Function.as_str(), "function");
        assert_eq!(SymbolType::Class.as_str(), "class");
        assert_eq!(SymbolType::Variable.as_str(), "variable");
    }

    #[test]
    fn test_access_modifier_as_str() {
        assert_eq!(AccessModifier::Public.as_str(), "public");
        assert_eq!(AccessModifier::Private.as_str(), "private");
        assert_eq!(AccessModifier::Protected.as_str(), "protected");
    }
}