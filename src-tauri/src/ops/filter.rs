use anyhow::{bail, Result};
use std::f64::consts::PI;
use crate::audio::AudioBuffer;

#[derive(Clone)]
struct Biquad {
    b0: f64, b1: f64, b2: f64,
    a1: f64, a2: f64,
    z1: f64, z2: f64,
}

impl Biquad {
    fn process(&mut self, x: f32) -> f32 {
        let x = x as f64;
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y as f32
    }

    fn low_pass(sample_rate: u32, cutoff_hz: f64) -> Result<Self> {
        let sr = sample_rate as f64;
        if cutoff_hz <= 0.0 || cutoff_hz >= sr / 2.0 {
            bail!("Low-pass cutoff {cutoff_hz}Hz out of range (0, {})", sr / 2.0);
        }
        let q = 0.7071067811865476; // 1/sqrt(2)
        let w0 = 2.0 * PI * cutoff_hz / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);
        let b0 = (1.0 - cos_w0) / 2.0;
        let b1 = 1.0 - cos_w0;
        let b2 = (1.0 - cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;
        Ok(Self { b0: b0/a0, b1: b1/a0, b2: b2/a0, a1: a1/a0, a2: a2/a0, z1: 0.0, z2: 0.0 })
    }

    fn high_pass(sample_rate: u32, cutoff_hz: f64) -> Result<Self> {
        let sr = sample_rate as f64;
        if cutoff_hz <= 0.0 || cutoff_hz >= sr / 2.0 {
            bail!("High-pass cutoff {cutoff_hz}Hz out of range (0, {})", sr / 2.0);
        }
        let q = 0.7071067811865476;
        let w0 = 2.0 * PI * cutoff_hz / sr;
        let cos_w0 = w0.cos();
        let alpha = w0.sin() / (2.0 * q);
        let b0 = (1.0 + cos_w0) / 2.0;
        let b1 = -(1.0 + cos_w0);
        let b2 = (1.0 + cos_w0) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;
        Ok(Self { b0: b0/a0, b1: b1/a0, b2: b2/a0, a1: a1/a0, a2: a2/a0, z1: 0.0, z2: 0.0 })
    }

    fn band_pass(sample_rate: u32, center_hz: f64, bandwidth_hz: f64) -> Result<Self> {
        let sr = sample_rate as f64;
        if center_hz <= 0.0 || center_hz >= sr / 2.0 {
            bail!("Band-pass center {center_hz}Hz out of range");
        }
        if bandwidth_hz <= 0.0 {
            bail!("Band-pass bandwidth must be positive");
        }
        let q = center_hz / bandwidth_hz;
        let w0 = 2.0 * PI * center_hz / sr;
        let alpha = w0.sin() / (2.0 * q);
        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;
        Ok(Self { b0: b0/a0, b1: b1/a0, b2: b2/a0, a1: a1/a0, a2: a2/a0, z1: 0.0, z2: 0.0 })
    }
}

pub enum FilterSpec {
    LowPass { cutoff_hz: f64 },
    HighPass { cutoff_hz: f64 },
    BandPass { center_hz: f64, bandwidth_hz: f64 },
}

impl FilterSpec {
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        match parts.as_slice() {
            ["lowpass", hz] => Ok(Self::LowPass { cutoff_hz: hz.parse()? }),
            ["highpass", hz] => Ok(Self::HighPass { cutoff_hz: hz.parse()? }),
            ["bandpass", center, bw] => Ok(Self::BandPass {
                center_hz: center.parse()?,
                bandwidth_hz: bw.parse()?,
            }),
            _ => bail!("Invalid filter spec '{s}'. Use: lowpass:<hz> | highpass:<hz> | bandpass:<hz>:<bw_hz>"),
        }
    }
}

pub fn apply_filter(buf: &mut AudioBuffer, spec: &FilterSpec) -> Result<()> {
    let mut filters: Vec<Biquad> = (0..buf.channels)
        .map(|_| match spec {
            FilterSpec::LowPass { cutoff_hz } => Biquad::low_pass(buf.sample_rate, *cutoff_hz),
            FilterSpec::HighPass { cutoff_hz } => Biquad::high_pass(buf.sample_rate, *cutoff_hz),
            FilterSpec::BandPass { center_hz, bandwidth_hz } => {
                Biquad::band_pass(buf.sample_rate, *center_hz, *bandwidth_hz)
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let frames = buf.num_frames();
    for f in 0..frames {
        for (ch, filt) in filters.iter_mut().enumerate() {
            buf.samples[ch][f] = filt.process(buf.samples[ch][f]);
        }
    }
    Ok(())
}
