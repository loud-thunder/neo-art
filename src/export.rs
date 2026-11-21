use crate::genome::BackgroundGenome;
use crate::renderer::{render_background, LayerVisibility};
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AspectRatioPreset {
    Landscape16x9_4K,
    Landscape16x9_1080p,
    Mobile9x16Story,
    Ultrawide21x9,
    Square1x1,
    Classic4x3,
    Custom,
}

impl AspectRatioPreset {
    pub const ALL: &'static [Self] = &[
        Self::Landscape16x9_4K,
        Self::Landscape16x9_1080p,
        Self::Mobile9x16Story,
        Self::Ultrawide21x9,
        Self::Square1x1,
        Self::Classic4x3,
        Self::Custom,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Landscape16x9_4K => "16:9 4K UHD (3840×2160)",
            Self::Landscape16x9_1080p => "16:9 1080p FHD (1920×1080)",
            Self::Mobile9x16Story => "9:16 Mobile / Story (1080×1920)",
            Self::Ultrawide21x9 => "21:9 Ultrawide (3440×1440)",
            Self::Square1x1 => "1:1 Square (2048×2048)",
            Self::Classic4x3 => "4:3 Classic (1600×1200)",
            Self::Custom => "Custom Resolution",
        }
    }

    pub fn dimensions(&self) -> Option<(u32, u32)> {
        match self {
            Self::Landscape16x9_4K => Some((3840, 2160)),
            Self::Landscape16x9_1080p => Some((1920, 1080)),
            Self::Mobile9x16Story => Some((1080, 1920)),
            Self::Ultrawide21x9 => Some((3440, 1440)),
            Self::Square1x1 => Some((2048, 2048)),
            Self::Classic4x3 => Some((1600, 1200)),
            Self::Custom => None,
        }
    }

    pub fn from_dimensions(w: u32, h: u32) -> Self {
        for &preset in Self::ALL {
            if preset.dimensions() == Some((w, h)) {
                return preset;
            }
        }
        Self::Custom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub width: u32,
    pub height: u32,
    pub preset: AspectRatioPreset,
    pub output_dir: String,
    pub filename_prefix: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            width: 3840,
            height: 2160,
            preset: AspectRatioPreset::Landscape16x9_4K,
            output_dir: "output".to_string(),
            filename_prefix: "background".to_string(),
        }
    }
}


/// Exports a single background image to disk as a high-resolution PNG
pub fn export_single_background(
    genome: &BackgroundGenome,
    config: &ExportConfig,
    vis: LayerVisibility,
) -> Result<PathBuf, String> {
    let out_dir = Path::new(&config.output_dir);
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let rgba = render_background(genome, config.width, config.height, vis);
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(config.width, config.height, rgba)
            .ok_or("Failed to construct image buffer")?;

    let img_path = out_dir.join(format!("{}_{:03}.png", config.filename_prefix, genome.id));
    img.save(&img_path).map_err(|e| e.to_string())?;

    Ok(img_path)
}

/// Multi-threaded batch export of backgrounds to high-resolution PNGs
pub fn export_batch_backgrounds(
    batch: &[BackgroundGenome],
    config: &ExportConfig,
    progress_counter: Arc<AtomicUsize>,
) -> Result<(), String> {
    let out_dir = Path::new(&config.output_dir);
    fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;

    let vis = LayerVisibility::default();

    batch
        .par_iter()
        .try_for_each(|genome| -> Result<(), String> {
            export_single_background(genome, config, vis)?;
            progress_counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })?;

    Ok(())
}
