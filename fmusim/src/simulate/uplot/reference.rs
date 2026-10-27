/// Resample reference samples onto the simulation time grid (linear interpolation).
pub fn resample_to_sim_time(sim_time: &[f64], ref_time: &[f64], ref_values: &[f64]) -> Option<Vec<f64>> {
    if ref_time.is_empty() || ref_time.len() != ref_values.len() || sim_time.is_empty() {
        return None;
    }

    if sim_time.len() == ref_time.len() && sim_time == ref_time {
        return Some(ref_values.to_vec());
    }

    let last = ref_time.len() - 1;
    let mut out = Vec::with_capacity(sim_time.len());

    for &t in sim_time {
        if t <= ref_time[0] {
            out.push(ref_values[0]);
            continue;
        }
        if t >= ref_time[last] {
            out.push(ref_values[last]);
            continue;
        }

        let i = ref_time.partition_point(|&rt| rt < t).saturating_sub(1).min(last - 1);
        let t0 = ref_time[i];
        let t1 = ref_time[i + 1];
        let y0 = ref_values[i];
        let y1 = ref_values[i + 1];
        let y = if t1 == t0 { y0 } else { y0 + (y1 - y0) * (t - t0) / (t1 - t0) };
        out.push(y);
    }

    Some(out)
}
