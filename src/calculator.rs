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

    /// The name of the calculator.
    const NAME: &str;
    /// The version of the calculator.
    const VERSION: &str;
    /// The game this calculator is designed for.
    const GAME: &str;

    /// Returns the human-readable name of the calculator (e.g., "MinaCalc").
    fn name(&self) -> &str {
        Self::NAME
    }

    /// Returns the version of the calculator algorithm (e.g., "5.15").
    fn version(&self) -> &str {
        Self::VERSION
    }

    /// Returns the game this calculator is designed for (e.g., "osu!mania").
    fn game(&self) -> &str {
        Self::GAME
    }

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

        const NAME: &str = "MockCalc";
        const VERSION: &str = "1.0.0";
        const GAME: &str = "GenericVSRG";

        fn calculate(&self, _chart: &RoxChart) -> Result<Self::Output, CalculatorError> {
            Ok(MockRating { stars: 5.0 })
        }
    }

    #[test]
    fn test_mock_calculator_name() {
        let calc = MockCalculator;
        assert_eq!(calc.name(), "MockCalc");
    }

    #[test]
    fn test_mock_calculator_version() {
        let calc = MockCalculator;
        assert_eq!(calc.version(), "1.0.0");
    }

    #[test]
    fn test_mock_calculator_game() {
        let calc = MockCalculator;
        assert_eq!(calc.game(), "GenericVSRG");
    }

    #[test]
    fn test_mock_calculator_calculate() {
        let calc = MockCalculator;
        let chart = RoxChart::new(4);
        let result = calc.calculate(&chart).expect("Calculation should succeed");
        assert_eq!(result.stars, 5.0);
    }
}
