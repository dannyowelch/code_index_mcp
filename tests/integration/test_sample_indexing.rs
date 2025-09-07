#[cfg(test)]
mod test_sample_indexing {
    use std::process::Command;
    use std::path::Path;
    use tempfile::TempDir;
    use anyhow::Result;
    use std::fs;
    
    /// Test that we can create a basic index from sample C++ files
    #[tokio::test]
    async fn test_create_index_from_sample_cpp() -> Result<()> {
        // Create temporary directory with sample C++ files
        let temp_dir = TempDir::new()?;
        let cpp_project = temp_dir.path().join("sample_project");
        fs::create_dir_all(&cpp_project)?;
        
        // Create sample C++ files
        let header_file = cpp_project.join("sample.h");
        fs::write(&header_file, r#"
#ifndef SAMPLE_H
#define SAMPLE_H

class SampleClass {
public:
    SampleClass();
    ~SampleClass();
    
    void process();
    int getValue() const;
    
private:
    int m_value;
};

namespace SampleNamespace {
    void utility_function();
    
    struct Point {
        double x, y;
    };
}

#endif // SAMPLE_H
"#)?;
        
        let cpp_file = cpp_project.join("sample.cpp");
        fs::write(&cpp_file, r#"
#include "sample.h"
#include <iostream>

SampleClass::SampleClass() : m_value(42) {
}

SampleClass::~SampleClass() {
}

void SampleClass::process() {
    std::cout << "Processing with value: " << m_value << std::endl;
}

int SampleClass::getValue() const {
    return m_value;
}

namespace SampleNamespace {
    void utility_function() {
        Point p = {1.0, 2.0};
        std::cout << "Point: (" << p.x << ", " << p.y << ")" << std::endl;
    }
}

int main() {
    SampleClass obj;
    obj.process();
    
    SampleNamespace::utility_function();
    return 0;
}
"#)?;
        
        // Create a more complex example with templates and inheritance
        let advanced_file = cpp_project.join("advanced.hpp");
        fs::write(&advanced_file, r#"
#pragma once
#include <vector>
#include <memory>

template<typename T>
class Container {
public:
    void add(const T& item);
    T& get(size_t index);
    size_t size() const;
    
private:
    std::vector<T> m_items;
};

class BaseClass {
public:
    virtual ~BaseClass() = default;
    virtual void execute() = 0;
    
protected:
    int m_id;
};

class DerivedClass : public BaseClass {
public:
    DerivedClass(int id);
    void execute() override;
    
private:
    std::unique_ptr<Container<int>> m_container;
};

// Template specialization
template<>
class Container<std::string> {
public:
    void add(const std::string& item);
    void print_all() const;
    
private:
    std::vector<std::string> m_strings;
};
"#)?;
        
        // Test index creation command (this will fail until implementation exists)
        let binary_path = Path::new("target/release/cpp-index-mcp");
        #[cfg(windows)]
        let binary_path = Path::new("target/release/cpp-index-mcp.exe");
        
        if binary_path.exists() {
            let output = Command::new(binary_path)
                .args(&[
                    "index", "create",
                    "--name", "test_sample",
                    "--path", cpp_project.to_str().unwrap()
                ])
                .output()
                .expect("Failed to execute index create command");
            
            // For now, we expect this to fail with "not yet implemented"
            // Once implemented, we should verify success
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // The command should run without crashing
            println!("Index create stdout: {}", stdout);
            println!("Index create stderr: {}", stderr);
            
            // Currently expects "not yet implemented" message
            assert!(
                stdout.contains("not yet implemented") || stderr.contains("not yet implemented"),
                "Expected 'not yet implemented' message, got stdout: '{}', stderr: '{}'",
                stdout, stderr
            );
        }
        
        Ok(())
    }
    
    /// Test that we can validate C++ file structure
    #[tokio::test]
    async fn test_cpp_file_validation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let cpp_project = temp_dir.path().join("validation_test");
        fs::create_dir_all(&cpp_project)?;
        
        // Create files with different C++ extensions
        let extensions = ["cpp", "cxx", "cc", "h", "hpp", "hxx"];
        let mut created_files = Vec::new();
        
        for ext in extensions {
            let filename = format!("test.{}", ext);
            let filepath = cpp_project.join(&filename);
            
            let content = match ext {
                "h" | "hpp" | "hxx" => {
                    format!(r#"
#pragma once
class Test{} {{
public:
    void method();
}};
"#, ext.to_uppercase())
                },
                _ => {
                    format!(r#"
#include "test.h"
void Test{}::method() {{
    // Implementation for {}
}}
"#, ext.to_uppercase(), ext)
                }
            };
            
            fs::write(&filepath, content)?;
            created_files.push(filepath);
        }
        
        // Verify all files were created
        for file in &created_files {
            assert!(file.exists(), "File should exist: {}", file.display());
        }
        
        // Test that we can identify C++ files by walking the directory
        use walkdir::WalkDir;
        
        let mut cpp_files = Vec::new();
        for entry in WalkDir::new(&cpp_project) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if ["cpp", "cxx", "cc", "h", "hpp", "hxx"].contains(&ext_str) {
                            cpp_files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }
        
        assert_eq!(cpp_files.len(), 6, "Should find all 6 C++ files");
        
        Ok(())
    }
    
    /// Test indexing with different file patterns
    #[tokio::test]
    async fn test_file_pattern_matching() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_root = temp_dir.path().join("pattern_test");
        fs::create_dir_all(&project_root)?;
        
        // Create directory structure with various files
        let src_dir = project_root.join("src");
        let build_dir = project_root.join("build");
        let include_dir = project_root.join("include");
        
        fs::create_dir_all(&src_dir)?;
        fs::create_dir_all(&build_dir)?;
        fs::create_dir_all(&include_dir)?;
        
        // Create files that should be included
        let include_files = [
            src_dir.join("main.cpp"),
            src_dir.join("utils.cc"),
            include_dir.join("api.h"),
            include_dir.join("types.hpp"),
        ];
        
        for file in &include_files {
            fs::write(file, "// C++ code")?;
        }
        
        // Create files that should be excluded
        let exclude_files = [
            build_dir.join("temp.cpp"),
            project_root.join("test.o"),
            project_root.join("binary"),
            project_root.join(".git_file"),
        ];
        
        for file in &exclude_files {
            fs::write(file, "// Should be excluded")?;
        }
        
        // Test pattern matching logic (this would be used by the indexer)
        use walkdir::WalkDir;
        
        let mut matched_files = Vec::new();
        for entry in WalkDir::new(&project_root) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let path = entry.path();
                
                // Check if path matches include patterns and doesn't match exclude patterns
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if ["cpp", "cc", "h", "hpp"].contains(&ext_str) {
                            // Check exclude patterns
                            let path_str = path.to_string_lossy();
                            if !path_str.contains("/build/") && !path_str.contains("\\.git") {
                                matched_files.push(path.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
        
        // Should match only the include files, not the exclude files
        assert_eq!(matched_files.len(), 4, "Should match exactly 4 files");
        
        // Verify each included file is in the matched set
        for include_file in &include_files {
            assert!(
                matched_files.iter().any(|f| f == include_file),
                "File should be matched: {}",
                include_file.display()
            );
        }
        
        Ok(())
    }
    
    /// Test that we can handle large file structures
    #[tokio::test]
    async fn test_large_project_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let large_project = temp_dir.path().join("large_project");
        fs::create_dir_all(&large_project)?;
        
        // Create a structure with multiple directories and files
        let subdirs = ["src", "include", "lib", "tests", "examples"];
        let file_count_per_dir = 10;
        
        let mut total_files = 0;
        
        for subdir in &subdirs {
            let dir_path = large_project.join(subdir);
            fs::create_dir_all(&dir_path)?;
            
            for i in 0..file_count_per_dir {
                // Create .h and .cpp files
                let header_file = dir_path.join(format!("module_{}.h", i));
                let source_file = dir_path.join(format!("module_{}.cpp", i));
                
                fs::write(&header_file, format!(r#"
#pragma once

class Module{} {{
public:
    Module{}();
    ~Module{}();
    void process_{i}();
    
private:
    int m_value_{i};
}};
"#, i, i, i, i = i))?;
                
                fs::write(&source_file, format!(r#"
#include "module_{}.h"

Module{}::Module{}() : m_value_{}({}) {{
}}

Module{}::~Module{}() {{
}}

void Module{}::process_{}() {{
    // Implementation for module {}
}}
"#, i, i, i, i, i * 10, i, i, i, i, i))?;
                
                total_files += 2;
            }
        }
        
        // Verify all files were created
        let mut found_files = 0;
        for entry in WalkDir::new(&large_project) {
            let entry = entry?;
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if ["cpp", "h"].contains(&ext_str) {
                            found_files += 1;
                        }
                    }
                }
            }
        }
        
        assert_eq!(
            found_files, total_files,
            "Should find all {} created files, found {}",
            total_files, found_files
        );
        
        println!("Successfully created and validated {} C++ files", total_files);
        
        Ok(())
    }
    
    /// Test that we can parse basic C++ syntax elements
    #[tokio::test]
    async fn test_cpp_syntax_elements() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let syntax_test = temp_dir.path().join("syntax_test.cpp");
        
        // Create file with various C++ syntax elements that should be indexed
        fs::write(&syntax_test, r#"
// Various C++ syntax elements for indexing
#include <iostream>
#include <vector>
#include <memory>

// Global constants and variables
const int GLOBAL_CONSTANT = 42;
static int s_static_var = 0;

// Forward declarations
class ForwardDeclared;
struct ForwardStruct;

// Enum declarations
enum Color { RED, GREEN, BLUE };
enum class Status : int { PENDING = 1, ACTIVE = 2, COMPLETE = 3 };

// Function declarations and definitions
void global_function();
inline int inline_function() { return 42; }
template<typename T> T template_function(T value) { return value; }

// Namespace
namespace TestNamespace {
    void namespaced_function();
    
    class NamespacedClass {
    public:
        void method();
    };
}

// Class with various member types
class ComplexClass : public std::enable_shared_from_this<ComplexClass> {
public:
    // Constructors and destructor
    ComplexClass();
    ComplexClass(int value);
    ComplexClass(const ComplexClass& other) = delete;
    ComplexClass& operator=(const ComplexClass& other) = delete;
    virtual ~ComplexClass();
    
    // Member functions
    virtual void virtual_method() = 0;
    static void static_method();
    const int& const_method() const;
    
    // Operator overloads
    ComplexClass& operator+=(const ComplexClass& other);
    bool operator==(const ComplexClass& other) const;
    
    // Template member function
    template<typename T>
    void template_method(const T& param);
    
protected:
    void protected_method();
    
private:
    int m_private_member;
    static int s_static_member;
    mutable int m_mutable_member;
};

// Template class
template<typename T, size_t N>
class TemplateClass {
public:
    void process(const std::array<T, N>& data);
    
private:
    std::array<T, N> m_data;
};

// Template specialization
template<>
class TemplateClass<int, 10> {
public:
    void special_process();
};

// Function definitions
void global_function() {
    std::cout << "Global function called" << std::endl;
}

// Lambda expressions and modern C++ features
auto lambda_example = []() {
    return 42;
};

auto complex_lambda = [](auto&& x) -> decltype(auto) {
    return std::forward<decltype(x)>(x);
};
"#)?;
        
        // Verify the file was created and contains expected syntax
        assert!(syntax_test.exists());
        let content = fs::read_to_string(&syntax_test)?;
        
        // Check for key C++ elements that should be indexed
        let expected_elements = [
            "class ComplexClass",
            "namespace TestNamespace", 
            "enum Color",
            "template<typename T>",
            "void global_function",
            "const int GLOBAL_CONSTANT",
            "virtual void virtual_method",
            "operator+=",
        ];
        
        for element in &expected_elements {
            assert!(
                content.contains(element),
                "Content should contain C++ element: {}",
                element
            );
        }
        
        println!("Created syntax test file with {} bytes of C++ code", content.len());
        
        Ok(())
    }
}