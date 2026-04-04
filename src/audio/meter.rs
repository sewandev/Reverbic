pub fn rms_to_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return -60.0;
    }
    let mean_sq: f32 = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    if mean_sq <= 0.0 {
        return -60.0;
    }
    let db = 20.0 * mean_sq.sqrt().log10();
    db.clamp(-60.0, 0.0)
}
