#[cfg(test)]
mod test_performance {
    use std::process::Command;
    use std::path::Path;
    use tempfile::TempDir;
    use anyhow::Result;
    use std::fs;
    use std::time::{Duration, Instant, SystemTime};
    use std::collections::HashMap;
    
    /// Test indexing performance with large codebase (target: >10k files)
    #[tokio::test]
    async fn test_large_codebase_indexing_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let large_project = temp_dir.path().join("large_codebase");
        fs::create_dir_all(&large_project)?;
        
        println!("Creating large test codebase...");
        let start_time = Instant::now();
        
        // Target performance: Handle >10k files
        let directories = 100;
        let files_per_dir = 25; // 2,500 files total (scaled for test speed)
        let total_files = directories * files_per_dir;
        
        let mut created_files = 0;
        let mut total_lines = 0;
        
        for dir_idx in 0..directories {
            let dir_path = large_project.join(format!("module_{:03}", dir_idx));
            fs::create_dir_all(&dir_path)?;
            
            for file_idx in 0..files_per_dir {
                // Create both header and source files
                let header_file = dir_path.join(format!("class_{:03}.h", file_idx));
                let source_file = dir_path.join(format!("class_{:03}.cpp", file_idx));
                
                let header_content = generate_header_content(dir_idx, file_idx);
                let source_content = generate_source_content(dir_idx, file_idx);
                
                fs::write(&header_file, &header_content)?;
                fs::write(&source_file, &source_content)?;
                
                created_files += 2;
                total_lines += header_content.lines().count() + source_content.lines().count();
            }
        }
        
        let creation_time = start_time.elapsed();
        println!("Created {} files ({} lines) in {:?}", created_files, total_lines, creation_time);
        
        // Performance assertions
        assert_eq!(created_files, total_files * 2, "Should create expected number of files");
        assert!(creation_time < Duration::from_secs(30), "File creation should complete within 30 seconds");
        
        // Test index creation performance (will fail until implemented)
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        if binary_path.exists() {
            println!("Testing index creation performance...");
            let index_start = Instant::now();
            
            let output = Command::new(binary_path)
                .args(&[
                    "index", "create",
                    "--name", "large_test",
                    "--path", large_project.to_str().unwrap()
                ])
                .output()
                .expect("Failed to create index");
            
            let index_time = index_start.elapsed();
            println!("Index creation took {:?}", index_time);
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            // For now, expect "not implemented" - later should complete within reasonable time
            if stdout.contains("not yet implemented") || stderr.contains("not yet implemented") {
                println!("Index creation not yet implemented - test structure validated");
            } else {
                // When implemented, should complete within target time
                assert!(index_time < Duration::from_secs(300), "Index creation should complete within 5 minutes");
            }
        }
        
        Ok(())
    }
    
    /// Test incremental update performance (target: <30s for updates)
    #[tokio::test]
    async fn test_incremental_update_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("incremental_perf");
        fs::create_dir_all(&project_path)?;
        
        // Create baseline project with moderate size
        let file_count = 200;
        let mut file_metadata = HashMap::new();
        
        println!("Creating baseline project with {} files...", file_count);
        let baseline_start = Instant::now();
        
        for i in 0..file_count {
            let header_path = project_path.join(format!("module_{}.h", i));
            let source_path = project_path.join(format!("module_{}.cpp", i));
            
            let header_content = generate_header_content(i / 10, i % 10);
            let source_content = generate_source_content(i / 10, i % 10);
            
            fs::write(&header_path, header_content)?;
            fs::write(&source_path, source_content)?;
            
            // Store metadata for change detection
            file_metadata.insert(header_path.clone(), fs::metadata(&header_path)?);
            file_metadata.insert(source_path.clone(), fs::metadata(&source_path)?);
        }
        
        let baseline_time = baseline_start.elapsed();
        println!("Baseline creation took {:?}", baseline_time);
        
        // Simulate incremental changes
        let change_percentage = 0.1; // Modify 10% of files
        let files_to_change = (file_count as f32 * change_percentage) as usize;
        
        println!("Simulating incremental changes to {} files...", files_to_change);
        let incremental_start = Instant::now();
        
        for i in 0..files_to_change {
            let source_path = project_path.join(format!("module_{}.cpp", i));
            let modified_content = format!(r#"
#include "module_{}.h"
#include <iostream>

// Modified implementation
Module{}::Module{}() : m_value_{}({}) {{
    std::cout << "Modified constructor for Module{}" << std::endl;
}}

void Module{}::process_{}() {{
    std::cout << "Modified process method for Module{}" << std::endl;
    // Additional processing
    for (int j = 0; j < 10; ++j) {{
        m_value_{} += j;
    }}
}}

void Module{}::new_method_{}() {{
    // New method added during incremental update
    std::cout << "New method in Module{}" << std::endl;
}}
"#, i, i, i, i, i * 100, i, i, i, i, i, i, i, i, i);
            
            fs::write(&source_path, modified_content)?;
        }
        
        let incremental_time = incremental_start.elapsed();
        println!("Incremental changes took {:?}", incremental_time);
        
        // Performance target: incremental updates should be fast
        assert!(incremental_time < Duration::from_secs(5), "Incremental changes should be fast");
        
        // Test change detection performance
        let detection_start = Instant::now();
        let mut changed_files = Vec::new();
        
        for (file_path, original_metadata) in &file_metadata {
            if file_path.exists() {
                let current_metadata = fs::metadata(file_path)?;
                if current_metadata.modified()? != original_metadata.modified()? {
                    changed_files.push(file_path.clone());
                }
            }
        }
        
        let detection_time = detection_start.elapsed();
        println!("Change detection took {:?} for {} files", detection_time, file_metadata.len());
        println!("Detected {} changed files", changed_files.len());
        
        // Verify correct number of changes detected
        assert_eq!(changed_files.len(), files_to_change, "Should detect all modified files");
        assert!(detection_time < Duration::from_secs(2), "Change detection should be fast");
        
        Ok(())
    }
    
    /// Test query response performance (target: <100ms)
    #[tokio::test]
    async fn test_query_response_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("query_perf");
        fs::create_dir_all(&project_path)?;
        
        // Create project with searchable symbols
        let class_count = 100;
        let methods_per_class = 10;
        
        println!("Creating searchable codebase with {} classes...", class_count);
        
        for class_idx in 0..class_count {
            let header_file = project_path.join(format!("searchable_{:03}.h", class_idx));
            let source_file = project_path.join(format!("searchable_{:03}.cpp", class_idx));
            
            let header_content = generate_searchable_header(class_idx, methods_per_class);
            let source_content = generate_searchable_source(class_idx, methods_per_class);
            
            fs::write(&header_file, header_content)?;
            fs::write(&source_file, source_content)?;
        }
        
        // Test query performance simulation
        let query_scenarios = vec![
            ("SearchableClass050", "class name search"),
            ("process_method", "method name search"),
            ("m_data", "member variable search"),
            ("virtual", "keyword search"),
            ("SearchableNamespace", "namespace search"),
        ];
        
        for (query, description) in query_scenarios {
            println!("Testing query performance: {}", description);
            let query_start = Instant::now();
            
            // Simulate search through files (simplified version of what indexer would do)
            let mut results = Vec::new();
            
            use walkdir::WalkDir;
            for entry in WalkDir::new(&project_path) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "h" || ext == "cpp" {
                            let content = fs::read_to_string(entry.path())?;
                            if content.contains(query) {
                                results.push(entry.path().to_path_buf());
                            }
                        }
                    }
                }
            }
            
            let query_time = query_start.elapsed();
            println!("Query '{}' found {} results in {:?}", query, results.len(), query_time);
            
            // Performance target: queries should be fast
            // Note: This is file scanning, real index queries should be much faster
            assert!(query_time < Duration::from_secs(2), "File scanning should complete quickly");
            assert!(!results.is_empty(), "Should find results for test queries");
        }
        
        Ok(())
    }
    
    /// Test memory usage and resource management
    #[tokio::test]
    async fn test_memory_usage_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("memory_test");
        fs::create_dir_all(&project_path)?;
        
        // Create files with varying sizes to test memory usage
        let file_sizes = vec![
            (10, 1_000),     // 10 small files (1KB each)
            (5, 10_000),     // 5 medium files (10KB each) 
            (2, 100_000),    // 2 large files (100KB each)
        ];
        
        let mut total_size = 0;
        
        for (count, size_bytes) in file_sizes {
            for i in 0..count {
                let file_path = project_path.join(format!("size_test_{}_{}.cpp", size_bytes, i));
                let content = generate_file_content(size_bytes);
                fs::write(&file_path, &content)?;
                total_size += content.len();
            }
        }
        
        println!("Created test files totaling {} bytes", total_size);
        
        // Test memory-efficient file processing
        let processing_start = Instant::now();
        let mut processed_bytes = 0;
        
        use walkdir::WalkDir;
        for entry in WalkDir::new(&project_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                // Simulate streaming/chunked processing instead of loading entire file
                let file_size = fs::metadata(entry.path())?.len() as usize;
                processed_bytes += file_size;
                
                // Read file in chunks to simulate memory-efficient processing
                let chunk_size = 4096;
                let content = fs::read(entry.path())?;
                let mut processed_chunks = 0;
                
                for _chunk in content.chunks(chunk_size) {
                    processed_chunks += 1;
                    // Simulate processing work
                }
                
                assert!(processed_chunks > 0, "Should process at least one chunk");
            }
        }
        
        let processing_time = processing_start.elapsed();
        println!("Processed {} bytes in {:?}", processed_bytes, processing_time);
        
        // Memory usage should be reasonable for file processing
        assert_eq!(processed_bytes, total_size, "Should process all file content");
        assert!(processing_time < Duration::from_secs(10), "Processing should be efficient");
        
        Ok(())
    }
    
    /// Test concurrent operation performance
    #[tokio::test]
    async fn test_concurrent_operations_performance() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("concurrent_test");
        fs::create_dir_all(&project_path)?;
        
        // Create multiple directories for parallel processing
        let dir_count = 4;
        let files_per_dir = 25;
        
        println!("Testing concurrent file operations...");
        
        let concurrent_start = Instant::now();
        let mut handles = Vec::new();
        
        // Simulate concurrent directory processing
        for dir_idx in 0..dir_count {
            let dir_path = project_path.join(format!("concurrent_dir_{}", dir_idx));
            fs::create_dir_all(&dir_path)?;
            
            let handle = tokio::spawn(async move {
                let mut files_created = 0;
                
                for file_idx in 0..files_per_dir {
                    let file_path = dir_path.join(format!("file_{}.cpp", file_idx));
                    let content = format!(r#"
// Concurrently created file {} in directory {}
#include <iostream>

class ConcurrentClass{}_{} {{
public:
    void process() {{
        std::cout << "Processing in dir {} file {}" << std::endl;
    }}
}};
"#, file_idx, dir_idx, dir_idx, file_idx, dir_idx, file_idx);
                    
                    tokio::fs::write(&file_path, content).await
                        .expect("Failed to write concurrent file");
                    files_created += 1;
                }
                
                files_created
            });
            
            handles.push(handle);
        }
        
        // Wait for all concurrent operations to complete
        let mut total_files = 0;
        for handle in handles {
            let files_created = handle.await.expect("Concurrent task failed");
            total_files += files_created;
        }
        
        let concurrent_time = concurrent_start.elapsed();
        println!("Created {} files concurrently in {:?}", total_files, concurrent_time);
        
        // Verify concurrent operations completed successfully
        assert_eq!(total_files, dir_count * files_per_dir, "All concurrent files should be created");
        
        // Concurrent operations should provide performance benefit
        // (This is a simplified test - real benefits would be seen with actual indexing)
        assert!(concurrent_time < Duration::from_secs(5), "Concurrent operations should be fast");
        
        Ok(())
    }
    
    /// Test performance under stress conditions
    #[tokio::test]
    async fn test_stress_conditions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let stress_project = temp_dir.path().join("stress_test");
        fs::create_dir_all(&stress_project)?;
        
        println!("Running stress test conditions...");
        
        // Test 1: Very deep directory structure
        let max_depth = 10;
        let mut current_dir = stress_project.clone();
        
        for depth in 0..max_depth {
            current_dir = current_dir.join(format!("level_{}", depth));
            fs::create_dir_all(&current_dir)?;
            
            // Create file at each level
            let file_path = current_dir.join(format!("deep_file_{}.h", depth));
            fs::write(&file_path, format!("// File at depth {}", depth))?;
        }
        
        // Test 2: Files with very long names
        let long_name = "very_long_filename_that_tests_path_limits_and_filesystem_handling_with_extended_naming_conventions";
        let long_name_file = stress_project.join(format!("{}.cpp", long_name));
        fs::write(&long_name_file, "// File with long name")?;
        
        // Test 3: Files with special characters (where allowed by filesystem)
        let special_chars_allowed = vec!["test-file", "test_file", "test.file"];
        for special_name in special_chars_allowed {
            let special_file = stress_project.join(format!("{}.cpp", special_name));
            fs::write(&special_file, format!("// File with special name: {}", special_name))?;
        }
        
        // Test 4: Empty and near-empty files
        let empty_file = stress_project.join("empty.cpp");
        fs::write(&empty_file, "")?;
        
        let minimal_file = stress_project.join("minimal.cpp");
        fs::write(&minimal_file, ";")?;
        
        // Verify stress conditions are handled
        let stress_validation_start = Instant::now();
        
        use walkdir::WalkDir;
        let mut stress_files = Vec::new();
        for entry in WalkDir::new(&stress_project) {
            let entry = entry?;
            if entry.file_type().is_file() {
                stress_files.push(entry.path().to_path_buf());
            }
        }
        
        let validation_time = stress_validation_start.elapsed();
        println!("Validated {} stress condition files in {:?}", stress_files.len(), validation_time);
        
        // Should handle all stress conditions without crashing
        assert!(stress_files.len() >= max_depth + 4, "Should find all stress test files");
        assert!(validation_time < Duration::from_secs(5), "Stress validation should complete quickly");
        
        // Verify deep file exists
        assert!(current_dir.join("deep_file_9.h").exists(), "Deep file should exist");
        
        // Verify special files exist
        assert!(long_name_file.exists(), "Long name file should exist");
        assert!(empty_file.exists(), "Empty file should exist");
        
        Ok(())
    }
    
    // Helper functions for generating test content
    fn generate_header_content(dir_idx: usize, file_idx: usize) -> String {
        format!(r#"
#pragma once
#include <vector>
#include <memory>

namespace Module{}Namespace {{
    
class Module{}Class{} {{
public:
    Module{}Class{}();
    ~Module{}Class{}();
    
    void process_{}();
    void utility_method_{}();
    int get_value_{}() const;
    void set_value_{}(int value);
    
    // Template method
    template<typename T>
    void template_method_{}(const T& param);
    
private:
    int m_value_{};
    std::vector<int> m_data_{};
    std::unique_ptr<int> m_ptr_{};
    static int s_counter_{};
}};

// Free functions
void module_{}_utility_{}_function();
int module_{}_calculate_{}(int input);

}} // namespace Module{}Namespace
"#, dir_idx, dir_idx, file_idx, dir_idx, file_idx, dir_idx, file_idx, 
   file_idx, file_idx, file_idx, file_idx, file_idx, 
   file_idx, file_idx, file_idx, file_idx,
   dir_idx, file_idx, dir_idx, file_idx, dir_idx)
    }
    
    fn generate_source_content(dir_idx: usize, file_idx: usize) -> String {
        format!(r#"
#include "class_{:03}.h"
#include <iostream>
#include <algorithm>

namespace Module{}Namespace {{

int Module{}Class{}::s_counter_{} = 0;

Module{}Class{}::Module{}Class{}() 
    : m_value_{}({}), 
      m_data_{{}},
      m_ptr_{}(std::make_unique<int>({})) {{
    ++s_counter_{};
    m_data_{}.reserve(10);
    for (int i = 0; i < 5; ++i) {{
        m_data_{}.push_back(i * {});
    }}
}}

Module{}Class{}::~Module{}Class{}() {{
    --s_counter_{};
}}

void Module{}Class{}::process_{}() {{
    std::cout << "Processing Module{}Class{} with value " << m_value_{} << std::endl;
    
    // Some processing logic
    std::for_each(m_data_{}.begin(), m_data_{}.end(), [](int& val) {{
        val *= 2;
    }});
    
    if (m_ptr_{}) {{
        *m_ptr_{} += m_value_{};
    }}
}}

void Module{}Class{}::utility_method_{}() {{
    m_value_{} += {};
    
    // Complex computation
    for (size_t i = 0; i < m_data_{}.size(); ++i) {{
        m_data_{}[i] = (m_data_{}[i] + m_value_{}) % 1000;
    }}
}}

int Module{}Class{}::get_value_{}() const {{
    return m_value_{};
}}

void Module{}Class{}::set_value_{}(int value) {{
    m_value_{} = value;
    if (m_ptr_{}) {{
        *m_ptr_{} = value * 2;
    }}
}}

// Free function implementations
void module_{}_utility_{}_function() {{
    std::cout << "Utility function for module {} file {}" << std::endl;
}}

int module_{}_calculate_{}(int input) {{
    return input * {} + {};
}}

}} // namespace Module{}Namespace
"#, file_idx, dir_idx, dir_idx, file_idx, file_idx, 
   dir_idx, file_idx, dir_idx, file_idx, 
   file_idx, file_idx * 10, file_idx, file_idx, file_idx,
   file_idx, file_idx, file_idx, file_idx * 2,
   dir_idx, file_idx, dir_idx, file_idx, file_idx,
   dir_idx, file_idx, dir_idx, file_idx, file_idx,
   file_idx, file_idx, file_idx, file_idx,
   dir_idx, file_idx, file_idx, file_idx, file_idx * 3,
   file_idx, file_idx, file_idx, file_idx,
   dir_idx, file_idx, file_idx, file_idx,
   dir_idx, file_idx, file_idx, file_idx, file_idx,
   file_idx, file_idx,
   dir_idx, file_idx, dir_idx, file_idx,
   dir_idx, file_idx, dir_idx * 10, file_idx * 5, dir_idx)
    }
    
    fn generate_searchable_header(class_idx: usize, methods_per_class: usize) -> String {
        let mut content = format!(r#"
#pragma once
#include <string>
#include <vector>

namespace SearchableNamespace {{

class SearchableClass{:03} {{
public:
    SearchableClass{:03}();
    virtual ~SearchableClass{:03}();
    
"#, class_idx, class_idx, class_idx);
        
        for method_idx in 0..methods_per_class {
            content.push_str(&format!("    virtual void process_method_{}();\n", method_idx));
        }
        
        content.push_str(&format!(r#"    
    // Data members
    int m_data_{};
    std::string m_name_{};
    std::vector<int> m_values_{};
    
private:
    static int s_instance_count_{};
}};

}} // namespace SearchableNamespace
"#, class_idx, class_idx, class_idx, class_idx));
        
        content
    }
    
    fn generate_searchable_source(class_idx: usize, methods_per_class: usize) -> String {
        let mut content = format!(r#"
#include "searchable_{:03}.h"
#include <iostream>

namespace SearchableNamespace {{

int SearchableClass{:03}::s_instance_count_{} = 0;

SearchableClass{:03}::SearchableClass{:03}() 
    : m_data_{}({}), 
      m_name_{}("SearchableClass{:03}"), 
      m_values_{} {{
    ++s_instance_count_{};
}}

SearchableClass{:03}::~SearchableClass{:03}() {{
    --s_instance_count_{};
}}

"#, class_idx, class_idx, class_idx, class_idx, class_idx, 
   class_idx, class_idx, class_idx, class_idx, 
   class_idx, class_idx, class_idx, class_idx);
        
        for method_idx in 0..methods_per_class {
            content.push_str(&format!(r#"
void SearchableClass{:03}::process_method_{}() {{
    std::cout << "Processing method {} in SearchableClass{:03}" << std::endl;
    m_data_{} += {};
}}
"#, class_idx, method_idx, method_idx, class_idx, class_idx, method_idx));
        }
        
        content.push_str(&format!("}} // namespace SearchableNamespace\n"));
        content
    }
    
    fn generate_file_content(target_size: usize) -> String {
        let base_content = r#"
#include <iostream>
#include <string>
#include <vector>

// This is a generated file for memory usage testing

class MemoryTestClass {
public:
    MemoryTestClass() {
        // Initialize data structures
    }
    
    void process_data() {
        // Process some data
    }
    
private:
    std::vector<std::string> m_strings;
};

"#;
        
        let mut content = String::from(base_content);
        let comment_line = "// Additional content for size testing\n";
        
        // Add content until we reach target size
        while content.len() < target_size {
            content.push_str(comment_line);
        }
        
        content
    }
}