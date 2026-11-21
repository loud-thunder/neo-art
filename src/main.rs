mod app;
mod cellular;
mod export;
mod genome;
mod glitch;
mod renderer;

use app::NeoArtApp;
use eframe::egui;
use export::{export_batch_backgrounds, AspectRatioPreset, ExportConfig};

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Support flexible CLI headless batch export (`cargo run --release -- --export-batch [options]`)
    if args.iter().any(|a| a == "--export-batch") {
        let mut count: u32 = 50;
        let mut width: u32 = 3840;
        let mut height: u32 = 2160;
        let mut preset = AspectRatioPreset::Landscape16x9_4K;
        let mut output_dir = "output".to_string();
        let mut prefix = "background".to_string();
        let mut seed: u64 = 2026_11_21;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--count" if i + 1 < args.len() => {
                    if let Ok(c) = args[i + 1].parse() {
                        count = c;
                    }
                    i += 1;
                }
                "--width" if i + 1 < args.len() => {
                    if let Ok(w) = args[i + 1].parse() {
                        width = w;
                        preset = AspectRatioPreset::Custom;
                    }
                    i += 1;
                }
                "--height" if i + 1 < args.len() => {
                    if let Ok(h) = args[i + 1].parse() {
                        height = h;
                        preset = AspectRatioPreset::Custom;
                    }
                    i += 1;
                }
                "--aspect" if i + 1 < args.len() => {
                    match args[i + 1].to_lowercase().as_str() {
                        "16:9" | "4k" => {
                            preset = AspectRatioPreset::Landscape16x9_4K;
                            width = 3840;
                            height = 2160;
                        }
                        "1080p" | "fhd" => {
                            preset = AspectRatioPreset::Landscape16x9_1080p;
                            width = 1920;
                            height = 1080;
                        }
                        "9:16" | "mobile" => {
                            preset = AspectRatioPreset::Mobile9x16Story;
                            width = 1080;
                            height = 1920;
                        }
                        "21:9" | "ultrawide" => {
                            preset = AspectRatioPreset::Ultrawide21x9;
                            width = 3440;
                            height = 1440;
                        }
                        "1:1" | "square" => {
                            preset = AspectRatioPreset::Square1x1;
                            width = 2048;
                            height = 2048;
                        }
                        "4:3" => {
                            preset = AspectRatioPreset::Classic4x3;
                            width = 1600;
                            height = 1200;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                "--output-dir" if i + 1 < args.len() => {
                    output_dir = args[i + 1].clone();
                    i += 1;
                }
                "--prefix" if i + 1 < args.len() => {
                    prefix = args[i + 1].clone();
                    i += 1;
                }
                "--seed" if i + 1 < args.len() => {
                    if let Ok(s) = args[i + 1].parse() {
                        seed = s;
                    }
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }

        println!("Neo-Art Procedural Background Batch Exporter");
        println!("===========================================");
        println!("Generating {} unique backgrounds (seed={})...", count, seed);
        let batch = genome::generate_batch(seed, count);

        let config = ExportConfig {
            width,
            height,
            preset,
            output_dir: output_dir.clone(),
            filename_prefix: prefix,
        };

        println!(
            "Rendering {} backgrounds at {}×{} (aspect ratio {:.3}:1) to {}/...",
            count, width, height, width as f32 / height as f32, output_dir
        );

        let progress = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        export_batch_backgrounds(&batch, &config, progress).expect("Batch export succeeded");

        println!("Done! Successfully exported {} PNG backgrounds.", count);
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1420.0, 900.0])
            .with_min_inner_size([1024.0, 680.0])
            .with_title("Neo-Art — Procedural Abstract Background & Wallpaper Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "Neo-Art Background Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(NeoArtApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use crate::export::{export_batch_backgrounds, AspectRatioPreset, ExportConfig};
    use crate::genome::{generate_batch, SecondaryGeometry};
    use crate::renderer::{render_background, LayerVisibility};
    use std::collections::HashSet;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    #[test]
    fn test_batch_uniqueness_and_co_occurrence_rules() {
        let batch = generate_batch(2026_11_21, 50);
        assert_eq!(batch.len(), 50);

        let mut signatures = HashSet::new();
        for g in &batch {
            assert!(
                signatures.insert(g.signature()),
                "Duplicate trait signature detected for background #{}",
                g.id
            );

            // Verify Co-occurrence repulsion rules are strictly respected
            let affinity =
                SecondaryGeometry::co_occurrence_weight(g.primary_geom, g.secondary_geom);
            assert!(
                affinity > 0,
                "Forbidden co-occurrence pairing ({:?}, {:?}) on background #{}",
                g.primary_geom,
                g.secondary_geom,
                g.id
            );
            assert_eq!(affinity, g.co_occurrence_affinity);
        }
    }

    #[test]
    fn test_deterministic_isotropic_rendering() {
        let batch = generate_batch(999_888_777, 5);
        let genome = &batch[2];
        let vis = LayerVisibility::default();

        // Test non-square widescreen (16:9) rendering determinism
        let render_16_9_a = render_background(genome, 320, 180, vis);
        let render_16_9_b = render_background(genome, 320, 180, vis);
        assert_eq!(render_16_9_a.len(), 320 * 180 * 4);
        assert_eq!(
            render_16_9_a, render_16_9_b,
            "16:9 widescreen render must be 100% deterministic"
        );

        // Test non-square portrait mobile (9:16) rendering determinism
        let render_9_16_a = render_background(genome, 180, 320, vis);
        let render_9_16_b = render_background(genome, 180, 320, vis);
        assert_eq!(render_9_16_a.len(), 180 * 320 * 4);
        assert_eq!(
            render_9_16_a, render_9_16_b,
            "9:16 portrait mobile render must be 100% deterministic"
        );
    }

    #[test]
    fn test_batch_export_png_output() {
        let batch = generate_batch(42, 3);
        let tmp_dir = std::env::temp_dir().join("neo_art_test_bg_export");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let config = ExportConfig {
            width: 256,
            height: 144,
            preset: AspectRatioPreset::Landscape16x9_1080p,
            output_dir: tmp_dir.to_string_lossy().to_string(),
            filename_prefix: "bg_test".to_string(),
        };

        let progress = Arc::new(AtomicUsize::new(0));
        export_batch_backgrounds(&batch, &config, progress).unwrap();

        for id in 1..=3 {
            let p = tmp_dir.join(format!("bg_test_{:03}.png", id));
            assert!(p.exists(), "Exported image {:?} must exist on disk", p);
            let metadata = std::fs::metadata(&p).unwrap();
            assert!(metadata.len() > 100, "PNG file must not be empty");
        }

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
