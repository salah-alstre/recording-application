//! Frame buffers, a reuse pool and cheap per-frame effects (fit-to-canvas, privacy boxes,
//! cursor highlight and click rings).
use std::sync::Arc;

/// Tightly packed BGRA8 pixels.
pub struct FrameBuf {
    pub data: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// Reuses frame allocations: a slot is free again once every `Arc` clone handed out has been dropped.
#[derive(Default)]
pub struct FramePool {
    slots: Vec<Arc<FrameBuf>>,
}

impl FramePool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fills a (possibly recycled) frame of `w`×`h` using `fill` and returns a shared handle.
    pub fn produce(&mut self, w: u32, h: u32, fill: impl FnOnce(&mut Vec<u8>)) -> Arc<FrameBuf> {
        let need = w as usize * h as usize * 4;
        let idx = self.slots.iter().position(|s| {
            Arc::strong_count(s) == 1 && s.data.capacity() >= need && s.w == w && s.h == h
        });
        let idx = match idx {
            Some(i) => i,
            None => {
                if self.slots.len() >= 8 {
                    // drop one unused slot of the wrong size before growing further
                    if let Some(p) = self.slots.iter().position(|s| Arc::strong_count(s) == 1) {
                        self.slots.swap_remove(p);
                    }
                }
                self.slots.push(Arc::new(FrameBuf {
                    data: Vec::with_capacity(need),
                    w,
                    h,
                }));
                self.slots.len() - 1
            }
        };
        let slot = Arc::get_mut(&mut self.slots[idx]).expect("slot is uniquely owned");
        slot.w = w;
        slot.h = h;
        slot.data.clear();
        fill(&mut slot.data);
        slot.data.resize(need, 0);
        self.slots[idx].clone()
    }
}

/// Copies `src` into a `cw`×`ch` canvas (top-left aligned, black padding, cropped if larger).
pub fn fit_into(src: &FrameBuf, cw: u32, ch: u32, out: &mut Vec<u8>) {
    out.clear();
    out.resize(cw as usize * ch as usize * 4, 0);
    // opaque black
    for px in out.chunks_exact_mut(4) {
        px[3] = 255;
    }
    let copy_w = src.w.min(cw) as usize * 4;
    for y in 0..src.h.min(ch) as usize {
        let s = y * src.w as usize * 4;
        let d = y * cw as usize * 4;
        out[d..d + copy_w].copy_from_slice(&src.data[s..s + copy_w]);
    }
}

pub fn fill_rect(buf: &mut [u8], bw: u32, bh: u32, x: i32, y: i32, w: i32, h: i32, bgra: [u8; 4]) {
    let x0 = x.clamp(0, bw as i32) as usize;
    let y0 = y.clamp(0, bh as i32) as usize;
    let x1 = (x + w).clamp(0, bw as i32) as usize;
    let y1 = (y + h).clamp(0, bh as i32) as usize;
    for row in y0..y1 {
        let base = row * bw as usize * 4;
        for px in buf[base + x0 * 4..base + x1 * 4].chunks_exact_mut(4) {
            px.copy_from_slice(&bgra);
        }
    }
}

/// Alpha-blends a filled disc (`ring == 0`) or ring (`ring > 0` = stroke width) centred on (cx, cy).
pub fn draw_disc(
    buf: &mut [u8],
    bw: u32,
    bh: u32,
    cx: i32,
    cy: i32,
    radius: i32,
    ring: i32,
    bgr: [u8; 3],
    alpha: f32,
) {
    let r2 = (radius * radius) as f32;
    let inner2 = ((radius - ring).max(0) * (radius - ring).max(0)) as f32;
    for y in (cy - radius).max(0)..(cy + radius + 1).min(bh as i32) {
        for x in (cx - radius).max(0)..(cx + radius + 1).min(bw as i32) {
            let d2 = ((x - cx) * (x - cx) + (y - cy) * (y - cy)) as f32;
            if d2 > r2 || (ring > 0 && d2 < inner2) {
                continue;
            }
            let i = (y as usize * bw as usize + x as usize) * 4;
            for c in 0..3 {
                buf[i + c] = (buf[i + c] as f32 * (1.0 - alpha) + bgr[c] as f32 * alpha) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_recycles_after_release() {
        let mut p = FramePool::new();
        let a = p.produce(4, 4, |v| v.extend([1u8; 64]));
        let ptr = a.data.as_ptr();
        drop(a);
        let b = p.produce(4, 4, |v| v.extend([2u8; 64]));
        assert_eq!(b.data.as_ptr(), ptr, "allocation should be reused");
        assert_eq!(b.data[0], 2);
    }

    #[test]
    fn pool_does_not_reuse_frames_still_in_flight() {
        let mut p = FramePool::new();
        let a = p.produce(2, 2, |v| v.extend([1u8; 16]));
        let b = p.produce(2, 2, |v| v.extend([2u8; 16]));
        assert_ne!(a.data.as_ptr(), b.data.as_ptr());
        assert_eq!(a.data[0], 1);
    }

    #[test]
    fn fit_pads_and_crops() {
        let src = FrameBuf {
            data: vec![9u8; 2 * 2 * 4],
            w: 2,
            h: 2,
        };
        let mut out = vec![];
        fit_into(&src, 3, 3, &mut out);
        assert_eq!(out.len(), 36);
        assert_eq!(out[0], 9); // copied
        assert_eq!(out[2 * 4], 0); // padded pixel (2,0) is black
        assert_eq!(out[2 * 4 + 3], 255);
        fit_into(&src, 1, 1, &mut out);
        assert_eq!(out.len(), 4);
        assert_eq!(out[0], 9);
    }

    #[test]
    fn fill_rect_clips_to_buffer() {
        let mut b = vec![0u8; 4 * 4 * 4];
        fill_rect(&mut b, 4, 4, -2, -2, 4, 4, [1, 2, 3, 255]);
        assert_eq!(&b[0..4], &[1, 2, 3, 255]); // (0,0) inside
        assert_eq!(&b[(2 * 4 + 2) * 4..(2 * 4 + 2) * 4 + 4], &[0, 0, 0, 0]); // (2,2) outside
    }

    #[test]
    fn disc_stays_within_bounds() {
        let mut b = vec![0u8; 8 * 8 * 4];
        draw_disc(&mut b, 8, 8, 0, 0, 5, 0, [255, 255, 255], 1.0);
        assert!(b[0] == 255);
    }
}
