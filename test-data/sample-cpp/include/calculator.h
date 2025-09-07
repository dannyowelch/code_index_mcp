#ifndef CALCULATOR_H
#define CALCULATOR_H

#include <memory>
#include <vector>
#include <string>

namespace math {
    
    /// Abstract base class for mathematical operations
    class Operation {
    public:
        virtual ~Operation() = default;
        virtual double execute(double a, double b) const = 0;
        virtual std::string getName() const = 0;
    };

    /// Addition operation implementation
    class Addition : public Operation {
    public:
        double execute(double a, double b) const override;
        std::string getName() const override;
    };

    /// Subtraction operation implementation  
    class Subtraction : public Operation {
    public:
        double execute(double a, double b) const override;
        std::string getName() const override;
    };

    /// Calculator class that manages operations
    class Calculator {
    private:
        std::vector<std::unique_ptr<Operation>> operations_;
        std::vector<double> history_;
        
    public:
        Calculator();
        ~Calculator();
        
        // Copy constructor and assignment operator
        Calculator(const Calculator& other);
        Calculator& operator=(const Calculator& other);
        
        // Move constructor and assignment operator
        Calculator(Calculator&& other) noexcept;
        Calculator& operator=(Calculator&& other) noexcept;
        
        /// Add a new operation to the calculator
        void addOperation(std::unique_ptr<Operation> op);
        
        /// Execute operation by name
        double calculate(const std::string& operation_name, double a, double b);
        
        /// Get calculation history
        const std::vector<double>& getHistory() const;
        
        /// Clear calculation history
        void clearHistory();
        
        /// Get number of available operations
        size_t getOperationCount() const;
        
        /// Template method for type-safe operations
        template<typename T>
        T processValue(T value) {
            static_assert(std::is_arithmetic_v<T>, "T must be arithmetic type");
            return static_cast<T>(value * 1.0);
        }
    };

    /// Utility functions
    namespace utils {
        double roundToPrecision(double value, int precision);
        bool isValidNumber(const std::string& str);
        std::vector<std::string> getAvailableOperations();
    }
    
    /// Constants
    constexpr double PI = 3.14159265359;
    constexpr double E = 2.71828182846;
    
    /// Type aliases
    using OperationPtr = std::unique_ptr<Operation>;
    using HistoryContainer = std::vector<double>;

} // namespace math

#endif // CALCULATOR_H