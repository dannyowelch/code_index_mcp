#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdbool.h>

// C-style enumerations
typedef enum {
    OPERATION_ADD = 0,
    OPERATION_SUBTRACT,
    OPERATION_MULTIPLY,
    OPERATION_DIVIDE,
    OPERATION_MAX
} operation_type_t;

// C-style structures
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

// Function prototypes
double calculate_distance(const point_t* p1, const point_t* p2);
bool point_in_circle(const point_t* point, const circle_t* circle);
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

// Function implementations
double calculate_distance(const point_t* p1, const point_t* p2) {
    if (!p1 || !p2) {
        return -1.0; // Error indicator
    }
    
    double dx = p2->x - p1->x;
    double dy = p2->y - p1->y;
    return sqrt(dx * dx + dy * dy);
}

bool point_in_circle(const point_t* point, const circle_t* circle) {
    if (!point || !circle) {
        return false;
    }
    
    double distance = calculate_distance(point, &circle->center);
    return distance <= circle->radius;
}

circle_t* create_circle(double x, double y, double radius) {
    if (radius <= 0.0) {
        return NULL;
    }
    
    circle_t* circle = (circle_t*)malloc(sizeof(circle_t));
    if (!circle) {
        return NULL;
    }
    
    circle->center.x = x;
    circle->center.y = y;
    circle->radius = radius;
    
    return circle;
}

void destroy_circle(circle_t* circle) {
    if (circle) {
        free(circle);
    }
}

void print_circle_info(const circle_t* circle) {
    if (!circle) {
        printf("Invalid circle\n");
        return;
    }
    
    printf("Circle: center=(%.2f, %.2f), radius=%.2f\n", 
           circle->center.x, circle->center.y, circle->radius);
    printf("Area: %.2f, Circumference: %.2f\n",
           calculate_circle_area(circle->radius),
           calculate_circle_circumference(circle->radius));
}

calculation_node_t* create_calculation_node(double value, operation_type_t op) {
    calculation_node_t* node = (calculation_node_t*)malloc(sizeof(calculation_node_t));
    if (!node) {
        return NULL;
    }
    
    node->value = value;
    node->operation = op;
    node->next = NULL;
    
    return node;
}

void add_calculation(calculation_node_t** head, double value, operation_type_t op) {
    if (!head) {
        return;
    }
    
    calculation_node_t* new_node = create_calculation_node(value, op);
    if (!new_node) {
        return;
    }
    
    new_node->next = *head;
    *head = new_node;
}

void free_calculation_list(calculation_node_t* head) {
    calculation_node_t* current = head;
    while (current) {
        calculation_node_t* temp = current;
        current = current->next;
        free(temp);
    }
}

size_t count_calculations(const calculation_node_t* head) {
    size_t count = 0;
    const calculation_node_t* current = head;
    
    while (current) {
        count++;
        current = current->next;
    }
    
    return count;
}

void print_calculation_history(const calculation_node_t* head) {
    if (!head) {
        printf("No calculation history\n");
        return;
    }
    
    printf("Calculation History:\n");
    const calculation_node_t* current = head;
    size_t index = 1;
    
    while (current) {
        printf("%zu. %.2f (%s)\n", 
               index++, 
               current->value, 
               operation_to_string(current->operation));
        current = current->next;
    }
}

char* operation_to_string(operation_type_t op) {
    switch (op) {
        case OPERATION_ADD:      return "Addition";
        case OPERATION_SUBTRACT: return "Subtraction";
        case OPERATION_MULTIPLY: return "Multiplication";
        case OPERATION_DIVIDE:   return "Division";
        default:                 return "Unknown";
    }
}

bool is_valid_double_string(const char* str) {
    if (!str || strlen(str) == 0) {
        return false;
    }
    
    char* endptr;
    strtod(str, &endptr);
    
    // Check if the entire string was consumed
    return *endptr == '\0';
}

double safe_string_to_double(const char* str, bool* success) {
    if (success) {
        *success = false;
    }
    
    if (!is_valid_double_string(str)) {
        return 0.0;
    }
    
    double result = strtod(str, NULL);
    if (success) {
        *success = true;
    }
    
    return result;
}

double safe_divide(double a, double b, bool* success) {
    if (success) {
        *success = false;
    }
    
    if (fabs(b) < 1e-9) {  // Avoid division by zero
        return 0.0;
    }
    
    if (success) {
        *success = true;
    }
    
    return a / b;
}

double calculate_circle_area(double radius) {
    if (radius <= 0.0) {
        return 0.0;
    }
    
    return M_PI * radius * radius;
}

double calculate_circle_circumference(double radius) {
    if (radius <= 0.0) {
        return 0.0;
    }
    
    return 2.0 * M_PI * radius;
}