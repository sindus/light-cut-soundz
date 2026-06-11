use anyhow::{bail, Result};
use crate::audio::AudioBuffer;

pub fn trim(buf: &mut AudioBuffer, start_secs: f64, end_secs: f64) -> Result<()> {
    let total = buf.duration_secs();
    if start_secs < 0.0 || end_secs <= start_secs || end_secs > total + 0.001 {
        bail!(
            "Invalid trim range {start_secs:.3}:{end_secs:.3} (audio is {total:.3}s)"
        );
    }
    let start_frame = (start_secs * buf.sample_rate as f64).round() as usize;
    let end_frame = (end_secs * buf.sample_rate as f64)
        .round()
        .min(buf.num_frames() as f64) as usize;

    for ch in buf.samples.iter_mut() {
        *ch = ch[start_frame..end_frame].to_vec();
    }
    Ok(())
}
