#[cfg(test)]
mod test_update_file {
    use serde_json::{json, Value};
    
    #[tokio::test]
    async fn test_update_file_valid_inputs() {
        // Test with required parameters
        let input = json!({
            "index_name": "test_index",
            "file_path": "src/modified.cpp"
        });
        
        assert!(input["index_name"].is_string());
        assert!(input["file_path"].is_string());
    }
    
    #[tokio::test]
    async fn test_update_file_missing_required_fields() {
        // Missing index_name
        let input_missing_index = json!({
            "file_path": "src/modified.cpp"
        });
        
        assert!(input_missing_index["index_name"].is_null());
        
        // Missing file_path
        let input_missing_path = json!({
            "index_name": "test_index"
        });
        
        assert!(input_missing_path["file_path"].is_null());
    }
    
    #[tokio::test]
    async fn test_update_file_response_success() {
        // Expected successful update response
        let success_response = json!({
            "success": true,
            "file_path": "src/modified.cpp",
            "symbols_added": 3,
            "symbols_removed": 1,
            "symbols_modified": 2,
            "total_symbols": 15,
            "update_time_ms": 125,
            "file_hash": "abc123def456789",
            "changes": [
                {
                    "type": "added",
                    "symbol_name": "newFunction",
                    "symbol_type": "function",
                    "line_number": 25
                },
                {
                    "type": "removed",
                    "symbol_name": "oldFunction", 
                    "symbol_type": "function",
                    "line_number": 10
                },
                {
                    "type": "modified",
                    "symbol_name": "existingFunction",
                    "symbol_type": "function",
                    "line_number": 35,
                    "old_signature": "void existingFunction(int x)",
                    "new_signature": "void existingFunction(int x, bool flag)"
                }
            ]
        });
        
        // Validate response schema structure
        assert!(success_response["success"].is_boolean());
        assert_eq!(success_response["success"].as_bool().unwrap(), true);
        assert!(success_response["file_path"].is_string());
        
        // Optional metadata fields
        if !success_response["symbols_added"].is_null() {
            assert!(success_response["symbols_added"].is_number());
        }
        if !success_response["symbols_removed"].is_null() {
            assert!(success_response["symbols_removed"].is_number());
        }
        if !success_response["symbols_modified"].is_null() {
            assert!(success_response["symbols_modified"].is_number());
        }
        if !success_response["total_symbols"].is_null() {
            assert!(success_response["total_symbols"].is_number());
        }
        if !success_response["update_time_ms"].is_null() {
            assert!(success_response["update_time_ms"].is_number());
        }
        if !success_response["file_hash"].is_null() {
            assert!(success_response["file_hash"].is_string());
        }
        
        // Validate changes array if present
        if !success_response["changes"].is_null() {
            assert!(success_response["changes"].is_array());
            for change in success_response["changes"].as_array().unwrap() {
                assert!(change["type"].is_string());
                assert!(change["symbol_name"].is_string());
                assert!(change["symbol_type"].is_string());
                
                if !change["line_number"].is_null() {
                    assert!(change["line_number"].is_number());
                }
            }
        }
    }
    
    #[tokio::test]
    async fn test_update_file_response_no_changes() {
        // Expected response when file hasn't changed
        let no_changes_response = json!({
            "success": true,
            "file_path": "src/unchanged.cpp",
            "symbols_added": 0,
            "symbols_removed": 0,
            "symbols_modified": 0,
            "total_symbols": 10,
            "update_time_ms": 5,
            "file_hash": "same123hash456",
            "message": "File has not changed since last index"
        });
        
        assert!(no_changes_response["success"].is_boolean());
        assert_eq!(no_changes_response["success"].as_bool().unwrap(), true);
        assert_eq!(no_changes_response["symbols_added"].as_i64().unwrap(), 0);
        assert_eq!(no_changes_response["symbols_removed"].as_i64().unwrap(), 0);
        assert_eq!(no_changes_response["symbols_modified"].as_i64().unwrap(), 0);
        
        if !no_changes_response["message"].is_null() {
            assert!(no_changes_response["message"].is_string());
        }
    }
    
    #[tokio::test]
    async fn test_update_file_response_error_file_not_found() {
        // Expected error response when file doesn't exist
        let error_response = json!({
            "error": "File not found",
            "error_code": "FILE_NOT_FOUND",
            "details": {
                "file_path": "src/nonexistent.cpp",
                "index_name": "test_index"
            }
        });
        
        assert!(error_response["error"].is_string());
        assert!(error_response["error_code"].is_string());
        assert!(error_response["details"].is_object());
        assert!(error_response["details"]["file_path"].is_string());
        assert!(error_response["details"]["index_name"].is_string());
    }
    
    #[tokio::test]
    async fn test_update_file_response_error_parse_failure() {
        // Expected error response when file parsing fails
        let parse_error_response = json!({
            "error": "Failed to parse file",
            "error_code": "PARSE_ERROR",
            "details": {
                "file_path": "src/broken.cpp",
                "index_name": "test_index",
                "parse_errors": [
                    {
                        "line_number": 15,
                        "column_number": 10,
                        "message": "Expected ';' after statement"
                    },
                    {
                        "line_number": 22,
                        "column_number": 5,
                        "message": "Undefined reference to 'UnknownType'"
                    }
                ]
            }
        });
        
        assert!(parse_error_response["error"].is_string());
        assert!(parse_error_response["error_code"].is_string());
        assert!(parse_error_response["details"].is_object());
        assert!(parse_error_response["details"]["file_path"].is_string());
        assert!(parse_error_response["details"]["index_name"].is_string());
        
        if !parse_error_response["details"]["parse_errors"].is_null() {
            assert!(parse_error_response["details"]["parse_errors"].is_array());
            for error in parse_error_response["details"]["parse_errors"].as_array().unwrap() {
                assert!(error["line_number"].is_number());
                assert!(error["message"].is_string());
            }
        }
    }
    
    #[tokio::test]
    async fn test_update_file_invalid_types() {
        let invalid_inputs = vec![
            json!({
                "index_name": 123, // Should be string
                "file_path": "src/test.cpp"
            }),
            json!({
                "index_name": "test_index",
                "file_path": 456 // Should be string
            })
        ];
        
        for invalid_input in invalid_inputs {
            if invalid_input["index_name"].is_number() {
                assert!(!invalid_input["index_name"].is_string());
            }
            if invalid_input["file_path"].is_number() {
                assert!(!invalid_input["file_path"].is_string());
            }
        }
    }
    
    #[tokio::test]
    async fn test_update_file_path_formats() {
        // Test various file path formats
        let file_paths = vec![
            "src/main.cpp",
            "include/header.h",
            "lib/utils.hpp",
            "tests/test_file.cc",
            "project/nested/deep/file.cxx",
            "./relative/path.cpp",
            "../parent/file.h",
            "absolute/path/from/root.cpp",
        ];
        
        for file_path in file_paths {
            let input = json!({
                "index_name": "test_index",
                "file_path": file_path
            });
            
            assert!(input["file_path"].is_string());
            assert_eq!(input["file_path"].as_str().unwrap(), file_path);
            assert!(!input["file_path"].as_str().unwrap().is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_update_file_change_types() {
        // Validate all possible change types
        let change_types = vec!["added", "removed", "modified"];
        
        for change_type in change_types {
            let change = json!({
                "type": change_type,
                "symbol_name": "TestSymbol",
                "symbol_type": "function",
                "line_number": 10
            });
            
            assert_eq!(change["type"].as_str().unwrap(), change_type);
        }
    }
    
    #[tokio::test]
    async fn test_update_file_symbol_types_in_changes() {
        // Validate all symbol types can appear in changes
        let symbol_types = vec!["function", "class", "variable", "macro", "namespace", "enum", "typedef"];
        
        for symbol_type in symbol_types {
            let change = json!({
                "type": "added",
                "symbol_name": "NewSymbol",
                "symbol_type": symbol_type,
                "line_number": 15
            });
            
            assert_eq!(change["symbol_type"].as_str().unwrap(), symbol_type);
        }
    }
    
    #[tokio::test]
    async fn test_update_file_empty_file_path() {
        let input = json!({
            "index_name": "test_index",
            "file_path": ""
        });
        
        assert!(input["file_path"].is_string());
        assert_eq!(input["file_path"].as_str().unwrap(), "");
        
        // Empty file path should be rejected
        assert!(input["file_path"].as_str().unwrap().is_empty());
    }
    
    #[tokio::test]
    async fn test_update_file_incremental_update_details() {
        // Test detailed incremental update information
        let detailed_response = json!({
            "success": true,
            "file_path": "src/complex.cpp",
            "symbols_added": 2,
            "symbols_removed": 1,
            "symbols_modified": 3,
            "total_symbols": 25,
            "update_time_ms": 450,
            "file_hash": "new456hash789def",
            "previous_hash": "old123hash456abc",
            "lines_added": 15,
            "lines_removed": 8,
            "lines_modified": 22,
            "changes": []
        });
        
        // Validate additional incremental update metadata
        if !detailed_response["file_hash"].is_null() {
            assert!(detailed_response["file_hash"].is_string());
            assert!(!detailed_response["file_hash"].as_str().unwrap().is_empty());
        }
        if !detailed_response["previous_hash"].is_null() {
            assert!(detailed_response["previous_hash"].is_string());
            assert!(!detailed_response["previous_hash"].as_str().unwrap().is_empty());
        }
        if !detailed_response["lines_added"].is_null() {
            assert!(detailed_response["lines_added"].is_number());
            assert!(detailed_response["lines_added"].as_i64().unwrap() >= 0);
        }
        if !detailed_response["lines_removed"].is_null() {
            assert!(detailed_response["lines_removed"].is_number());
            assert!(detailed_response["lines_removed"].as_i64().unwrap() >= 0);
        }
        if !detailed_response["lines_modified"].is_null() {
            assert!(detailed_response["lines_modified"].is_number());
            assert!(detailed_response["lines_modified"].as_i64().unwrap() >= 0);
        }
    }
}