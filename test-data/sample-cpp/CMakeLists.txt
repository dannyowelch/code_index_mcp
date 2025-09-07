cmake_minimum_required(VERSION 3.16)
project(CalculatorSample VERSION 1.0.0 LANGUAGES CXX)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler-specific options
if(MSVC)
    add_compile_options(/W4 /WX)
else()
    add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# Include directories
include_directories(include)

# Source files
set(SOURCES
    src/calculator.cpp
    src/main.cpp
)

set(HEADERS
    include/calculator.h
)

# Create executable
add_executable(${PROJECT_NAME} ${SOURCES} ${HEADERS})

# Create static library for testing
add_library(CalculatorLib STATIC
    src/calculator.cpp
    ${HEADERS}
)

target_include_directories(CalculatorLib PUBLIC include)

# Installation
install(TARGETS ${PROJECT_NAME}
    RUNTIME DESTINATION bin
)

install(TARGETS CalculatorLib
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(FILES ${HEADERS}
    DESTINATION include
)