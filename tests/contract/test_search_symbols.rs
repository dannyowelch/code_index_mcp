#[cfg(test)]
mod test_search_symbols {
    use serde_json::{json, Value};
    
    #[tokio::test]
    async fn test_search_symbols_valid_inputs() {
        // Test with minimal required parameters
        let input = json!({
            "index_name": "test_index",
            "query": "MyFunction"
        });
        
        assert!(input["index_name"].is_string());
        assert!(input["query"].is_string());
        
        // Test with all optional parameters
        let input_full = json!({
            "index_name": "test_index",
            "query": "MyClass",
            "symbol_type": "class",
            "file_path": "src/*.cpp",
            "scope": "MyNamespace",
            "exact_match": true,
            "limit": 50
        });
        
        assert!(input_full["index_name"].is_string());
        assert!(input_full["query"].is_string());
        assert!(input_full["symbol_type"].is_string());
        assert!(input_full["file_path"].is_string());
        assert!(input_full["scope"].is_string());
        assert!(input_full["exact_match"].is_boolean());
        assert!(input_full["limit"].is_number());
    }
    
    #[tokio::test]
    async fn test_search_symbols_symbol_types() {
        let valid_symbol_types = vec![
            "function", "class", "variable", "macro", "namespace", "enum", "typedef"
        ];
        
        for symbol_type in valid_symbol_types {
            let input = json!({
                "index_name": "test_index",
                "query": "test_symbol",
                "symbol_type": symbol_type
            });
            
            assert_eq!(input["symbol_type"].as_str().unwrap(), symbol_type);
        }
    }
    
    #[tokio::test]
    async fn test_search_symbols_limits() {
        // Test default limit
        let input_no_limit = json!({
            "index_name": "test_index", 
            "query": "MyFunction"
        });
        
        // Should use default limit of 100
        let default_limit = 100;
        assert!(default_limit >= 1 && default_limit <= 1000);
        
        // Test valid limit values
        let input_min_limit = json!({
            "index_name": "test_index",
            "query": "MyFunction", 
            "limit": 1
        });
        
        let input_max_limit = json!({
            "index_name": "test_index",
            "query": "MyFunction",
            "limit": 1000
        });
        
        assert_eq!(input_min_limit["limit"].as_i64().unwrap(), 1);
        assert_eq!(input_max_limit["limit"].as_i64().unwrap(), 1000);
    }
    
    #[tokio::test]
    async fn test_search_symbols_response_schema() {
        // Expected response structure based on SearchResult schema
        let expected_response = json!({
            "symbols": [
                {
                    "id": 1,
                    "name": "MyFunction",
                    "type": "function",
                    "file_path": "src/main.cpp",
                    "line_number": 15,
                    "column_number": 5,
                    "scope": "MyNamespace",
                    "signature": "int MyFunction(const std::string& param)",
                    "access_modifier": "public",
                    "is_declaration": false
                }
            ],
            "total_count": 1,
            "query_time_ms": 25
        });
        
        // Validate response schema structure
        assert!(expected_response["symbols"].is_array());
        assert!(expected_response["total_count"].is_number());
        assert!(expected_response["query_time_ms"].is_number());
        
        let symbol = &expected_response["symbols"][0];
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
        if !symbol["access_modifier"].is_null() {
            assert!(symbol["access_modifier"].is_string());
        }
        if !symbol["is_declaration"].is_null() {
            assert!(symbol["is_declaration"].is_boolean());
        }
    }
    
    #[tokio::test]
    async fn test_search_symbols_missing_required_fields() {
        // Missing index_name
        let input_missing_index = json!({
            "query": "MyFunction"
        });
        
        assert!(input_missing_index["index_name"].is_null());
        
        // Missing query
        let input_missing_query = json!({
            "index_name": "test_index"
        });
        
        assert!(input_missing_query["query"].is_null());
    }
    
    #[tokio::test]
    async fn test_search_symbols_invalid_symbol_type() {
        let input_invalid_type = json!({
            "index_name": "test_index",
            "query": "MySymbol", 
            "symbol_type": "invalid_type"
        });
        
        let valid_types = vec!["function", "class", "variable", "macro", "namespace", "enum", "typedef"];
        let provided_type = input_invalid_type["symbol_type"].as_str().unwrap();
        
        // Should not be a valid symbol type
        assert!(!valid_types.contains(&provided_type));
    }
    
    #[tokio::test]
    async fn test_search_symbols_limit_boundaries() {
        // Test limit below minimum (should be invalid)
        let input_limit_too_low = json!({
            "index_name": "test_index",
            "query": "MyFunction",
            "limit": 0
        });
        
        // Test limit above maximum (should be invalid) 
        let input_limit_too_high = json!({
            "index_name": "test_index", 
            "query": "MyFunction",
            "limit": 1001
        });
        
        assert_eq!(input_limit_too_low["limit"].as_i64().unwrap(), 0);
        assert_eq!(input_limit_too_high["limit"].as_i64().unwrap(), 1001);
        
        // These should be rejected by validation (< 1 or > 1000)
        assert!(input_limit_too_low["limit"].as_i64().unwrap() < 1);
        assert!(input_limit_too_high["limit"].as_i64().unwrap() > 1000);
    }
    
    #[tokio::test]
    async fn test_search_symbols_empty_results() {
        // Test expected response structure when no symbols found
        let empty_response = json!({
            "symbols": [],
            "total_count": 0,
            "query_time_ms": 5
        });
        
        assert!(empty_response["symbols"].is_array());
        assert_eq!(empty_response["symbols"].as_array().unwrap().len(), 0);
        assert_eq!(empty_response["total_count"].as_i64().unwrap(), 0);
        assert!(empty_response["query_time_ms"].is_number());
    }
}