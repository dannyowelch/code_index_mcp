#ifndef UTILS_H
#define UTILS_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Constants
#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

// Type definitions
typedef enum {
    OPERATION_ADD = 0,
    OPERATION_SUBTRACT,
    OPERATION_MULTIPLY,
    OPERATION_DIVIDE,
    OPERATION_MAX
} operation_type_t;

typedef struct {
    double x;
    double y;
} point_t;

typedef struct {
    point_t center;
    double radius;
} circle_t;

typedef struct calculation_node {
    double value;
    operation_type_t operation;
    struct calculation_node* next;
} calculation_node_t;

// Geometry functions
double calculate_distance(const point_t* p1, const point_t* p2);
bool point_in_circle(const point_t* point, const circle_t* circle);

// Circle management
circle_t* create_circle(double x, double y, double radius);
void destroy_circle(circle_t* circle);
void print_circle_info(const circle_t* circle);

// Calculation history management
calculation_node_t* create_calculation_node(double value, operation_type_t op);
void add_calculation(calculation_node_t** head, double value, operation_type_t op);
void free_calculation_list(calculation_node_t* head);
size_t count_calculations(const calculation_node_t* head);
void print_calculation_history(const calculation_node_t* head);

// String utilities
char* operation_to_string(operation_type_t op);
bool is_valid_double_string(const char* str);
double safe_string_to_double(const char* str, bool* success);

// Math utilities
double safe_divide(double a, double b, bool* success);
double calculate_circle_area(double radius);
double calculate_circle_circumference(double radius);

#ifdef __cplusplus
}
#endif

#endif // UTILS_H