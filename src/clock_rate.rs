/// Represents the game clock rate (playback speed).
///
/// # Why a dedicated type?
///
/// - **Encapsulation**: Hides internal representation (currently a percentage as u32)
/// - **Evolvability**: Allows changing precision (to f64, precise ratio, etc.) without impacting consumers
/// - **Robustness**: Validation at construction time
/// - **Clarity**: Explicit conversion API
///
/// # Current format
///
/// Stored as a percentage (100 = normal speed, 150 = 1.5x, 80 = 0.8x)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockRate {
    /// Percentage (100 = normal speed)
    percentage: u32,
}

impl ClockRate {
    /// Default value: normal speed (100%)
    pub const DEFAULT: Self = Self { percentage: 100 };

    /// Valid range for clock_rate`
    const MIN_PERCENTAGE: u32 = 1; // 0.01x
    const MAX_PERCENTAGE: u32 = 1000; // 10.0x

    /// Creates a `ClockRate` from a percentage.
    ///
    /// # Arguments
    ///
    /// * `percentage` - Clock rate percentage (100 = normal speed)
    ///
    /// # Errors
    ///
    /// Returns an error if the percentage is outside valid limits
    pub fn from_percentage(percentage: u32) -> Result<Self, ClockRateError> {
        if !(Self::MIN_PERCENTAGE..=Self::MAX_PERCENTAGE).contains(&percentage) {
            return Err(ClockRateError::OutOfRange {
                value: percentage,
                min: Self::MIN_PERCENTAGE,
                max: Self::MAX_PERCENTAGE,
            });
        }
        Ok(Self { percentage })
    }

    /// Returns the raw percentage
    #[must_use]
    pub fn as_percentage(&self) -> u32 {
        self.percentage
    }
}

impl Default for ClockRate {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Converts `ClockRate` to `f64` multiplier.
///
/// # Why f64?
///
/// f64 is used in difficulty calculations for higher precision.
/// This conversion allows seamless integration with existing calculation code.
///
/// # Examples
///
/// ```
/// # use metron::clock_rate::ClockRate;
/// let rate = ClockRate::from_percentage(150).unwrap();
/// let multiplier: f64 = rate.into();
/// assert_eq!(multiplier, 1.5);
/// ```
impl From<ClockRate> for f64 {
    fn from(clock_rate: ClockRate) -> Self {
        f64::from(clock_rate.percentage) / 100.0
    }
}

/// Errors related to `ClockRate`
#[derive(Debug, thiserror::Error)]
pub enum ClockRateError {
    #[error("Clock rate {value}% out of range (min: {min}%, max: {max}%)")]
    OutOfRange { value: u32, min: u32, max: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_default_clock_rate() {
        let rate = ClockRate::default();
        assert_eq!(rate.as_percentage(), 100);
        assert_relative_eq!(f64::from(rate), 1.0);
    }

    #[test]
    fn test_from_percentage_valid() {
        let rate = ClockRate::from_percentage(150).expect("Valid percentage");
        assert_eq!(rate.as_percentage(), 150);
        assert_relative_eq!(f64::from(rate), 1.5);
    }

    #[test]
    fn test_from_percentage_invalid_too_low() {
        let result = ClockRate::from_percentage(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_percentage_invalid_too_high() {
        let result = ClockRate::from_percentage(1100);
        assert!(result.is_err());
    }

    #[test]
    fn test_edge_cases() {
        // Min valide
        let min_rate = ClockRate::from_percentage(1).expect("Min valid");
        assert_relative_eq!(f64::from(min_rate), 0.01);

        // Max valide
        let max_rate = ClockRate::from_percentage(1000).expect("Max valid");
        assert_relative_eq!(f64::from(max_rate), 10.0);
    }
}
