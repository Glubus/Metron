use crate::calculator::{Calculator, CalculatorError, CalculatorResult, Rating};
use crate::clock_rate::ClockRate;
use crate::etterna::convert::chart_to_notes;
use crate::etterna::rows::Rows;
use minacalc_rs::{Calc, CalcMode, SkillsetScores};
use rox::RoxChart;
use std::cell::RefCell;

// One Calc per thread — Calc is not Send (underlying C++ is not thread-safe).
thread_local! {
    static CALC: RefCell<Calc> = RefCell::new(
        Calc::new().expect("Failed to allocate MinaCalc instance")
    );
}

fn with_calc<F, T>(f: F) -> Result<T, CalculatorError>
where
    F: FnOnce(&Calc) -> Result<T, minacalc_rs::Error>,
{
    CALC.with(|c| {
        let result = f(&c.borrow());
        // The C++ threw (and was caught on its side): its state is unknown, so this thread starts again from a fresh calculator.
        if matches!(result, Err(minacalc_rs::Error::CalcThrew))
            && let Ok(fresh) = Calc::new()
        {
            *c.borrow_mut() = fresh;
        }
        result.map_err(|e| CalculatorError::Calculation(e.to_string()))
    })
}

#[derive(Debug, Clone, Copy)]
pub struct MinaCalcDifficultyContext {
    pub clock_rate: ClockRate,
    /// `CalcMode::Msd` = normal/uncapped (default, used on etternaonline.com).
    /// `CalcMode::Ssr` = nerfed/capped — some integrations use this silently.
    pub mode: CalcMode,
    /// Note-time model; see [`Rows`]. `Rows::Quantized` (default) snaps charts that are not
    /// already on Etterna's rows; `Rows::Exact` keeps their times (pre-0.4 behaviour).
    pub rows: Rows,
    /// How a rate that is not computed directly is rated; see [`RateMode`]. `RateMode::Exact` (default) runs MinaCalc at the requested rate.
    pub rate_mode: RateMode,
}

impl Default for MinaCalcDifficultyContext {
    fn default() -> Self {
        Self {
            clock_rate: ClockRate::default(),
            mode: CalcMode::Msd,
            rows: Rows::default(),
            rate_mode: RateMode::default(),
        }
    }
}

/// How [`MinaCalc527`] rates a chart at a music rate.
///
/// Both modes are MinaCalc 527 output; they differ in whether MinaCalc runs at the requested rate or reads Etterna's
/// cached grid of rates.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RateMode {
    /// MinaCalc runs at the requested rate itself.
    ///
    /// Why: it is the real value at every rate, and the computation Etterna itself uses for a score's SSR (a play at
    /// 1.35x is rated at 1.35x, never interpolated). The default.
    #[default]
    Exact,
    /// Etterna's song-select value (`Steps::GetMSD`): MinaCalc runs only at the cached rates 0.7, 0.8, …, 2.0.
    ///
    /// A cached rate returns its value. A rate between two cached rates is the linear interpolation of its neighbours.
    /// Above 2.0 the 2.0x value is extrapolated, `d2 + d2 * (rate - 2) * 0.5`; below 0.7 the 0.7x value is
    /// extrapolated and clamped at zero, `max(d07 - d07 * (0.7 - rate), 0)`.
    ///
    /// Why: it reproduces the MSD Etterna SHOWS for a chart at a rate that is not a multiple of 0.1 (1.05, 1.35…),
    /// which can differ from the real value by several tenths (0.46 MSD on a reference chart at 1.35x). It also lets
    /// a caller keep [`MinaCalc527::rates_table`] (14 values per chart) and answer any rate without the chart or the
    /// calculator. On a cached rate both modes give the same number.
    Interpolated,
}

#[derive(Debug, Clone, Copy)]
pub struct EtternaDifficulty {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

impl Rating for EtternaDifficulty {}

impl From<SkillsetScores> for EtternaDifficulty {
    fn from(scores: SkillsetScores) -> Self {
        Self {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        }
    }
}

impl EtternaDifficulty {
    /// Applies `f` to overall and to every skillset.
    fn map_fields(self, f: impl Fn(f32) -> f32) -> Self {
        Self {
            overall: f(self.overall),
            stream: f(self.stream),
            jumpstream: f(self.jumpstream),
            handstream: f(self.handstream),
            stamina: f(self.stamina),
            jackspeed: f(self.jackspeed),
            chordjack: f(self.chordjack),
            technical: f(self.technical),
        }
    }

    /// Applies `f` to each field of `self` and `other`.
    fn zip_fields(self, other: Self, f: impl Fn(f32, f32) -> f32) -> Self {
        Self {
            overall: f(self.overall, other.overall),
            stream: f(self.stream, other.stream),
            jumpstream: f(self.jumpstream, other.jumpstream),
            handstream: f(self.handstream, other.handstream),
            stamina: f(self.stamina, other.stamina),
            jackspeed: f(self.jackspeed, other.jackspeed),
            chordjack: f(self.chordjack, other.chordjack),
            technical: f(self.technical, other.technical),
        }
    }
}

/// Context for performance calculation.
///
/// Defaults to `CalcMode::Ssr` (nerfed, capped at player accuracy) since that
/// is the typical use-case for a score-relative rating.
/// Switch to `CalcMode::Msd` if you want the uncapped version at a given accuracy.
#[derive(Debug, Clone, Copy)]
pub struct MinaCalcPerformanceContext {
    pub clock_rate: ClockRate,
    /// Player's accuracy / score goal (e.g. 0.93 for 93%).
    pub score_goal: f32,
    /// `CalcMode::Ssr` = nerfed/capped (default). `CalcMode::Msd` = uncapped.
    pub mode: CalcMode,
    /// Note-time model; see [`Rows`]. Same meaning as in [`MinaCalcDifficultyContext`].
    pub rows: Rows,
}

impl Default for MinaCalcPerformanceContext {
    fn default() -> Self {
        Self {
            clock_rate: ClockRate::default(),
            score_goal: 0.93,
            mode: CalcMode::Ssr,
            rows: Rows::default(),
        }
    }
}

pub struct MinaCalc527;

impl Calculator for MinaCalc527 {
    type DifficultyContext = MinaCalcDifficultyContext;
    type PerformanceContext = MinaCalcPerformanceContext;

    type Difficulty = EtternaDifficulty;
    type Performance = EtternaDifficulty;

    const NAME: &'static str = "MinaCalc";
    const VERSION: &'static str = "527";
    const GAME: &'static str = "etterna";
    const YEAR: u32 = 2026;

    /// Raw chart difficulty (MSD — normal mode, as shown on etternaonline.com).
    fn calculate_difficulty(
        &self,
        chart: &RoxChart,
        context: &Self::DifficultyContext,
    ) -> CalculatorResult<Self::Difficulty> {
        let key_count = supported_key_count(chart)?;
        let clock_rate: f64 = context.clock_rate.into();
        let rate = clock_rate as f32;
        let notes = chart_to_notes(chart, context.rows)?;

        match context.rate_mode {
            RateMode::Exact => {
                with_calc(|calc| calc.calc_at_rate(&notes, rate, 0.93, key_count, context.mode))
                    .map(EtternaDifficulty::from)
            }
            RateMode::Interpolated => interpolated_at_rate(rate, |grid| {
                let scores = with_calc(|calc| {
                    calc.calc_rates(&notes, &[grid_rate(grid)], key_count, context.mode)
                })?;
                Ok(EtternaDifficulty::from(scores[0]))
            }),
        }
    }

    /// Score-relative rating (SSR — nerfed, capped at player accuracy).
    fn calculate_performance(
        &self,
        chart: &RoxChart,
        _difficulty: &Self::Difficulty,
        context: &Self::PerformanceContext,
    ) -> CalculatorResult<Self::Performance> {
        let key_count = supported_key_count(chart)?;
        let clock_rate: f64 = context.clock_rate.into();
        let notes = chart_to_notes(chart, context.rows)?;

        with_calc(|calc| {
            calc.calc_at_rate(
                &notes,
                clock_rate as f32,
                context.score_goal,
                key_count,
                context.mode,
            )
        })
        .map(EtternaDifficulty::from)
    }
}

impl MinaCalc527 {
    /// Etterna's cached rate grid: MinaCalc at 0.7x, 0.8x, …, 2.0x, where entry `k` is `(k + 7) / 10`x. `Steps::GetMSD` reads this table.
    ///
    /// All entries come from one `calc_all_rates` call. Use `CalcMode::Msd` for Etterna's song-select values.
    pub fn rates_table(
        chart: &RoxChart,
        rows: Rows,
        mode: CalcMode,
    ) -> CalculatorResult<[EtternaDifficulty; RATE_GRID_LEN]> {
        let key_count = supported_key_count(chart)?;
        let notes = chart_to_notes(chart, rows)?;
        let all = with_calc(|calc| calc.calc_all_rates(&notes, key_count, mode))?;
        Ok(all.rates.map(EtternaDifficulty::from))
    }
}

/// The key counts MinaCalc accepts; anything else is refused before the C++ runs.
fn supported_key_count(chart: &RoxChart) -> CalculatorResult<u32> {
    let key_count = u32::from(chart.key_count);
    if key_count != 4 && key_count != 6 && key_count != 7 {
        return Err(CalculatorError::Calculation(format!(
            "Unsupported key count: {key_count}"
        )));
    }
    Ok(key_count)
}

/// Number of cached rates: 0.7x through 2.0x in steps of 0.1.
const RATE_GRID_LEN: usize = 14;

/// Rate of cached grid entry `index`, computed as Etterna's `MinaSDCalc` does (`static_cast<float>(i) / 10.F`).
fn grid_rate(index: usize) -> f32 {
    (index + 7) as f32 / 10.0
}

/// Etterna's `Steps::GetMSD` (`Models/StepsAndStyles/Steps.cpp`) with the cached grid supplied by `at`.
///
/// `at(index)` is the cached entry at grid `index`; only the entries the rate needs are asked for.
/// The float operations keep Etterna's order.
fn interpolated_at_rate(
    rate: f32,
    at: impl Fn(usize) -> CalculatorResult<EtternaDifficulty>,
) -> CalculatorResult<EtternaDifficulty> {
    // Above 2x: extrapolate from the 2x entry.
    if rate > 2.0 {
        let top = at(RATE_GRID_LEN - 1)?;
        return Ok(top.map_fields(|p| p + p * ((rate - 2.0) * 0.5)));
    }
    // Below 0.7x: extrapolate from the 0.7x entry, steeper, never below zero.
    if rate < 0.7 {
        let bottom = at(0)?;
        return Ok(bottom.map_fields(|p| (p - p * (0.7 - rate)).max(0.0)));
    }

    // A cached rate returns its entry; any other rate interpolates its two neighbours.
    let scaled = rate * 10.0;
    let index = (scaled as i32 - 7) as usize;
    let prop = scaled % 1.0;
    let low = at(index)?;
    if prop == 0.0 {
        return Ok(low);
    }
    let high = at(index + 1)?;
    // Etterna's `lerp(t, a, b)` = `(1 - t) * a + t * b`.
    Ok(low.zip_fields(high, |a, b| (1.0 - prop) * a + prop * b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etterna::convert::chart_to_notes;
    use crate::etterna::rows::Rows;
    use minacalc_rs::Calc;
    use rox::Note;
    use rox::formats::auto_decode;

    fn test_chart() -> RoxChart {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/test.osu");
        auto_decode(path).expect("Failed to decode test.osu")
    }

    #[test]
    fn test_minacalc527_difficulty() {
        let chart = test_chart();
        let calc = MinaCalc527;
        let context = MinaCalcDifficultyContext::default();

        let result = calc.calculate_difficulty(&chart, &context);
        assert!(result.is_ok());
        let diff = result.unwrap();
        println!("MinaCalc527 overall: {}", diff.overall);
        assert!(diff.overall > 0.0);
    }

    #[test]
    fn rated_wrapper_matches_minacalc_direct_at_non_1x_rate() {
        let chart = test_chart();
        let notes = chart_to_notes(&chart, Rows::Quantized).expect("convert chart");
        let direct = Calc::new()
            .expect("create calc")
            .calc_at_rate(&notes, 1.5, 0.93, u32::from(chart.key_count), CalcMode::Msd)
            .expect("direct minacalc");

        let wrapped = MinaCalc527
            .calculate_difficulty(
                &chart,
                &MinaCalcDifficultyContext {
                    clock_rate: ClockRate::from_percentage(150).unwrap(),
                    mode: CalcMode::Msd,
                    rows: Rows::Quantized,
                    rate_mode: RateMode::Exact,
                },
            )
            .expect("wrapped minacalc");

        let delta = (wrapped.overall - direct.overall).abs();
        assert!(
            delta < 0.0001,
            "wrapped={} direct={}",
            wrapped.overall,
            direct.overall
        );
    }

    #[test]
    fn mines_do_not_contribute_to_minacalc_rows() {
        let mut chart = RoxChart::new(4);
        chart.notes.push(Note::tap(0, 0));
        chart.notes.push(Note::mine(0, 1));
        chart.notes.push(Note::tap(100_000, 2));

        let notes = chart_to_notes(&chart, Rows::Exact).expect("convert chart");
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].notes, 0b0001);
        assert_eq!(notes[1].notes, 0b0100);
    }

    /// Etterna times rows from the first one, so moving the whole chart (the song's offset) never changes its rating.
    #[test]
    fn the_rating_does_not_depend_on_where_the_chart_starts() {
        let chart = test_chart();
        let rate = |offset_us: i64| {
            let mut moved = chart.clone();
            for note in &mut moved.notes {
                note.time_us += offset_us;
            }
            MinaCalc527
                .calculate_difficulty(&moved, &MinaCalcDifficultyContext::default())
                .expect("rate the moved chart")
                .overall
        };
        let at_start = rate(0);
        assert_eq!(at_start, rate(2_731_000));
        assert_eq!(at_start, rate(-935_000));
    }

    fn uniform(value: f32) -> EtternaDifficulty {
        EtternaDifficulty {
            overall: value,
            stream: value,
            jumpstream: value,
            handstream: value,
            stamina: value,
            jackspeed: value,
            chordjack: value,
            technical: value,
        }
    }

    fn assert_fields_close(actual: EtternaDifficulty, expected: EtternaDifficulty, tolerance: f32) {
        let pairs = [
            ("overall", actual.overall, expected.overall),
            ("stream", actual.stream, expected.stream),
            ("jumpstream", actual.jumpstream, expected.jumpstream),
            ("handstream", actual.handstream, expected.handstream),
            ("stamina", actual.stamina, expected.stamina),
            ("jackspeed", actual.jackspeed, expected.jackspeed),
            ("chordjack", actual.chordjack, expected.chordjack),
            ("technical", actual.technical, expected.technical),
        ];
        for (name, a, e) in pairs {
            assert!(
                (a - e).abs() <= tolerance,
                "{name}: actual={a} expected={e}"
            );
        }
    }

    fn difficulty_at(chart: &RoxChart, percent: u32, rate_mode: RateMode) -> EtternaDifficulty {
        MinaCalc527
            .calculate_difficulty(
                chart,
                &MinaCalcDifficultyContext {
                    clock_rate: ClockRate::from_percentage(percent).unwrap(),
                    mode: CalcMode::Msd,
                    rows: Rows::Quantized,
                    rate_mode,
                },
            )
            .expect("rate the chart")
    }

    #[test]
    fn rate_mode_defaults_to_exact() {
        assert_eq!(
            MinaCalcDifficultyContext::default().rate_mode,
            RateMode::Exact
        );
    }

    #[test]
    fn interpolated_equals_exact_on_grid_rates() {
        let chart = test_chart();
        for percent in [100, 130, 200] {
            assert_fields_close(
                difficulty_at(&chart, percent, RateMode::Interpolated),
                difficulty_at(&chart, percent, RateMode::Exact),
                1e-4,
            );
        }
    }

    #[test]
    fn interpolated_at_1_35_is_the_mean_of_1_3_and_1_4() {
        let chart = test_chart();
        let low = difficulty_at(&chart, 130, RateMode::Exact);
        let high = difficulty_at(&chart, 140, RateMode::Exact);
        assert_fields_close(
            difficulty_at(&chart, 135, RateMode::Interpolated),
            low.zip_fields(high, |a, b| (a + b) / 2.0),
            1e-4,
        );
    }

    #[test]
    fn interpolated_above_two_extrapolates_from_two() {
        let chart = test_chart();
        let top = difficulty_at(&chart, 200, RateMode::Exact);
        assert_fields_close(
            difficulty_at(&chart, 220, RateMode::Interpolated),
            top.map_fields(|p| p + p * ((2.2_f32 - 2.0) * 0.5)),
            1e-4,
        );
    }

    #[test]
    fn interpolated_below_point_seven_extrapolates_from_point_seven() {
        let chart = test_chart();
        let bottom = difficulty_at(&chart, 70, RateMode::Exact);
        assert_fields_close(
            difficulty_at(&chart, 60, RateMode::Interpolated),
            bottom.map_fields(|p| (p - p * (0.7_f32 - 0.6_f32)).max(0.0)),
            1e-4,
        );
    }

    #[test]
    fn interpolation_requests_only_the_neighbouring_grid_entries() {
        let only = |allowed: &'static [usize]| {
            move |k: usize| {
                if allowed.contains(&k) {
                    Ok(uniform(10.0 + k as f32))
                } else {
                    Err(CalculatorError::Calculation(format!(
                        "entry {k} not needed"
                    )))
                }
            }
        };
        let grid = interpolated_at_rate(1.0, only(&[3])).expect("grid rate needs one entry");
        assert_eq!(grid.overall, 13.0);
        let between = interpolated_at_rate(1.35, only(&[6, 7])).expect("between needs two");
        assert!((between.overall - 16.5).abs() < 1e-3);
        let above = interpolated_at_rate(2.2, only(&[13])).expect("above 2x needs 2x");
        assert!((above.overall - 25.3).abs() < 1e-3);
        let below = interpolated_at_rate(0.6, only(&[0])).expect("below 0.7x needs 0.7x");
        assert!((below.overall - 9.0).abs() < 1e-3);
    }

    #[test]
    fn rates_table_matches_exact_and_drives_interpolation() {
        let chart = test_chart();
        let table =
            MinaCalc527::rates_table(&chart, Rows::Quantized, CalcMode::Msd).expect("rates table");
        for (index, entry) in table.iter().enumerate() {
            let percent = (index as u32 + 7) * 10;
            assert_fields_close(
                *entry,
                difficulty_at(&chart, percent, RateMode::Exact),
                1e-4,
            );
        }
        for percent in [60, 100, 135, 220] {
            let rate: f64 = ClockRate::from_percentage(percent).unwrap().into();
            let from_table = interpolated_at_rate(rate as f32, |k| Ok(table[k])).expect("lookup");
            assert_fields_close(
                difficulty_at(&chart, percent, RateMode::Interpolated),
                from_table,
                1e-4,
            );
        }
    }
}
