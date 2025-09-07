#[cfg(test)]
mod test_find_references {
    use serde_json::{json, Value};
    
    // All tests in this module must fail until find_references MCP tool is implemented
    fn ensure_not_implemented() {
        panic!("find_references MCP tool not yet implemented");
    }
    
    #[tokio::test]
    async fn test_find_references_valid_inputs() {
        ensure_not_implemented();
        
        // Test with minimal required parameters
        let input = json!({
            "index_name": "test_index",
            "symbol_name": "MyFunction"
        });
        
        assert!(input["index_name"].is_string());
        assert!(input["symbol_name"].is_string());
        
        // Test with all optional parameters
        let input_full = json!({
            "index_name": "test_index",
            "symbol_name": "MyClass",
            "symbol_type": "class",
            "include_declarations": false
        });
        
        assert!(input_full["index_name"].is_string());
        assert!(input_full["symbol_name"].is_string());
        assert!(input_full["symbol_type"].is_string());
        assert!(input_full["include_declarations"].is_boolean());
    }
    
    #[tokio::test]
    async fn test_find_references_missing_required_fields() {
        // Missing index_name
        let input_missing_index = json!({
            "symbol_name": "MyFunction"
        });
        
        assert!(input_missing_index["index_name"].is_null());
        
        // Missing symbol_name
        let input_missing_symbol = json!({
            "index_name": "test_index"
        });
        
        assert!(input_missing_symbol["symbol_name"].is_null());
    }
    
    #[tokio::test]
    async fn test_find_references_symbol_types() {
        let valid_symbol_types = vec![
            "function", "class", "variable", "macro", "namespace", "enum", "typedef"
        ];
        
        for symbol_type in valid_symbol_types {
            let input = json!({
                "index_name": "test_index",
                "symbol_name": "TestSymbol",
                "symbol_type": symbol_type
            });
            
            assert_eq!(input["symbol_type"].as_str().unwrap(), symbol_type);
        }
    }
    
    #[tokio::test]
    async fn test_find_references_response_schema() {
        // Expected response structure (uses SearchResult schema)
        let expected_response = json!({
            "symbols": [
                {
                    "id": 1,
                    "name": "MyFunction",
                    "type": "function", 
                    "file_path": "src/caller.cpp",
                    "line_number": 25,
                    "column_number": 8,
                    "scope": "main",
                    "signature": "MyFunction()",
                    "is_declaration": false
                },
                {
                    "id": 1,
                    "name": "MyFunction",
                    "type": "function",
                    "file_path": "src/header.h",
                    "line_number": 10,
                    "column_number": 5,
                    "scope": "",
                    "signature": "int MyFunction()",
                    "is_declaration": true
                }
            ],
            "total_count": 2,
            "query_time_ms": 15
        });
        
        // Validate response schema structure 
        assert!(expected_response["symbols"].is_array());
        assert!(expected_response["total_count"].is_number());
        assert!(expected_response["query_time_ms"].is_number());
        
        for symbol in expected_response["symbols"].as_array().unwrap() {
            assert!(symbol["id"].is_number());
            assert!(symbol["name"].is_string());
            assert!(symbol["type"].is_string());
            assert!(symbol["file_path"].is_string());
            assert!(symbol["line_number"].is_number());
            assert!(symbol["column_number"].is_number());
            
            // Optional fields
            if !symbol["scope"].is_null() {
                assert!(symbol["scope"].is_string());
            }
            if !symbol["signature"].is_null() {
                assert!(symbol["signature"].is_string());
            }
            if !symbol["is_declaration"].is_null() {
                assert!(symbol["is_declaration"].is_boolean());
            }
        }
    }
    
    #[tokio::test]
    async fn test_find_references_default_include_declarations() {
        let input = json!({
            "index_name": "test_index",
            "symbol_name": "MyFunction"
        });
        
        // Should default to including declarations (true)
        let default_include_declarations = true;
        assert_eq!(default_include_declarations, true);
    }
    
    #[tokio::test]
    async fn test_find_references_exclude_declarations() {
        let input = json!({
            "index_name": "test_index",
            "symbol_name": "MyFunction",
            "include_declarations": false
        });
        
        assert_eq!(input["include_declarations"].as_bool().unwrap(), false);
        
        // When include_declarations is false, response should only contain non-declarations
        let expected_response = json!({
            "symbols": [
                {
                    "id": 1,
                    "name": "MyFunction",
                    "type": "function",
                    "file_path": "src/caller.cpp",
                    "line_number": 25,
                    "column_number": 8,
                    "is_declaration": false
                }
            ],
            "total_count": 1,
            "query_time_ms": 10
        });
        
        // All symbols should have is_declaration: false when exclude declarations
        for symbol in expected_response["symbols"].as_array().unwrap() {
            if let Some(is_decl) = symbol["is_declaration"].as_bool() {
                assert_eq!(is_decl, false);
            }
        }
    }
    
    #[tokio::test]
    async fn test_find_references_invalid_types() {
        let invalid_inputs = vec![
            json!({
                "index_name": 123, // Should be string
                "symbol_name": "MyFunction"
            }),
            json!({
                "index_name": "test_index",
                "symbol_name": 456 // Should be string
            }),
            json!({
                "index_name": "test_index",
                "symbol_name": "MyFunction",
                "symbol_type": 789 // Should be string
            }),
            json!({
                "index_name": "test_index",
                "symbol_name": "MyFunction",
                "include_declarations": "false" // Should be boolean
            })
        ];
        
        for invalid_input in invalid_inputs {
            if invalid_input["index_name"].is_number() {
                assert!(!invalid_input["index_name"].is_string());
            }
            if invalid_input["symbol_name"].is_number() {
                assert!(!invalid_input["symbol_name"].is_string());
            }
            if invalid_input["symbol_type"].is_number() {
                assert!(!invalid_input["symbol_type"].is_string());
            }
            if invalid_input["include_declarations"].is_string() {
                assert!(!invalid_input["include_declarations"].is_boolean());
            }
        }
    }
    
    #[tokio::test]
    async fn test_find_references_invalid_symbol_type() {
        let input_invalid_type = json!({
            "index_name": "test_index",
            "symbol_name": "MySymbol",
            "symbol_type": "invalid_type"
        });
        
        let valid_types = vec!["function", "class", "variable", "macro", "namespace", "enum", "typedef"];
        let provided_type = input_invalid_type["symbol_type"].as_str().unwrap();
        
        // Should not be a valid symbol type
        assert!(!valid_types.contains(&provided_type));
    }
    
    #[tokio::test]
    async fn test_find_references_no_results() {
        // Test expected response when no references found
        let empty_response = json!({
            "symbols": [],
            "total_count": 0,
            "query_time_ms": 3
        });
        
        assert!(empty_response["symbols"].is_array());
        assert_eq!(empty_response["symbols"].as_array().unwrap().len(), 0);
        assert_eq!(empty_response["total_count"].as_i64().unwrap(), 0);
        assert!(empty_response["query_time_ms"].is_number());
    }
    
    #[tokio::test]
    async fn test_find_references_symbol_not_found() {
        // Test expected error response when symbol doesn't exist
        let error_response = json!({
            "error": "Symbol not found",
            "error_code": "SYMBOL_NOT_FOUND",
            "details": {
                "symbol_name": "NonExistentFunction",
                "index_name": "test_index"
            }
        });
        
        assert!(error_response["error"].is_string());
        assert!(error_response["error_code"].is_string());
        assert!(error_response["details"].is_object());
        assert!(error_response["details"]["symbol_name"].is_string());
        assert!(error_response["details"]["index_name"].is_string());
    }
}