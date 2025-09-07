use serde::{Deserialize, Serialize};

/// Tracks relationships between code elements (inheritance, usage, includes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolRelationship {
    /// Unique identifier (auto-increment)
    pub id: Option<i64>,
    /// Foreign key to Code Element (source)
    pub from_symbol_id: i64,
    /// Foreign key to Code Element (target)
    pub to_symbol_id: i64,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// File where relationship occurs
    pub file_path: String,
    /// Line number where relationship is declared
    pub line_number: u32,
}

/// Type of relationship between code elements
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipType {
    /// Class inheritance (class A : public B)
    Inherits,
    /// Usage of a symbol (variable usage, function call)
    Uses,
    /// Include directive (#include "header.h")
    Includes,
    /// Function call relationship
    Calls,
    /// Definition relationship (variable definition, function definition)
    Defines,
    /// Template instantiation
    Instantiates,
    /// Namespace membership
    ContainedIn,
    /// Friend relationship
    Friend,
    /// Override relationship (virtual function override)
    Overrides,
    /// Template specialization
    Specializes,
}

impl SymbolRelationship {
    /// Creates a new SymbolRelationship
    pub fn new(
        from_symbol_id: i64,
        to_symbol_id: i64,
        relationship_type: RelationshipType,
        file_path: String,
        line_number: u32,
    ) -> Self {
        Self {
            id: None,
            from_symbol_id,
            to_symbol_id,
            relationship_type,
            file_path,
            line_number,
        }
    }

    /// Validates the symbol relationship fields
    pub fn validate(&self) -> Result<(), String> {
        if self.from_symbol_id == self.to_symbol_id {
            return Err("From and to symbol IDs must be different".to_string());
        }

        if self.from_symbol_id <= 0 {
            return Err("From symbol ID must be positive".to_string());
        }

        if self.to_symbol_id <= 0 {
            return Err("To symbol ID must be positive".to_string());
        }

        if self.file_path.trim().is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        // Validate file path is relative
        if std::path::Path::new(&self.file_path).is_absolute() {
            return Err("File path must be relative".to_string());
        }

        if self.line_number == 0 {
            return Err("Line number must be positive (1-based)".to_string());
        }

        Ok(())
    }

    /// Returns true if this is a directional relationship
    pub fn is_directional(&self) -> bool {
        matches!(
            self.relationship_type,
            RelationshipType::Inherits
            | RelationshipType::Uses
            | RelationshipType::Calls
            | RelationshipType::Defines
            | RelationshipType::Instantiates
            | RelationshipType::ContainedIn
            | RelationshipType::Overrides
            | RelationshipType::Specializes
        )
    }

    /// Returns true if this is a bidirectional relationship
    pub fn is_bidirectional(&self) -> bool {
        matches!(
            self.relationship_type,
            RelationshipType::Friend | RelationshipType::Includes
        )
    }

    /// Returns the inverse relationship if applicable
    pub fn inverse_relationship_type(&self) -> Option<RelationshipType> {
        match self.relationship_type {
            RelationshipType::ContainedIn => Some(RelationshipType::Defines),
            RelationshipType::Defines => Some(RelationshipType::ContainedIn),
            _ => None,
        }
    }

    /// Creates the inverse relationship if this relationship type supports it
    pub fn create_inverse(&self) -> Option<SymbolRelationship> {
        if let Some(inverse_type) = self.inverse_relationship_type() {
            Some(SymbolRelationship::new(
                self.to_symbol_id,
                self.from_symbol_id,
                inverse_type,
                self.file_path.clone(),
                self.line_number,
            ))
        } else {
            None
        }
    }
}

impl RelationshipType {
    /// Returns all relationship types as a slice
    pub fn all() -> &'static [RelationshipType] {
        &[
            RelationshipType::Inherits,
            RelationshipType::Uses,
            RelationshipType::Includes,
            RelationshipType::Calls,
            RelationshipType::Defines,
            RelationshipType::Instantiates,
            RelationshipType::ContainedIn,
            RelationshipType::Friend,
            RelationshipType::Overrides,
            RelationshipType::Specializes,
        ]
    }

    /// Returns string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Inherits => "inherits",
            RelationshipType::Uses => "uses",
            RelationshipType::Includes => "includes",
            RelationshipType::Calls => "calls",
            RelationshipType::Defines => "defines",
            RelationshipType::Instantiates => "instantiates",
            RelationshipType::ContainedIn => "contained_in",
            RelationshipType::Friend => "friend",
            RelationshipType::Overrides => "overrides",
            RelationshipType::Specializes => "specializes",
        }
    }

    /// Returns a description of the relationship
    pub fn description(&self) -> &'static str {
        match self {
            RelationshipType::Inherits => "Class inheritance relationship",
            RelationshipType::Uses => "General usage relationship",
            RelationshipType::Includes => "File inclusion directive",
            RelationshipType::Calls => "Function call relationship",
            RelationshipType::Defines => "Definition relationship",
            RelationshipType::Instantiates => "Template instantiation",
            RelationshipType::ContainedIn => "Namespace/scope membership",
            RelationshipType::Friend => "Friend class/function relationship",
            RelationshipType::Overrides => "Virtual function override",
            RelationshipType::Specializes => "Template specialization",
        }
    }

    /// Returns true if this relationship type represents a structural dependency
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            RelationshipType::Inherits 
            | RelationshipType::ContainedIn 
            | RelationshipType::Defines
            | RelationshipType::Overrides
        )
    }

    /// Returns true if this relationship type represents a usage dependency
    pub fn is_usage(&self) -> bool {
        matches!(
            self,
            RelationshipType::Uses 
            | RelationshipType::Calls 
            | RelationshipType::Instantiates
        )
    }

    /// Returns true if this relationship type represents a compile-time dependency
    pub fn is_compile_time(&self) -> bool {
        matches!(
            self,
            RelationshipType::Includes 
            | RelationshipType::Inherits 
            | RelationshipType::Instantiates
            | RelationshipType::Specializes
        )
    }
}

/// Builder for creating complex relationship queries
#[derive(Debug, Clone)]
pub struct RelationshipQuery {
    pub from_symbol_id: Option<i64>,
    pub to_symbol_id: Option<i64>,
    pub relationship_types: Vec<RelationshipType>,
    pub file_path_pattern: Option<String>,
    pub include_inverse: bool,
}

impl RelationshipQuery {
    /// Creates a new empty query
    pub fn new() -> Self {
        Self {
            from_symbol_id: None,
            to_symbol_id: None,
            relationship_types: Vec::new(),
            file_path_pattern: None,
            include_inverse: false,
        }
    }

    /// Sets the from symbol ID
    pub fn from_symbol(mut self, symbol_id: i64) -> Self {
        self.from_symbol_id = Some(symbol_id);
        self
    }

    /// Sets the to symbol ID
    pub fn to_symbol(mut self, symbol_id: i64) -> Self {
        self.to_symbol_id = Some(symbol_id);
        self
    }

    /// Adds relationship types to filter by
    pub fn with_types(mut self, types: Vec<RelationshipType>) -> Self {
        self.relationship_types = types;
        self
    }

    /// Sets a file path pattern to match
    pub fn in_file(mut self, pattern: String) -> Self {
        self.file_path_pattern = Some(pattern);
        self
    }

    /// Include inverse relationships in results
    pub fn include_inverse(mut self) -> Self {
        self.include_inverse = true;
        self
    }
}

impl Default for RelationshipQuery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_relationship() -> SymbolRelationship {
        SymbolRelationship::new(
            1,
            2,
            RelationshipType::Inherits,
            "src/test.cpp".to_string(),
            10,
        )
    }

    #[test]
    fn test_symbol_relationship_new() {
        let rel = SymbolRelationship::new(
            5,
            10,
            RelationshipType::Calls,
            "src/main.cpp".to_string(),
            42,
        );

        assert_eq!(rel.from_symbol_id, 5);
        assert_eq!(rel.to_symbol_id, 10);
        assert_eq!(rel.relationship_type, RelationshipType::Calls);
        assert_eq!(rel.file_path, "src/main.cpp");
        assert_eq!(rel.line_number, 42);
        assert!(rel.id.is_none());
    }

    #[test]
    fn test_validation() {
        let mut rel = create_test_relationship();
        assert!(rel.validate().is_ok());

        // Test same symbol IDs
        rel.from_symbol_id = 1;
        rel.to_symbol_id = 1;
        assert!(rel.validate().is_err());

        // Test zero symbol IDs
        rel.from_symbol_id = 0;
        rel.to_symbol_id = 2;
        assert!(rel.validate().is_err());

        rel.from_symbol_id = 1;
        rel.to_symbol_id = 0;
        assert!(rel.validate().is_err());

        // Test empty file path
        rel.to_symbol_id = 2;
        rel.file_path = "".to_string();
        assert!(rel.validate().is_err());

        // Test absolute file path
        rel.file_path = if cfg!(windows) {
            "C:\\absolute\\path.cpp".to_string()
        } else {
            "/absolute/path.cpp".to_string()
        };
        assert!(rel.validate().is_err());

        // Test zero line number
        rel.file_path = "relative/path.cpp".to_string();
        rel.line_number = 0;
        assert!(rel.validate().is_err());
    }

    #[test]
    fn test_relationship_directionality() {
        let inherits = create_test_relationship(); // Inherits type
        assert!(inherits.is_directional());
        assert!(!inherits.is_bidirectional());

        let mut friend_rel = create_test_relationship();
        friend_rel.relationship_type = RelationshipType::Friend;
        assert!(!friend_rel.is_directional());
        assert!(friend_rel.is_bidirectional());
    }

    #[test]
    fn test_inverse_relationships() {
        let mut contained_rel = create_test_relationship();
        contained_rel.relationship_type = RelationshipType::ContainedIn;
        
        let inverse = contained_rel.create_inverse().unwrap();
        assert_eq!(inverse.from_symbol_id, contained_rel.to_symbol_id);
        assert_eq!(inverse.to_symbol_id, contained_rel.from_symbol_id);
        assert_eq!(inverse.relationship_type, RelationshipType::Defines);

        // Test relationship without inverse
        let inherits = create_test_relationship();
        assert!(inherits.create_inverse().is_none());
    }

    #[test]
    fn test_relationship_type_properties() {
        assert!(RelationshipType::Inherits.is_structural());
        assert!(!RelationshipType::Inherits.is_usage());
        assert!(RelationshipType::Inherits.is_compile_time());

        assert!(!RelationshipType::Uses.is_structural());
        assert!(RelationshipType::Uses.is_usage());
        assert!(!RelationshipType::Uses.is_compile_time());

        assert!(!RelationshipType::Includes.is_structural());
        assert!(!RelationshipType::Includes.is_usage());
        assert!(RelationshipType::Includes.is_compile_time());
    }

    #[test]
    fn test_relationship_type_strings() {
        assert_eq!(RelationshipType::Inherits.as_str(), "inherits");
        assert_eq!(RelationshipType::Uses.as_str(), "uses");
        assert_eq!(RelationshipType::Calls.as_str(), "calls");
        
        assert!(!RelationshipType::Inherits.description().is_empty());
        assert!(!RelationshipType::Uses.description().is_empty());
    }

    #[test]
    fn test_relationship_query_builder() {
        let query = RelationshipQuery::new()
            .from_symbol(1)
            .to_symbol(2)
            .with_types(vec![RelationshipType::Inherits, RelationshipType::Uses])
            .in_file("src/*.cpp".to_string())
            .include_inverse();

        assert_eq!(query.from_symbol_id, Some(1));
        assert_eq!(query.to_symbol_id, Some(2));
        assert_eq!(query.relationship_types.len(), 2);
        assert_eq!(query.file_path_pattern, Some("src/*.cpp".to_string()));
        assert!(query.include_inverse);
    }

    #[test]
    fn test_relationship_type_all() {
        let all_types = RelationshipType::all();
        assert!(all_types.len() >= 10);
        assert!(all_types.contains(&RelationshipType::Inherits));
        assert!(all_types.contains(&RelationshipType::Uses));
        assert!(all_types.contains(&RelationshipType::Calls));
    }
}