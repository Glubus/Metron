use super::super::interpolation::{search_left, search_right};
use super::super::note::Note;
use super::super::smoothing::{SmoothMode, smooth_on_corners_into};

pub fn compute_pbar(
    hit_leniency: f64,
    notes: &[Note],
    anchor: &[f64],
    base_corners: &[f64],
) -> Vec<f64> {
    let mut p_step = vec![0.0; base_corners.len()];

    for pair in notes.windows(2) {
        let h_l = pair[0].hit_time;
        let h_r = pair[1].hit_time;
        let delta_time = h_r - h_l;

        if delta_time == 0 {
            let spike = 1000.0 * (0.02 * (4.0 / hit_leniency - 24.0)).powf(0.25);
            let li = search_left(base_corners, h_l as f64);
            let ri = search_right(base_corners, h_l as f64);
            for value in &mut p_step[li..ri] {
                *value += spike;
            }
            continue;
        }

        let li = search_left(base_corners, h_l as f64);
        let ri = search_left(base_corners, h_r as f64);
        if ri <= li {
            continue;
        }

        let delta = 0.001 * f64::from(delta_time as i32);
        let b_val = stream_booster(delta);
        let base_inc = (0.08
            * hit_leniency.powi(-1)
            * (1.0 - 24.0 * hit_leniency.powi(-1) * (hit_leniency / 6.0).powi(2)))
        .powf(0.25);

        let inc = if delta < 2.0 * hit_leniency / 3.0 {
            delta.powi(-1)
                * (0.08
                    * hit_leniency.powi(-1)
                    * (1.0 - 24.0 * hit_leniency.powi(-1) * (delta - hit_leniency / 2.0).powi(2)))
                .powf(0.25)
                * b_val.max(1.0)
        } else {
            delta.powi(-1) * base_inc * b_val.max(1.0)
        };

        for (value, &seg_anchor) in p_step[li..ri].iter_mut().zip(&anchor[li..ri]) {
            *value += (inc * seg_anchor).min(inc.max(inc * 2.0 - 10.0));
        }
    }

    let mut out = vec![0.0; base_corners.len()];
    smooth_on_corners_into(
        base_corners,
        &p_step,
        500.0,
        0.001,
        SmoothMode::Sum,
        &mut out,
    );
    out
}

fn stream_booster(delta: f64) -> f64 {
    let bpm = (7.5 / delta).clamp(0.0, 420.0);
    let primary = 0.10 / (1.0 + (-0.06 * (bpm - 175.0)).exp());
    let secondary = if (200.0..=350.0).contains(&bpm) {
        0.30 * (1.0 - (-0.02 * (bpm - 200.0)).exp())
    } else {
        0.0
    };
    1.0 + primary + secondary
}
