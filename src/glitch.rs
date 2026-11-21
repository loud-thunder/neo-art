use crate::genome::GlitchStyle;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[inline]
fn pixel_luminance(px: [u8; 4]) -> u16 {
    ((px[0] as u32 * 54 + px[1] as u32 * 183 + px[2] as u32 * 19) >> 8) as u16
}

/// Applies Layer 7 (Pixel Sorting & Scanline Slice Displacement) in-place to an RGBA8 pixel buffer
pub fn apply_glitch_layer(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    style: GlitchStyle,
    intensity: f32,
    seed: u64,
) {
    if style == GlitchStyle::PristineClean || intensity <= 0.01 {
        return;
    }

    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x7777_1337_BEEF_CAFE);
    let w = width as usize;
    let h = height as usize;

    match style {
        GlitchStyle::PristineClean => {}
        GlitchStyle::HorizontalLuminancePixelSort => {
            horizontal_pixel_sort(pixels, w, h, intensity, &mut rng);
        }
        GlitchStyle::VerticalGravityPixelSort => {
            vertical_pixel_sort(pixels, w, h, intensity, &mut rng);
        }
        GlitchStyle::ScanlineSliceDisplacement => {
            scanline_slice_displacement(pixels, w, h, intensity, &mut rng, false);
        }
        GlitchStyle::DualSortAndScanlineShear => {
            horizontal_pixel_sort(pixels, w, h, intensity * 0.85, &mut rng);
            scanline_slice_displacement(pixels, w, h, intensity * 0.9, &mut rng, false);
        }
        GlitchStyle::MicroScanlineJitter => {
            scanline_slice_displacement(pixels, w, h, intensity * 0.55, &mut rng, true);
        }
    }
}

fn horizontal_pixel_sort(
    pixels: &mut [u8],
    w: usize,
    h: usize,
    intensity: f32,
    rng: &mut ChaCha8Rng,
) {
    let low_thresh = (55.0 + (1.0 - intensity) * 45.0) as u16;
    let high_thresh = (175.0 + intensity * 65.0) as u16;
    let max_span = ((w as f32) * (0.08 + intensity * 0.28)).round() as usize;
    let band_height = (h / 64).max(1);

    let mut row_active = false;
    for y in 0..h {
        if y % band_height == 0 {
            row_active = rng.gen_bool((0.28 + intensity as f64 * 0.55).clamp(0.1, 0.9));
        }
        if !row_active {
            continue;
        }

        let row_offset = y * w * 4;
        let mut x = 0;
        while x < w {
            let p_idx = row_offset + x * 4;
            let lum = pixel_luminance([
                pixels[p_idx],
                pixels[p_idx + 1],
                pixels[p_idx + 2],
                pixels[p_idx + 3],
            ]);

            if lum >= low_thresh && lum <= high_thresh {
                let start = x;
                while x < w && (x - start) < max_span {
                    let idx = row_offset + x * 4;
                    let l = pixel_luminance([
                        pixels[idx],
                        pixels[idx + 1],
                        pixels[idx + 2],
                        pixels[idx + 3],
                    ]);
                    if l < low_thresh || l > high_thresh {
                        break;
                    }
                    x += 1;
                }
                let span_len = x - start;
                if span_len > 3 {
                    let mut span: Vec<[u8; 4]> = (start..x)
                        .map(|xi| {
                            let i = row_offset + xi * 4;
                            [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
                        })
                        .collect();
                    span.sort_unstable_by_key(|px| pixel_luminance(*px));
                    for (offset, px) in span.into_iter().enumerate() {
                        let i = row_offset + (start + offset) * 4;
                        pixels[i..i + 4].copy_from_slice(&px);
                    }
                }
            } else {
                x += 1;
            }
        }
    }
}

fn vertical_pixel_sort(
    pixels: &mut [u8],
    w: usize,
    h: usize,
    intensity: f32,
    rng: &mut ChaCha8Rng,
) {
    let low_thresh = (60.0 + (1.0 - intensity) * 40.0) as u16;
    let high_thresh = (180.0 + intensity * 60.0) as u16;
    let max_span = ((h as f32) * (0.07 + intensity * 0.25)).round() as usize;
    let col_group = (w / 64).max(1);

    let mut col_active = false;
    for x in 0..w {
        if x % col_group == 0 {
            col_active = rng.gen_bool((0.25 + intensity as f64 * 0.50).clamp(0.1, 0.85));
        }
        if !col_active {
            continue;
        }

        let mut y = 0;
        while y < h {
            let p_idx = (y * w + x) * 4;
            let lum = pixel_luminance([
                pixels[p_idx],
                pixels[p_idx + 1],
                pixels[p_idx + 2],
                pixels[p_idx + 3],
            ]);

            if lum >= low_thresh && lum <= high_thresh {
                let start = y;
                while y < h && (y - start) < max_span {
                    let idx = (y * w + x) * 4;
                    let l = pixel_luminance([
                        pixels[idx],
                        pixels[idx + 1],
                        pixels[idx + 2],
                        pixels[idx + 3],
                    ]);
                    if l < low_thresh || l > high_thresh {
                        break;
                    }
                    y += 1;
                }
                let span_len = y - start;
                if span_len > 3 {
                    let mut span: Vec<[u8; 4]> = (start..y)
                        .map(|yi| {
                            let i = (yi * w + x) * 4;
                            [pixels[i], pixels[i + 1], pixels[i + 2], pixels[i + 3]]
                        })
                        .collect();
                    span.sort_unstable_by_key(|px| std::cmp::Reverse(pixel_luminance(*px)));
                    for (offset, px) in span.into_iter().enumerate() {
                        let i = ((start + offset) * w + x) * 4;
                        pixels[i..i + 4].copy_from_slice(&px);
                    }
                }
            } else {
                y += 1;
            }
        }
    }
}

fn scanline_slice_displacement(
    pixels: &mut [u8],
    w: usize,
    h: usize,
    intensity: f32,
    rng: &mut ChaCha8Rng,
    micro_mode: bool,
) {
    let num_slices = if micro_mode {
        (24.0 + intensity * 40.0) as usize
    } else {
        (10.0 + intensity * 24.0) as usize
    };

    let mut row_buf = vec![0u8; w * 4];

    for _ in 0..num_slices {
        let y_center_norm: f32 = rng.gen_range(0.04..0.96);
        let slice_h_norm: f32 = if micro_mode {
            rng.gen_range(0.002..0.010)
        } else {
            rng.gen_range(0.005..0.032)
        };
        let shift_norm: f32 = if micro_mode {
            rng.gen_range(-0.025..0.025) * intensity
        } else {
            rng.gen_range(-0.085..0.085) * intensity
        };

        let y_start = ((y_center_norm * h as f32) as usize).min(h - 1);
        let slice_h = ((slice_h_norm * h as f32).round() as usize).max(1);
        let y_end = (y_start + slice_h).min(h);
        let shift_px = (shift_norm * w as f32).round() as isize;

        if shift_px == 0 {
            continue;
        }

        for y in y_start..y_end {
            let row_start = y * w * 4;
            row_buf.copy_from_slice(&pixels[row_start..row_start + w * 4]);
            for x in 0..w {
                let src_x = (x as isize - shift_px).rem_euclid(w as isize) as usize;
                let dst_i = row_start + x * 4;
                let src_i = src_x * 4;
                pixels[dst_i..dst_i + 4].copy_from_slice(&row_buf[src_i..src_i + 4]);
            }
        }
    }
}
