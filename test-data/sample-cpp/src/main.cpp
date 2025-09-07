#include "calculator.h"
#include <iostream>
#include <iomanip>
#include <string>
#include <exception>

using namespace math;

/// Display menu options to the user
void displayMenu() {
    std::cout << "\n=== Calculator Menu ===\n";
    std::cout << "1. Addition\n";
    std::cout << "2. Subtraction\n";
    std::cout << "3. View History\n";
    std::cout << "4. Clear History\n";
    std::cout << "5. Exit\n";
    std::cout << "Choose an option: ";
}

/// Get user input for numbers
std::pair<double, double> getNumbers() {
    double a, b;
    std::cout << "Enter first number: ";
    std::cin >> a;
    std::cout << "Enter second number: ";
    std::cin >> b;
    return std::make_pair(a, b);
}

/// Display calculation history
void displayHistory(const Calculator& calc) {
    const auto& history = calc.getHistory();
    
    if (history.empty()) {
        std::cout << "No calculations in history.\n";
        return;
    }
    
    std::cout << "\n=== Calculation History ===\n";
    for (size_t i = 0; i < history.size(); ++i) {
        std::cout << std::setw(3) << (i + 1) << ". " 
                  << std::fixed << std::setprecision(2) 
                  << history[i] << "\n";
    }
}

/// Main application function
int main() {
    try {
        Calculator calculator;
        int choice;
        bool running = true;
        
        std::cout << "Welcome to the C++ Calculator!\n";
        std::cout << "Available operations: " << calculator.getOperationCount() << "\n";
        
        // Test template method
        auto processed_int = calculator.processValue(42);
        auto processed_double = calculator.processValue(3.14159);
        
        std::cout << "Template processing test - Int: " << processed_int 
                  << ", Double: " << processed_double << "\n";
        
        while (running) {
            displayMenu();
            std::cin >> choice;
            
            switch (choice) {
                case 1: {
                    auto numbers = getNumbers();
                    try {
                        double result = calculator.calculate("addition", numbers.first, numbers.second);
                        std::cout << "Result: " << std::fixed << std::setprecision(2) 
                                  << numbers.first << " + " << numbers.second 
                                  << " = " << result << "\n";
                    } catch (const std::exception& e) {
                        std::cerr << "Error: " << e.what() << "\n";
                    }
                    break;
                }
                case 2: {
                    auto numbers = getNumbers();
                    try {
                        double result = calculator.calculate("subtraction", numbers.first, numbers.second);
                        std::cout << "Result: " << std::fixed << std::setprecision(2) 
                                  << numbers.first << " - " << numbers.second 
                                  << " = " << result << "\n";
                    } catch (const std::exception& e) {
                        std::cerr << "Error: " << e.what() << "\n";
                    }
                    break;
                }
                case 3:
                    displayHistory(calculator);
                    break;
                case 4:
                    calculator.clearHistory();
                    std::cout << "History cleared.\n";
                    break;
                case 5:
                    running = false;
                    std::cout << "Goodbye!\n";
                    break;
                default:
                    std::cout << "Invalid choice. Please try again.\n";
                    break;
            }
        }
        
        // Test utility functions
        std::cout << "\nUtility function tests:\n";
        std::cout << "PI rounded to 2 places: " 
                  << utils::roundToPrecision(PI, 2) << "\n";
        std::cout << "Is '123.45' a valid number? " 
                  << (utils::isValidNumber("123.45") ? "Yes" : "No") << "\n";
        std::cout << "Is 'abc' a valid number? " 
                  << (utils::isValidNumber("abc") ? "Yes" : "No") << "\n";
        
        auto available_ops = utils::getAvailableOperations();
        std::cout << "Available operations: ";
        for (const auto& op : available_ops) {
            std::cout << op << " ";
        }
        std::cout << "\n";
        
    } catch (const std::exception& e) {
        std::cerr << "Fatal error: " << e.what() << "\n";
        return 1;
    }
    
    return 0;
}