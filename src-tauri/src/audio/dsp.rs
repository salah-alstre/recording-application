//! Small, allocation-free DSP blocks for the microphone chain: gain → gate → compressor → limiter.
//! All operate on interleaved stereo `f32` at 48 kHz.

pub const RATE: f32 = 48_000.0;

pub fn db_to_lin(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}
pub fn lin_to_db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

fn coeff(ms: f32) -> f32 {
    (-1.0 / (ms * 0.001 * RATE)).exp()
}

/// Downward expander that mutes signal below a threshold (with attack/release smoothing and hold).
pub struct Gate {
    env: f32,
    gain: f32,
    hold: u32,
}
impl Gate {
    pub fn new() -> Self {
        Self {
            env: 0.0,
            gain: 0.0,
            hold: 0,
        }
    }
    pub fn process(&mut self, buf: &mut [f32], threshold_db: f32) {
        let thr = db_to_lin(threshold_db);
        let (att, rel) = (coeff(5.0), coeff(120.0));
        let hold_frames = (0.08 * RATE) as u32;
        for f in buf.chunks_exact_mut(2) {
            let level = f[0].abs().max(f[1].abs());
            self.env = if level > self.env {
                level
            } else {
                self.env * coeff(30.0) + level * (1.0 - coeff(30.0))
            };
            let open = self.env > thr;
            if open {
                self.hold = hold_frames;
            } else if self.hold > 0 {
                self.hold -= 1;
            }
            let target = if open || self.hold > 0 { 1.0 } else { 0.0 };
            let c = if target > self.gain { att } else { rel };
            self.gain = self.gain * c + target * (1.0 - c);
            f[0] *= self.gain;
            f[1] *= self.gain;
        }
    }
}

/// Feed-forward compressor: -18 dB threshold, 3:1 ratio, 10 ms attack, 120 ms release, auto make-up.
pub struct Compressor {
    env_db: f32,
}
impl Compressor {
    pub fn new() -> Self {
        Self { env_db: -90.0 }
    }
    pub fn process(&mut self, buf: &mut [f32]) {
        const THR: f32 = -18.0;
        const RATIO: f32 = 3.0;
        const MAKEUP_DB: f32 = 4.0;
        let (att, rel) = (coeff(10.0), coeff(120.0));
        for f in buf.chunks_exact_mut(2) {
            let l = lin_to_db(f[0].abs().max(f[1].abs()));
            let c = if l > self.env_db { att } else { rel };
            self.env_db = self.env_db * c + l * (1.0 - c);
            let over = (self.env_db - THR).max(0.0);
            let gain_db = -over * (1.0 - 1.0 / RATIO) + MAKEUP_DB;
            let g = db_to_lin(gain_db);
            f[0] *= g;
            f[1] *= g;
        }
    }
}

/// Look-ahead-free brick-wall limiter: instant attack, smooth release. Output never exceeds `ceiling`.
pub struct Limiter {
    gain: f32,
}
impl Limiter {
    pub fn new() -> Self {
        Self { gain: 1.0 }
    }
    pub fn process(&mut self, buf: &mut [f32], ceiling_db: f32) {
        let ceil = db_to_lin(ceiling_db);
        let rel = coeff(80.0);
        for f in buf.chunks_exact_mut(2) {
            let peak = f[0].abs().max(f[1].abs());
            let need = if peak > ceil { ceil / peak } else { 1.0 };
            self.gain = if need < self.gain {
                need
            } else {
                self.gain * rel + need * (1.0 - rel)
            };
            f[0] = (f[0] * self.gain).clamp(-ceil, ceil);
            f[1] = (f[1] * self.gain).clamp(-ceil, ceil);
        }
    }
}

pub fn peak(buf: &[f32]) -> f32 {
    buf.iter().fold(0.0f32, |m, s| m.max(s.abs()))
}

/// Streaming linear resampler to 48 kHz stereo.
pub struct Resampler {
    step: f64,
    pos: f64,
    prev: [f32; 2],
}
impl Resampler {
    pub fn new(src_rate: u32) -> Self {
        Self {
            step: src_rate as f64 / RATE as f64,
            pos: 0.0,
            prev: [0.0; 2],
        }
    }
    /// `input`: interleaved stereo at the source rate. Appends 48 kHz interleaved stereo to `out`.
    pub fn process(&mut self, input: &[f32], out: &mut Vec<f32>) {
        if (self.step - 1.0).abs() < 1e-9 {
            out.extend_from_slice(input);
            return;
        }
        let frames = input.len() / 2;
        // position 0.0 refers to `prev`, position k (k>=1) to input frame k-1
        while self.pos < frames as f64 {
            let i = self.pos.floor() as usize;
            let frac = (self.pos - i as f64) as f32;
            let a = if i == 0 {
                self.prev
            } else {
                [input[(i - 1) * 2], input[(i - 1) * 2 + 1]]
            };
            let b = [input[i * 2], input[i * 2 + 1]];
            out.push(a[0] + (b[0] - a[0]) * frac);
            out.push(a[1] + (b[1] - a[1]) * frac);
            self.pos += self.step;
        }
        self.pos -= frames as f64;
        if frames > 0 {
            self.prev = [input[(frames - 1) * 2], input[(frames - 1) * 2 + 1]];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(amp: f32, frames: usize) -> Vec<f32> {
        (0..frames)
            .flat_map(|i| {
                let v = amp * (i as f32 * 0.05).sin();
                [v, v]
            })
            .collect()
    }

    #[test]
    fn db_round_trip() {
        assert!((lin_to_db(db_to_lin(-12.0)) + 12.0).abs() < 1e-3);
        assert!((db_to_lin(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gate_mutes_quiet_and_passes_loud() {
        let mut g = Gate::new();
        let mut quiet = tone(0.001, 48_000);
        g.process(&mut quiet, -40.0);
        assert!(peak(&quiet[quiet.len() / 2..]) < 0.0005);
        let mut g = Gate::new();
        let mut loud = tone(0.5, 48_000);
        g.process(&mut loud, -40.0);
        assert!(peak(&loud[loud.len() / 2..]) > 0.45);
    }

    #[test]
    fn limiter_never_exceeds_ceiling() {
        let mut l = Limiter::new();
        let mut buf = tone(4.0, 10_000);
        l.process(&mut buf, -1.0);
        assert!(peak(&buf) <= db_to_lin(-1.0) + 1e-6);
    }

    #[test]
    fn compressor_reduces_dynamic_range() {
        let mut c = Compressor::new();
        let mut loud = tone(0.9, 48_000);
        let mut quiet = tone(0.05, 48_000);
        let (l0, q0) = (peak(&loud), peak(&quiet));
        c.process(&mut loud);
        let mut c2 = Compressor::new();
        c2.process(&mut quiet);
        let ratio_before = l0 / q0;
        let ratio_after = peak(&loud[24_000..]) / peak(&quiet[24_000..]);
        assert!(ratio_after < ratio_before);
    }

    #[test]
    fn resampler_produces_expected_frame_count() {
        let mut r = Resampler::new(44_100);
        let mut out = vec![];
        for _ in 0..100 {
            r.process(&tone(0.5, 441), &mut out);
        }
        let frames = out.len() / 2;
        assert!(
            (frames as i64 - 48_000 * 44_100 / 44_100 as i64).abs() < 8,
            "got {frames}"
        );
    }

    #[test]
    fn resampler_is_passthrough_at_48k() {
        let mut r = Resampler::new(48_000);
        let src = tone(0.3, 100);
        let mut out = vec![];
        r.process(&src, &mut out);
        assert_eq!(out, src);
    }
}
