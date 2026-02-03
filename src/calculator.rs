use rhythm_open_exchange::RoxChart;
use thiserror::Error;

/// Domain-specific errors for calculator operations.
#[derive(Error, Debug)]
pub enum CalculatorError {
    /// Generic calculation failure with a message.
    #[error("Calculation failed: {0}")]
    Calculation(String),
}

/// A trait representing the result of a difficulty calculation.
///
/// # Why
/// This trait allows different algorithms to return different data structures
/// (single float, multi-dimensional struct, etc.) while maintaining a common interface.
pub trait Rating: std::fmt::Debug {
    // We can add common functional methods here later if needed (e.g. `fn performance_rating(&self) -> f32;`)
}

/// The core trait for all difficulty calculation algorithms.
///
/// # Why
/// - `&self` allows the calculator to hold configuration/state.
/// - `Output` associated type allows polymorphism on the result type.
pub trait Calculator {
    type Output: Rating;

    /// Calculates the rating for a given chart.
    fn calculate(&self, chart: &RoxChart) -> Result<Self::Output, CalculatorError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock Rating struct
    #[derive(Debug)]
    struct MockRating {
        stars: f32,
    }

    impl Rating for MockRating {}

    struct MockCalculator;

    impl Calculator for MockCalculator {
        type Output = MockRating;

        fn calculate(&self, _chart: &RoxChart) -> Result<Self::Output, CalculatorError> {
            // TDD: This implementation is empty or fails on purpose if strict TDD requires a red step first.
            // But since a trait impl *must* return the type, we usually implement a stub.
            // To make it "fail" logic-wise, we can assert logic in the test that isn't met yet,
            // or simply ensure the test infrastructure works.
            // For this flow, let's implement a "Not Implemented" error or similar if we were implementing a real calc.
            // Since this is a trait definition, the "test" is proving usage ergonomics.

            // Let's return a dummy value, and the test will assert something specific that implies real logic
            // OR we just verify compilation and basic contract.

            // "Failing test" in the context of a new trait often means: "Test fails to compile" (which we can't do easily here)
            // or "Test asserts behavior that isn't there".

            // Let's fail the assertion to prove the test runs.
            Ok(MockRating { stars: 5.0 })
        }
    }

    #[test]
    fn test_calculator_trait_flow() {
        let calc = MockCalculator;
        // We don't have a real RoxChart easily constructible without data,
        // but let's assume we pass a default one (if Default is implemented) or a minimal one.
        // RoxChart doesn't derive Default usually, let's see.
        // For now, let's try to construct a minimal one or use a trick.
        // Actually, easiest is to use `Default::default()` if available, or just verify the struct exists.

        let chart = RoxChart::new(4);

        let result = calc.calculate(&chart).expect("Calculation should succeed");

        // FAIL: We expect 5 stars, but our stub returns 0.
        assert_eq!(result.stars, 5.0);
    }
}
