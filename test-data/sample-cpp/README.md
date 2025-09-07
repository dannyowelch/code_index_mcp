# Sample C++ Project for Indexing Tests

This is a test C++ codebase designed to validate the C++ indexing capabilities of the MCP server. It includes various C++ language features and constructs to ensure comprehensive parsing and symbol extraction.

## Project Structure

```
sample-cpp/
├── CMakeLists.txt          # Build configuration
├── README.md              # This file
├── include/               # Header files
│   ├── calculator.h       # Calculator class with modern C++ features
│   └── advanced_math.h    # Advanced C++17/20 features (templates, concepts)
├── src/                   # Source files
│   ├── calculator.cpp     # Calculator implementation
│   └── main.cpp          # Main application
└── lib/                   # C library files
    ├── utils.h           # C utility functions header
    └── utils.c           # C utility functions implementation
```

## Language Features Tested

### C++ Features
- **Classes and Inheritance**: `Operation`, `Addition`, `Subtraction`, `Calculator`
- **Templates**: Class templates, function templates, variadic templates
- **Modern C++ (C++11/14/17/20)**:
  - Smart pointers (`std::unique_ptr`)
  - Move semantics and perfect forwarding
  - Range-based for loops
  - Auto type deduction
  - Lambda expressions
  - Concepts (C++20)
  - Constexpr functions
  - Inline variables
- **Standard Library**: Extensive use of STL containers and algorithms
- **Namespaces**: Nested namespaces (`math::advanced`, `math::utils`)
- **Exception Handling**: Custom exceptions and RAII
- **Operator Overloading**: Matrix operations
- **Type Traits and SFINAE**: Template metaprogramming

### C Features
- **Structures and Enums**: `point_t`, `circle_t`, `operation_type_t`
- **Function Pointers**: Implicit in callback patterns
- **Dynamic Memory Management**: `malloc`, `free`
- **Linked Lists**: `calculation_node_t`
- **Preprocessor**: Include guards, conditional compilation

## Symbol Types for Testing

This project includes these symbol types for comprehensive indexing:

1. **Classes**: `Operation`, `Addition`, `Subtraction`, `Calculator`, `Matrix`
2. **Functions**: `calculate`, `execute`, `getName`, `roundToPrecision`
3. **Variables**: `history_`, `operations_`, `PI`, `E`
4. **Namespaces**: `math`, `math::advanced`, `math::utils`
5. **Templates**: `Matrix<T, Rows, Cols>`, `sum<T, Args...>`
6. **Typedefs/Using**: `OperationPtr`, `Matrix2x2f`
7. **Enums**: `operation_type_t`
8. **Structs**: `point_t`, `circle_t`, `calculation_node_t`
9. **Macros**: `M_PI` (conditional definition)
10. **Constants**: `PI`, `GOLDEN_RATIO`, `SQRT_2`

## Building

```bash
mkdir build
cd build
cmake ..
make
```

## Running

```bash
./CalculatorSample
```

This will start an interactive calculator that demonstrates the various implemented features.

## Testing the Indexer

This codebase is designed to test:

1. **Symbol extraction** from headers and implementation files
2. **Relationship mapping** between classes, functions, and variables
3. **Cross-reference resolution** (calls, inheritances, includes)
4. **Template instantiation** tracking
5. **Namespace resolution**
6. **Macro expansion** and conditional compilation
7. **C/C++ mixed codebases**
8. **Modern C++ feature parsing**