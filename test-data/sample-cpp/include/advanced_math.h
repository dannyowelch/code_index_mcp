#pragma once

#include <array>
#include <functional>
#include <type_traits>
#include <concepts>
#include <memory>

namespace math::advanced {

    // Modern C++ concepts
    template<typename T>
    concept Arithmetic = std::is_arithmetic_v<T>;

    template<typename T>
    concept Numeric = requires(T a, T b) {
        { a + b } -> std::convertible_to<T>;
        { a - b } -> std::convertible_to<T>;
        { a * b } -> std::convertible_to<T>;
        { a / b } -> std::convertible_to<T>;
    };

    // Template metaprogramming
    template<std::size_t N>
    struct Fibonacci {
        static constexpr std::size_t value = Fibonacci<N-1>::value + Fibonacci<N-2>::value;
    };

    template<>
    struct Fibonacci<0> {
        static constexpr std::size_t value = 0;
    };

    template<>
    struct Fibonacci<1> {
        static constexpr std::size_t value = 1;
    };

    // SFINAE example
    template<typename T, typename = void>
    struct has_size : std::false_type {};

    template<typename T>
    struct has_size<T, std::void_t<decltype(std::declval<T>().size())>> : std::true_type {};

    template<typename T>
    constexpr bool has_size_v = has_size<T>::value;

    // Variadic templates
    template<Arithmetic T>
    constexpr T sum(T value) {
        return value;
    }

    template<Arithmetic T, Arithmetic... Args>
    constexpr T sum(T first, Args... args) {
        return first + sum(args...);
    }

    // Modern C++ matrix class with RAII
    template<Numeric T, std::size_t Rows, std::size_t Cols>
    class Matrix {
    private:
        std::array<std::array<T, Cols>, Rows> data_{};

    public:
        using value_type = T;
        using size_type = std::size_t;
        using reference = T&;
        using const_reference = const T&;

        constexpr Matrix() = default;
        constexpr explicit Matrix(T initial_value) {
            for (auto& row : data_) {
                row.fill(initial_value);
            }
        }

        // Initialize with nested initializer list
        constexpr Matrix(std::initializer_list<std::initializer_list<T>> init) {
            size_type i = 0;
            for (const auto& row_init : init) {
                if (i >= Rows) break;
                size_type j = 0;
                for (const auto& value : row_init) {
                    if (j >= Cols) break;
                    data_[i][j] = value;
                    ++j;
                }
                ++i;
            }
        }

        constexpr reference operator()(size_type row, size_type col) {
            return data_[row][col];
        }

        constexpr const_reference operator()(size_type row, size_type col) const {
            return data_[row][col];
        }

        constexpr auto begin() { return data_.begin(); }
        constexpr auto end() { return data_.end(); }
        constexpr auto begin() const { return data_.begin(); }
        constexpr auto end() const { return data_.end(); }

        static constexpr size_type rows() { return Rows; }
        static constexpr size_type cols() { return Cols; }

        // Matrix operations
        template<std::size_t OtherCols>
        constexpr Matrix<T, Rows, OtherCols> operator*(const Matrix<T, Cols, OtherCols>& other) const {
            Matrix<T, Rows, OtherCols> result{};
            for (size_type i = 0; i < Rows; ++i) {
                for (size_type j = 0; j < OtherCols; ++j) {
                    for (size_type k = 0; k < Cols; ++k) {
                        result(i, j) += (*this)(i, k) * other(k, j);
                    }
                }
            }
            return result;
        }

        constexpr Matrix operator+(const Matrix& other) const {
            Matrix result{};
            for (size_type i = 0; i < Rows; ++i) {
                for (size_type j = 0; j < Cols; ++j) {
                    result(i, j) = (*this)(i, j) + other(i, j);
                }
            }
            return result;
        }

        constexpr Matrix operator-(const Matrix& other) const {
            Matrix result{};
            for (size_type i = 0; i < Rows; ++i) {
                for (size_type j = 0; j < Cols; ++j) {
                    result(i, j) = (*this)(i, j) - other(i, j);
                }
            }
            return result;
        }

        constexpr bool operator==(const Matrix& other) const = default;
    };

    // Type aliases for common matrix types
    template<Numeric T>
    using Matrix2x2 = Matrix<T, 2, 2>;

    template<Numeric T>
    using Matrix3x3 = Matrix<T, 3, 3>;

    template<Numeric T>
    using Matrix4x4 = Matrix<T, 4, 4>;

    using Matrix2x2f = Matrix2x2<float>;
    using Matrix3x3f = Matrix3x3<float>;
    using Matrix4x4f = Matrix4x4<float>;
    using Matrix2x2d = Matrix2x2<double>;
    using Matrix3x3d = Matrix3x3<double>;
    using Matrix4x4d = Matrix4x4<double>;

    // Function templates with constraints
    template<Numeric T>
    constexpr T determinant(const Matrix2x2<T>& m) {
        return m(0, 0) * m(1, 1) - m(0, 1) * m(1, 0);
    }

    template<Numeric T>
    constexpr T determinant(const Matrix3x3<T>& m) {
        return m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1))
             - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
             + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0));
    }

    // Lambda and function object support
    template<typename T, typename UnaryFunc>
    constexpr auto transform_matrix(const Matrix<T, 2, 2>& m, UnaryFunc func) 
        -> Matrix<decltype(func(std::declval<T>())), 2, 2> {
        using ResultType = decltype(func(std::declval<T>()));
        Matrix<ResultType, 2, 2> result{};
        
        for (std::size_t i = 0; i < 2; ++i) {
            for (std::size_t j = 0; j < 2; ++j) {
                result(i, j) = func(m(i, j));
            }
        }
        return result;
    }

    // Constants with inline variables
    inline constexpr double GOLDEN_RATIO = 1.618033988749;
    inline constexpr double SQRT_2 = 1.414213562373;
    inline constexpr double SQRT_3 = 1.732050807569;

    // Constexpr functions
    constexpr std::size_t factorial(std::size_t n) {
        return n == 0 ? 1 : n * factorial(n - 1);
    }

    constexpr double power(double base, std::size_t exponent) {
        return exponent == 0 ? 1.0 : base * power(base, exponent - 1);
    }

    // Modern C++20 features
    #ifdef __cpp_concepts
    template<std::integral T>
    constexpr bool is_even(T value) {
        return value % 2 == 0;
    }

    template<std::floating_point T>
    constexpr bool approximately_equal(T a, T b, T epsilon = static_cast<T>(1e-9)) {
        return (a - b < epsilon) && (b - a < epsilon);
    }
    #endif

    // Custom deleter for smart pointers example
    struct MatrixDeleter {
        template<typename T>
        void operator()(T* ptr) const {
            delete[] ptr;
        }
    };

    template<typename T>
    using unique_array = std::unique_ptr<T[], MatrixDeleter>;

} // namespace math::advanced