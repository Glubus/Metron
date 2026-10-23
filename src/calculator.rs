use rox::RoxChart;
use thiserror::Error;

/// Domain-specific errors for calculator operations.
#[derive(Error, Debug)]
pub enum CalculatorError {
    /// Generic calculation failure with a message.
    #[error("Calculation failed: {0}")]
    Calculation(String),
}

/// Alias for a `Result` with `CalculatorError`.
pub type CalculatorResult<T> = Result<T, CalculatorError>;

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
    type DifficultyContext;
    type PerformanceContext;

    type Difficulty: Rating;
    type Performance: Rating;

    /// The name of the calculator (e.g., "`MinaCalc`").
    const NAME: &'static str;
    /// The version of the calculator (e.g., "`5.15`").
    const VERSION: &'static str;
    /// The game this calculator is designed for.
    const GAME: &'static str;
    /// The year this calculator was released.
    const YEAR: u32;

    /// Returns the human-readable name of the calculator (e.g., "`MinaCalc`").
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

    /// Returns the year this calculator was released.
    fn year(&self) -> &u32 {
        &Self::YEAR
    }

    /// Calculates the difficulty for a given chart.
    ///
    /// # Errors
    /// Returns a `CalculatorError` if calculation fails (e.g. invalid chart data).
    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty>;

    /// Calculates the performance for a given difficulty and accuracy.
    ///
    /// # Errors
    /// Returns a `CalculatorError` if calculation fails or is not implemented.
    fn calculate_performance(
        &self,
        _chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        _context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        Err(CalculatorError::Calculation("not implemented".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock Rating struct
    #[derive(Debug)]
    struct MockRating {
        #[allow(dead_code)]
        stars: f32,
    }

    impl Rating for MockRating {}

    struct MockCalculator;

    impl Calculator for MockCalculator {
        type DifficultyContext = ();
        type PerformanceContext = ();

        type Difficulty = MockRating;
        type Performance = MockRating;

        const NAME: &'static str = "MockCalc";
        const VERSION: &'static str = "1.0.0";
        const GAME: &'static str = "GenericVSRG";
        const YEAR: u32 = 2026;

        fn calculate_difficulty(
            &self,
            _chart: &RoxChart,
            _context: &Self::DifficultyContext,
        ) -> CalculatorResult<Self::Difficulty> {
            Ok(MockRating { stars: 5.0 })
        }

        fn calculate_performance(
            &self,
            _chart: &RoxChart,
            _difficulty: &Self::Difficulty,
            _context: &Self::PerformanceContext,
        ) -> CalculatorResult<Self::Performance> {
            Ok(MockRating { stars: 100.0 })
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
}
