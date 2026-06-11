use anyhow::{bail, Result};
use crate::audio::AudioBuffer;

pub fn fade_in(buf: &mut AudioBuffer, duration_secs: f64) -> Result<()> {
    let frames = buf.num_frames();
    let fade_frames = (duration_secs * buf.sample_rate as f64).round() as usize;
    if fade_frames > frames {
        bail!("Fade in duration ({duration_secs:.2}s) exceeds audio length");
    }
    for ch in buf.samples.iter_mut() {
        for i in 0..fade_frames {
            ch[i] *= i as f32 / fade_frames as f32;
        }
    }
    Ok(())
}

pub fn fade_out(buf: &mut AudioBuffer, duration_secs: f64) -> Result<()> {
    let frames = buf.num_frames();
    let fade_frames = (duration_secs * buf.sample_rate as f64).round() as usize;
    if fade_frames > frames {
        bail!("Fade out duration ({duration_secs:.2}s) exceeds audio length");
    }
    let start = frames - fade_frames;
    for ch in buf.samples.iter_mut() {
        for i in 0..fade_frames {
            ch[start + i] *= 1.0 - (i as f32 / fade_frames as f32);
        }
    }
    Ok(())
}
