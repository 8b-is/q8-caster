use image::{DynamicImage, Rgba, RgbaImage};

use crate::Result;

pub struct AudioRenderer {
    // Audio rendering state
}

impl AudioRenderer {
    pub fn new() -> Self {
        Self {}
    }

    /// Render audio waveform visualization
    pub fn render_waveform(&self, samples: &[f32], width: u32, height: u32) -> Result<DynamicImage> {
        let mut img = RgbaImage::new(width, height);

        // Background color (dark)
        for pixel in img.pixels_mut() {
            *pixel = Rgba([20, 20, 30, 255]);
        }

        if samples.is_empty() {
            return Ok(DynamicImage::ImageRgba8(img));
        }

        let samples_per_pixel = samples.len() / width as usize;
        let mid_height = height / 2;

        // Draw waveform
        for x in 0..width {
            let sample_start = (x as usize) * samples_per_pixel;
            let sample_end = ((x + 1) as usize * samples_per_pixel).min(samples.len());

            if sample_start >= samples.len() {
                break;
            }

            // Get min and max for this pixel column
            let mut min_val = 1.0f32;
            let mut max_val = -1.0f32;

            for sample in &samples[sample_start..sample_end] {
                min_val = min_val.min(*sample);
                max_val = max_val.max(*sample);
            }

            // Convert to pixel coordinates
            let y_min = (mid_height as f32 - (max_val * mid_height as f32)) as u32;
            let y_max = (mid_height as f32 - (min_val * mid_height as f32)) as u32;

            // Draw vertical line for this sample range
            let color = Rgba([100, 200, 255, 255]);
            for y in y_min..=y_max.min(height - 1) {
                img.put_pixel(x, y, color);
            }
        }

        // Draw center line
        let center_color = Rgba([80, 80, 100, 128]);
        for x in 0..width {
            img.put_pixel(x, mid_height, center_color);
        }

        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Render frequency spectrum visualization
    pub fn render_spectrum(&self, frequencies: &[f32], width: u32, height: u32) -> Result<DynamicImage> {
        let mut img = RgbaImage::new(width, height);

        // Background color (dark)
        for pixel in img.pixels_mut() {
            *pixel = Rgba([20, 20, 30, 255]);
        }

        if frequencies.is_empty() {
            return Ok(DynamicImage::ImageRgba8(img));
        }

        let bars = width.min(frequencies.len() as u32);
        let bar_width = width / bars;

        for i in 0..bars {
            let freq_index = (i as f32 / bars as f32 * frequencies.len() as f32) as usize;
            let magnitude = frequencies.get(freq_index).copied().unwrap_or(0.0);

            // Normalize magnitude (assuming 0.0 to 1.0 range)
            let bar_height = (magnitude * height as f32) as u32;

            // Color gradient based on height (low = blue, high = red)
            let hue = (1.0 - magnitude) * 240.0; // Blue to red
            let color = hsv_to_rgb(hue, 1.0, magnitude.max(0.3));

            // Draw bar
            for x in (i * bar_width)..((i + 1) * bar_width).min(width) {
                for y in (height.saturating_sub(bar_height))..height {
                    img.put_pixel(x, y, Rgba([color.0, color.1, color.2, 255]));
                }
            }
        }

        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Render circular audio level meter
    pub fn render_level_meter(&self, level: f32, width: u32, height: u32) -> Result<DynamicImage> {
        let mut img = RgbaImage::new(width, height);

        // Background color (dark)
        for pixel in img.pixels_mut() {
            *pixel = Rgba([20, 20, 30, 255]);
        }

        let center_x = width as f32 / 2.0;
        let center_y = height as f32 / 2.0;
        let radius = width.min(height) as f32 / 2.0 - 10.0;

        // Draw outer circle
        draw_circle(&mut img, center_x, center_y, radius, Rgba([60, 60, 80, 255]), false);

        // Draw level arc
        let level_clamped = level.clamp(0.0, 1.0);
        let angle = level_clamped * std::f32::consts::PI * 1.5; // 270 degrees max

        let color = if level_clamped > 0.9 {
            Rgba([255, 50, 50, 255]) // Red for clipping
        } else if level_clamped > 0.7 {
            Rgba([255, 200, 50, 255]) // Yellow for high
        } else {
            Rgba([50, 255, 100, 255]) // Green for normal
        };

        draw_arc(&mut img, center_x, center_y, radius - 5.0, 0.0, angle, color);

        // Draw center text (level percentage)
        // Note: For actual text, we'd need a font rendering library like rusttype

        Ok(DynamicImage::ImageRgba8(img))
    }
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_prime = h / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn draw_circle(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, color: Rgba<u8>, fill: bool) {
    let (width, height) = img.dimensions();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if fill {
                if dist <= radius {
                    img.put_pixel(x, y, color);
                }
            } else {
                if (dist - radius).abs() < 2.0 {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }
}

fn draw_arc(
    img: &mut RgbaImage,
    cx: f32,
    cy: f32,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    color: Rgba<u8>,
) {
    let (width, height) = img.dimensions();
    let thickness = 10.0;

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx) + std::f32::consts::PI / 2.0;
            let angle = if angle < 0.0 {
                angle + 2.0 * std::f32::consts::PI
            } else {
                angle
            };

            if dist >= radius - thickness / 2.0
                && dist <= radius + thickness / 2.0
                && angle >= start_angle
                && angle <= end_angle
            {
                img.put_pixel(x, y, color);
            }
        }
    }
}
