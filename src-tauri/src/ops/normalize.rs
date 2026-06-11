use anyhow::{bail, Result};
use crate::audio::AudioBuffer;

pub fn normalize(buf: &mut AudioBuffer) -> Result<()> {
    let peak = buf
        .samples
        .iter()
        .flat_map(|ch| ch.iter())
        .map(|s| s.abs())
        .fold(0.0f32, f32::max);

    if peak < 1e-8 {
        bail!("Audio is silent, cannot normalize");
    }
    for ch in buf.samples.iter_mut() {
        for s in ch.iter_mut() {
            *s /= peak;
        }
    }
    Ok(())
}
