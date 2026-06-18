use std::cell::RefCell;
use std::cmp::Ordering;
use std::f64::consts::PI;

use rhythm_open_exchange::RoxChart;

use crate::calculator::{CalculatorError, CalculatorResult};

use super::bars::{abar, jbar, pbar, xbar};
use super::calculations::{anchor, ck, corners, key_usage};
use super::interpolation::{interp_values, interp_values_into, search_left, step_interp_into};
use super::map_data::{MAX_SUPPORTED_KEYS, MapData, preprocess_chart};
use super::note::Note;
use super::smoothing::{gaussian_filter1d, rescale_high};
use super::{DanielDifficulty, DanielDifficultyContext, DanielDifficultyGraph, DanielFactorCurves};

const BREAK_ZERO_THRESHOLD_MS: f64 = 400.0;
const GRAPH_RESAMPLE_INTERVAL_MS: f64 = 100.0;
const SMOOTH_SIGMA_MS: f64 = 800.0;

struct CurveOutputs {
    jbar: Vec<f64>,
    xbar: Vec<f64>,
    pbar: Vec<f64>,
    abar: Vec<f64>,
    c_arr: Vec<f64>,
    ks_arr: Vec<f64>,
}

impl CurveOutputs {
    const fn new() -> Self {
        Self {
            jbar: Vec::new(),
            xbar: Vec::new(),
            pbar: Vec::new(),
            abar: Vec::new(),
            c_arr: Vec::new(),
            ks_arr: Vec::new(),
        }
    }
}

struct RatingScratch {
    effective_weights: Vec<f64>,
    sorted_indices: Vec<usize>,
    cumulative_weights: Vec<f64>,
    note_times: Vec<f64>,
    proximity_curve: Vec<f64>,
}

impl RatingScratch {
    const fn new() -> Self {
        Self {
            effective_weights: Vec::new(),
            sorted_indices: Vec::new(),
            cumulative_weights: Vec::new(),
            note_times: Vec::new(),
            proximity_curve: Vec::new(),
        }
    }
}

struct Phase1Data {
    active_masks: Vec<u16>,
    anchor: Vec<f64>,
}

thread_local! {
    static CURVES: RefCell<CurveOutputs> = const { RefCell::new(CurveOutputs::new()) };
    static SCRATCH: RefCell<RatingScratch> = const { RefCell::new(RatingScratch::new()) };
}

pub fn calculate(
    chart: &RoxChart,
    context: &DanielDifficultyContext,
) -> CalculatorResult<DanielDifficulty> {
    validate_chart(chart)?;

    let clock_rate = f64::from(context.clock_rate.unwrap_or_default());
    let overall_difficulty = f64::from(context.overall_difficulty.unwrap_or(8.0));
    let map = preprocess_chart(chart, clock_rate, overall_difficulty);

    if map.notes.is_empty() {
        return Ok(empty_difficulty());
    }

    let (all_corners, base_corners, a_corners) =
        corners::get_corners(map.total_duration, &map.notes);
    let phase1 = build_phase1_data(&map, &base_corners);

    CURVES.with(|curve_cell| {
        let mut curves = curve_cell.borrow_mut();
        fill_curve_outputs(
            &map,
            &all_corners,
            &base_corners,
            &a_corners,
            &phase1,
            &mut curves,
        );

        let difficulty_curve = compute_difficulty_curve(
            &curves.jbar,
            &curves.xbar,
            &curves.pbar,
            &curves.abar,
            &curves.ks_arr,
        );

        SCRATCH.with(|scratch_cell| {
            let mut scratch = scratch_cell.borrow_mut();
            let stars = compute_star_rating(
                &difficulty_curve,
                &curves.c_arr,
                &all_corners,
                map.notes.len(),
                &mut scratch,
            );
            let graph_values =
                build_graph_curve(&all_corners, &difficulty_curve, &map.notes, &mut scratch);
            Ok(DanielDifficulty {
                stars,
                graph: DanielDifficultyGraph {
                    times_ms: all_corners.clone(),
                    values: graph_values,
                },
                factors: DanielFactorCurves {
                    pressing_intensity: curves.pbar.clone(),
                    unevenness: curves.abar.clone(),
                    same_column_pressure: curves.jbar.clone(),
                    cross_column_pressure: curves.xbar.clone(),
                },
            })
        })
    })
}

fn validate_chart(chart: &RoxChart) -> CalculatorResult<()> {
    let key_count = usize::from(chart.key_count);
    if !(1..=MAX_SUPPORTED_KEYS).contains(&key_count) {
        return Err(CalculatorError::Calculation(format!(
            "Daniel supports 1K to {MAX_SUPPORTED_KEYS}K charts, got {}K",
            chart.key_count
        )));
    }
    Ok(())
}

fn empty_difficulty() -> DanielDifficulty {
    DanielDifficulty {
        stars: 0.0,
        graph: DanielDifficultyGraph {
            times_ms: vec![0.0],
            values: vec![0.0],
        },
        factors: DanielFactorCurves {
            pressing_intensity: vec![0.0],
            unevenness: vec![1.0],
            same_column_pressure: vec![0.0],
            cross_column_pressure: vec![0.0],
        },
    }
}

fn build_phase1_data(map: &MapData, base_corners: &[f64]) -> Phase1Data {
    let corner_count = base_corners.len();
    let active_masks = key_usage::with_key_usage(
        map.column_count,
        map.total_duration,
        &map.notes,
        base_corners,
        |usage| key_usage::compute_active_masks(map.column_count, usage, corner_count),
    );
    let anchor =
        key_usage::with_key_usage_400(map.column_count, &map.notes, base_corners, |usage| {
            anchor::compute_anchor(map.column_count, usage, corner_count)
        });
    Phase1Data {
        active_masks,
        anchor,
    }
}

fn fill_curve_outputs(
    map: &MapData,
    all_corners: &[f64],
    base_corners: &[f64],
    a_corners: &[f64],
    phase1: &Phase1Data,
    curves: &mut CurveOutputs,
) {
    let (delta_ks, jbar_base) = jbar::compute_jbar(
        map.column_count,
        map.hit_leniency,
        &map.notes_by_column,
        base_corners,
    );
    interp_values_into(all_corners, base_corners, &jbar_base, &mut curves.jbar);

    let xbar_base = xbar::compute_xbar(
        map.column_count,
        map.hit_leniency,
        &map.notes_by_column,
        &phase1.active_masks,
        base_corners,
    );
    interp_values_into(all_corners, base_corners, &xbar_base, &mut curves.xbar);

    let pbar_base = pbar::compute_pbar(map.hit_leniency, &map.notes, &phase1.anchor, base_corners);
    interp_values_into(all_corners, base_corners, &pbar_base, &mut curves.pbar);

    let abar_base = abar::compute_abar(
        map.column_count,
        &phase1.active_masks,
        &delta_ks,
        a_corners,
        base_corners,
    );
    interp_values_into(all_corners, a_corners, &abar_base, &mut curves.abar);

    let (c_step, ks_step) = ck::compute_c_and_ks(&map.notes, &phase1.active_masks, base_corners);
    step_interp_into(all_corners, base_corners, &c_step, &mut curves.c_arr);
    step_interp_into(all_corners, base_corners, &ks_step, &mut curves.ks_arr);
}

fn compute_difficulty_curve(
    jbar: &[f64],
    xbar: &[f64],
    pbar: &[f64],
    abar: &[f64],
    ks_arr: &[f64],
) -> Vec<f64> {
    let mut difficulty = Vec::with_capacity(jbar.len());
    for ((((&j, &x), &p), &a), &ks) in jbar.iter().zip(xbar).zip(pbar).zip(abar).zip(ks_arr) {
        let s_all = combined_skill(j, p, a, ks);
        let t_all = transition_skill(a, ks, x, s_all);
        difficulty.push(difficulty_value(s_all, t_all));
    }
    difficulty
}

fn combined_skill(jbar: f64, pbar: f64, abar: f64, ks: f64) -> f64 {
    (0.4 * (abar.powf(3.0 / ks) * jbar.min(8.0 + 0.85 * jbar)).powf(1.5)
        + 0.6 * (abar.powf(2.0 / 3.0) * (0.8 * pbar)).powf(1.5))
    .powf(2.0 / 3.0)
}

fn transition_skill(abar: f64, ks: f64, xbar: f64, s_all: f64) -> f64 {
    (abar.powf(3.0 / ks) * xbar) / (xbar + s_all + 1.0)
}

fn difficulty_value(s_all: f64, t_all: f64) -> f64 {
    2.7 * s_all.sqrt() * t_all.powf(1.5) + s_all * 0.27
}

fn compute_star_rating(
    difficulty_curve: &[f64],
    c_arr: &[f64],
    all_corners: &[f64],
    total_notes: usize,
    scratch: &mut RatingScratch,
) -> f64 {
    if difficulty_curve.is_empty() {
        return 0.0;
    }

    fill_effective_weights(c_arr, all_corners, &mut scratch.effective_weights);
    sort_difficulty_indices(difficulty_curve, &mut scratch.sorted_indices);
    let total_weight = fill_cumulative_weights(
        &scratch.effective_weights,
        &scratch.sorted_indices,
        &mut scratch.cumulative_weights,
    );
    if total_weight <= 0.0 {
        return 0.0;
    }

    let percentile_indices = percentile_indices(
        &scratch.cumulative_weights,
        total_weight,
        difficulty_curve.len(),
    );
    let percentile_93 = average_percentile_group(
        difficulty_curve,
        &scratch.sorted_indices,
        &percentile_indices[..4],
    );
    let percentile_83 = average_percentile_group(
        difficulty_curve,
        &scratch.sorted_indices,
        &percentile_indices[4..],
    );
    let weighted_mean = weighted_mean_power5(
        difficulty_curve,
        &scratch.effective_weights,
        &scratch.sorted_indices,
    );

    let mut stars = 0.88 * percentile_93 * 0.25 + 0.94 * percentile_83 * 0.2 + weighted_mean * 0.55;
    let total_notes = total_notes as f64;
    stars *= total_notes / (total_notes + 60.0);
    rescale_high(stars) * 0.975
}

fn fill_effective_weights(c_arr: &[f64], all_corners: &[f64], out: &mut Vec<f64>) {
    out.clear();
    match all_corners.len() {
        0 => {}
        1 => out.push(0.0),
        len => {
            out.reserve(len.saturating_sub(out.capacity()));
            out.push(c_arr[0] * (all_corners[1] - all_corners[0]) / 2.0);
            for i in 1..(len - 1) {
                out.push(c_arr[i] * (all_corners[i + 1] - all_corners[i - 1]) / 2.0);
            }
            out.push(c_arr[len - 1] * (all_corners[len - 1] - all_corners[len - 2]) / 2.0);
        }
    }
}

fn sort_difficulty_indices(difficulty_curve: &[f64], out: &mut Vec<usize>) {
    out.clear();
    out.extend(0..difficulty_curve.len());
    out.sort_unstable_by(|&left, &right| {
        difficulty_curve[left]
            .partial_cmp(&difficulty_curve[right])
            .unwrap_or(Ordering::Equal)
    });
}

fn fill_cumulative_weights(
    effective_weights: &[f64],
    sorted_indices: &[usize],
    out: &mut Vec<f64>,
) -> f64 {
    out.clear();
    let mut total = 0.0;
    for &index in sorted_indices {
        total += effective_weights[index];
        out.push(total);
    }
    total
}

fn percentile_indices(cumulative_weights: &[f64], total_weight: f64, len: usize) -> [usize; 8] {
    let targets = [0.945, 0.935, 0.925, 0.915, 0.845, 0.835, 0.825, 0.815];
    let mut indices = [0usize; 8];
    for (index, target) in targets.iter().enumerate() {
        indices[index] = cumulative_weights
            .partition_point(|&value| value < target * total_weight)
            .min(len - 1);
    }
    indices
}

fn average_percentile_group(
    difficulty_curve: &[f64],
    sorted_indices: &[usize],
    percentiles: &[usize],
) -> f64 {
    percentiles
        .iter()
        .map(|&index| difficulty_curve[sorted_indices[index]])
        .sum::<f64>()
        / percentiles.len() as f64
}

fn weighted_mean_power5(
    difficulty_curve: &[f64],
    effective_weights: &[f64],
    sorted_indices: &[usize],
) -> f64 {
    let (num, den) = sorted_indices
        .iter()
        .fold((0.0, 0.0), |(num, den), &index| {
            let value = difficulty_curve[index];
            let weight = effective_weights[index];
            (num + value.powi(5) * weight, den + weight)
        });
    (num / den).powf(0.2)
}

fn build_graph_curve(
    all_corners: &[f64],
    difficulty_curve: &[f64],
    notes: &[Note],
    scratch: &mut RatingScratch,
) -> Vec<f64> {
    fill_note_times(notes, &mut scratch.note_times);
    fill_proximity_curve(
        all_corners,
        difficulty_curve,
        &scratch.note_times,
        &mut scratch.proximity_curve,
    );
    smooth_d_for_graph(all_corners, &scratch.proximity_curve, &scratch.note_times)
}

fn fill_note_times(notes: &[Note], out: &mut Vec<f64>) {
    out.clear();
    out.extend(notes.iter().map(|note| note.hit_time as f64));
}

fn fill_proximity_curve(
    all_corners: &[f64],
    difficulty_curve: &[f64],
    note_times: &[f64],
    out: &mut Vec<f64>,
) {
    out.clear();
    if note_times.is_empty() {
        out.extend_from_slice(difficulty_curve);
        return;
    }

    for (&corner, &value) in all_corners.iter().zip(difficulty_curve) {
        let distance = closest_note_distance(note_times, corner);
        let envelope = 0.5 * (1.0 + (PI * (distance / 500.0).clamp(0.0, 1.0)).cos());
        out.push(value * envelope);
    }
}

fn closest_note_distance(note_times: &[f64], time: f64) -> f64 {
    let index = search_left(note_times, time);
    let after = index.min(note_times.len() - 1);
    let before = index.saturating_sub(1).min(note_times.len() - 1);
    (note_times[after] - time)
        .abs()
        .min((note_times[before] - time).abs())
}

fn smooth_d_for_graph(
    all_corners: &[f64],
    difficulty_curve: &[f64],
    note_times: &[f64],
) -> Vec<f64> {
    let t_start = all_corners[0];
    let t_end = *all_corners.last().expect("non-empty");
    let uniform_t = arange_inclusive(t_start, t_end, GRAPH_RESAMPLE_INTERVAL_MS);
    let break_mask = compute_break_mask(&uniform_t, note_times);

    let mut uniform_d = interp_values(&uniform_t, all_corners, difficulty_curve);
    zero_breaks(&mut uniform_d, &break_mask);

    let sigma_samples = SMOOTH_SIGMA_MS / GRAPH_RESAMPLE_INTERVAL_MS;
    let mut uniform_result = gaussian_filter1d(&uniform_d, sigma_samples);
    zero_breaks(&mut uniform_result, &break_mask);

    interp_values(all_corners, &uniform_t, &uniform_result)
}

fn compute_break_mask(uniform_t: &[f64], note_times: &[f64]) -> Vec<bool> {
    if note_times.is_empty() {
        return vec![false; uniform_t.len()];
    }

    uniform_t
        .iter()
        .map(|&time| closest_note_distance(note_times, time) > BREAK_ZERO_THRESHOLD_MS)
        .collect()
}

fn zero_breaks(values: &mut [f64], break_mask: &[bool]) {
    for (value, &is_break) in values.iter_mut().zip(break_mask) {
        if is_break {
            *value = 0.0;
        }
    }
}

fn arange_inclusive(start: f64, end: f64, step: f64) -> Vec<f64> {
    let mut values = Vec::new();
    let mut current = start;
    while current <= end + step {
        values.push(current);
        current += step;
    }
    values
}
