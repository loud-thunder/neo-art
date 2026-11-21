use crate::export::{
    export_batch_backgrounds, export_single_background, AspectRatioPreset, ExportConfig,
};
use crate::genome::{
    generate_batch, BackgroundGenome, BackgroundStyle, CellularStyle, ColorPaletteTrait,
    GlitchStyle, HarmonyMode, PrimaryGeometry, ProximityStyle, SecondaryGeometry, SeedDistribution,
};
use crate::renderer::{render_background, LayerVisibility};
use eframe::egui::{self, Color32, ColorImage, RichText, TextureHandle, TextureOptions};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterTab {
    CanvasInspector,
    BatchGallery,
    BatchExport,
}

pub struct NeoArtApp {
    pub master_seed: u64,
    pub batch_count: u32,
    pub batch: Vec<BackgroundGenome>,
    pub selected_index: usize,
    pub layer_vis: LayerVisibility,
    pub active_tab: CenterTab,

    // Aspect ratio & resolution configuration
    pub canvas_preset: AspectRatioPreset,
    pub canvas_width: u32,
    pub canvas_height: u32,

    // Texture cache for single inspector
    pub current_texture: Option<TextureHandle>,
    pub preview_dirty: bool,

    // Thumbnail cache for gallery
    pub grid_textures: HashMap<u32, TextureHandle>,
    pub grid_page: usize,
    pub grid_page_size: usize,

    // Export state
    pub export_config: ExportConfig,
    pub is_exporting: Arc<AtomicBool>,
    pub export_progress: Arc<AtomicUsize>,
    pub status_message: String,
}

impl NeoArtApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let master_seed = 2026_11_21;
        let batch_count = 50;
        let batch = generate_batch(master_seed, batch_count);

        let canvas_preset = AspectRatioPreset::Landscape16x9_4K;
        let (canvas_width, canvas_height) = canvas_preset.dimensions().unwrap_or((3840, 2160));

        let mut export_config = ExportConfig::default();
        export_config.width = canvas_width;
        export_config.height = canvas_height;
        export_config.preset = canvas_preset;

        Self {
            master_seed,
            batch_count,
            batch,
            selected_index: 0,
            layer_vis: LayerVisibility::default(),
            active_tab: CenterTab::CanvasInspector,
            canvas_preset,
            canvas_width,
            canvas_height,
            current_texture: None,
            preview_dirty: true,
            grid_textures: HashMap::new(),
            grid_page: 0,
            grid_page_size: 20,
            export_config,
            is_exporting: Arc::new(AtomicBool::new(false)),
            export_progress: Arc::new(AtomicUsize::new(0)),
            status_message: format!(
                "Ready — {} procedural backgrounds generated from seed {}.",
                batch_count, master_seed
            ),
        }
    }

    fn regenerate_all(&mut self) {
        self.batch = generate_batch(self.master_seed, self.batch_count);
        if self.selected_index >= self.batch.len() {
            self.selected_index = 0;
        }
        self.grid_textures.clear();
        self.preview_dirty = true;
        self.status_message = format!(
            "Regenerated {} unique backgrounds from seed {}.",
            self.batch_count, self.master_seed
        );
    }

    fn refresh_current_preview(&mut self, ctx: &egui::Context) {
        if let Some(genome) = self.batch.get(self.selected_index) {
            // Preview renders at balanced resolution preserving exact aspect ratio
            let aspect = self.canvas_width as f32 / self.canvas_height as f32;
            let (pw, ph) = if aspect >= 1.0 {
                let w = 800u32;
                let h = ((w as f32 / aspect).round() as u32).max(120);
                (w, h)
            } else {
                let h = 800u32;
                let w = ((h as f32 * aspect).round() as u32).max(120);
                (w, h)
            };

            let rgba = render_background(genome, pw, ph, self.layer_vis);
            let color_image = ColorImage::from_rgba_unmultiplied([pw as usize, ph as usize], &rgba);
            self.current_texture = Some(ctx.load_texture(
                format!("bg_preview_{}", genome.id),
                color_image,
                TextureOptions::LINEAR,
            ));
            self.preview_dirty = false;
        }
    }

    fn ensure_grid_page_textures(&mut self, ctx: &egui::Context, visible_ids: &[u32]) {
        let missing: Vec<BackgroundGenome> = visible_ids
            .iter()
            .filter(|id| !self.grid_textures.contains_key(id))
            .filter_map(|&id| self.batch.get((id - 1) as usize).cloned())
            .collect();

        if missing.is_empty() {
            return;
        }

        let aspect = self.canvas_width as f32 / self.canvas_height as f32;
        let (tw, th) = if aspect >= 1.0 {
            let w = 180u32;
            let h = ((w as f32 / aspect).round() as u32).max(40);
            (w, h)
        } else {
            let h = 180u32;
            let w = ((h as f32 * aspect).round() as u32).max(40);
            (w, h)
        };

        let vis = self.layer_vis;
        let rendered: Vec<(u32, Vec<u8>)> = missing
            .par_iter()
            .map(|g| (g.id, render_background(g, tw, th, vis)))
            .collect();

        for (id, rgba) in rendered {
            let img = ColorImage::from_rgba_unmultiplied([tw as usize, th as usize], &rgba);
            let tex = ctx.load_texture(format!("thumb_{}", id), img, TextureOptions::LINEAR);
            self.grid_textures.insert(id, tex);
        }
    }

    fn apply_preset(&mut self, preset: AspectRatioPreset) {
        self.canvas_preset = preset;
        if let Some((w, h)) = preset.dimensions() {
            self.canvas_width = w;
            self.canvas_height = h;
            self.export_config.width = w;
            self.export_config.height = h;
            self.export_config.preset = preset;
            self.grid_textures.clear();
            self.preview_dirty = true;
        }
    }
}

impl eframe::App for NeoArtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.preview_dirty {
            self.refresh_current_preview(ctx);
        }

        if self.is_exporting.load(Ordering::Relaxed) {
            ctx.request_repaint();
        }

        self.render_top_bar(ctx);
        self.render_left_sidebar(ctx);
        self.render_right_inspector(ctx);
        self.render_center_panel(ctx);
    }
}

impl NeoArtApp {
    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_nav").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("NEO-ART // BACKGROUND STUDIO").strong());
                ui.separator();

                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::CanvasInspector,
                    "1. Background Canvas",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::BatchGallery,
                    "2. Batch Gallery",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::BatchExport,
                    "3. Batch Export",
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(&self.status_message).italics().small());
                });
            });
            ui.add_space(4.0);
        });
    }

    fn render_left_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_controls")
            .resizable(true)
            .default_width(330.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Batch Seed & Index");
                    ui.horizontal(|ui| {
                        ui.label("Master Seed:");
                        if ui
                            .add(egui::DragValue::new(&mut self.master_seed).speed(1.0))
                            .changed()
                        {
                            self.regenerate_all();
                        }
                        if ui.button("Randomize").clicked() {
                            self.master_seed = rand::random::<u64>() & 0xFFFF_FFFF;
                            self.regenerate_all();
                        }
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("<< Prev").clicked() && self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.preview_dirty = true;
                        }
                        let mut num = (self.selected_index + 1) as u32;
                        if ui
                            .add(
                                egui::Slider::new(&mut num, 1..=self.batch_count)
                                    .text("Background #"),
                            )
                            .changed()
                        {
                            self.selected_index = (num - 1) as usize;
                            self.preview_dirty = true;
                        }
                        if ui.button("Next >>").clicked()
                            && self.selected_index + 1 < self.batch.len()
                        {
                            self.selected_index += 1;
                            self.preview_dirty = true;
                        }
                    });

                    ui.separator();
                    ui.heading("7-Layer Visibility");
                    ui.label(RichText::new("Toggle individual procedural layers:").small());

                    let mut vis_changed = false;
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer1_color_override,
                            "Layer 1: Color System (OKLCH)",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer2_background,
                            "Layer 2: Background Field",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer3_cellular,
                            "Layer 3: Cellular Math Substrate",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer4_primary_geometry,
                            "Layer 4: Primary Geometry",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer5_co_occurrence,
                            "Layer 5: Co-occurrence Geometry",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer6_proximity,
                            "Layer 6: Proximity Fields & Laser Bridges",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer7_glitch,
                            "Layer 7: Glitch & Scanline Effects",
                        )
                        .changed();

                    ui.horizontal(|ui| {
                        if ui.button("Enable All Layers").clicked() {
                            self.layer_vis = LayerVisibility::default();
                            vis_changed = true;
                        }
                    });

                    if vis_changed {
                        self.grid_textures.clear();
                        self.preview_dirty = true;
                    }

                    ui.separator();
                    ui.heading("Procedural Parameters");

                    if let Some(genome) = self.batch.get_mut(self.selected_index) {
                        let mut changed = false;

                        egui::ComboBox::from_label("L1 Palette")
                            .selected_text(genome.palette.name())
                            .show_ui(ui, |ui| {
                                for &pal in ColorPaletteTrait::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.palette, pal, pal.name())
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L1 Harmony")
                            .selected_text(genome.harmony.name())
                            .show_ui(ui, |ui| {
                                for &h in HarmonyMode::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.harmony, h, h.name())
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.hue_jitter, -24.0..=24.0)
                                    .text("Hue Jitter (deg)"),
                            )
                            .changed();

                        egui::ComboBox::from_label("L2 Background")
                            .selected_text(genome.background.name())
                            .show_ui(ui, |ui| {
                                for &bg in BackgroundStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.background, bg, bg.name())
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L3 Cellular Math")
                            .selected_text(genome.cellular.name())
                            .show_ui(ui, |ui| {
                                for &cell in CellularStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.cellular, cell, cell.name())
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L3 Seed Lattice")
                            .selected_text(genome.seed_distribution.name())
                            .show_ui(ui, |ui| {
                                for &dist in SeedDistribution::ALL {
                                    changed |= ui
                                        .selectable_value(
                                            &mut genome.seed_distribution,
                                            dist,
                                            dist.name(),
                                        )
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.cell_count, 8..=96)
                                    .text("Cell Density"),
                            )
                            .changed();

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.lloyd_relaxation_steps, 0..=5)
                                    .text("Lloyd Relaxation Steps"),
                            )
                            .changed();

                        egui::ComboBox::from_label("L4 Primary Geom")
                            .selected_text(genome.primary_geom.name())
                            .show_ui(ui, |ui| {
                                for &pg in PrimaryGeometry::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.primary_geom, pg, pg.name())
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.geom_density, 0.1..=1.0)
                                    .text("Geometry Density"),
                            )
                            .changed();

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.geom_scale, 0.2..=1.5)
                                    .text("Geometry Scale"),
                            )
                            .changed();

                        egui::ComboBox::from_label("L5 Co-occurrence")
                            .selected_text(genome.secondary_geom.name())
                            .show_ui(ui, |ui| {
                                for &sg in SecondaryGeometry::ALL {
                                    let w = SecondaryGeometry::co_occurrence_weight(
                                        genome.primary_geom,
                                        sg,
                                    );
                                    let label = format!("{} (affinity: {})", sg.name(), w);
                                    changed |= ui
                                        .selectable_value(&mut genome.secondary_geom, sg, label)
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L6 Proximity")
                            .selected_text(genome.proximity.name())
                            .show_ui(ui, |ui| {
                                for &pr in ProximityStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.proximity, pr, pr.name())
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.proximity_threshold, 0.10..=0.45)
                                    .text("Proximity Radius"),
                            )
                            .changed();

                        egui::ComboBox::from_label("L7 Glitch")
                            .selected_text(genome.glitch.name())
                            .show_ui(ui, |ui| {
                                for &gl in GlitchStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.glitch, gl, gl.name())
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.glitch_intensity, 0.0..=1.0)
                                    .text("Glitch Intensity"),
                            )
                            .changed();

                        if changed {
                            genome.co_occurrence_affinity =
                                SecondaryGeometry::co_occurrence_weight(
                                    genome.primary_geom,
                                    genome.secondary_geom,
                                );
                            let id = genome.id;
                            self.grid_textures.remove(&id);
                            self.preview_dirty = true;
                        }
                    }

                    ui.separator();
                    ui.collapsing("📐 Geometry Co-occurrence Matrix", |ui| {
                        ui.label(
                            RichText::new("Affinity weights between Layer 4 and Layer 5:")
                                .small(),
                        );
                        egui::Grid::new("co_occurrence_matrix_grid")
                            .striped(true)
                            .spacing([6.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(RichText::new("Primary \\ Sec").small().strong());
                                for &sec in SecondaryGeometry::ALL {
                                    ui.label(RichText::new(&sec.name()[0..3]).small().strong());
                                }
                                ui.end_row();

                                for &prim in PrimaryGeometry::ALL {
                                    ui.label(RichText::new(prim.name()).small());
                                    for &sec in SecondaryGeometry::ALL {
                                        let w = SecondaryGeometry::co_occurrence_weight(prim, sec);
                                        let color = if w == 0 {
                                            Color32::from_rgb(200, 75, 75)
                                        } else if w >= 30 {
                                            Color32::from_rgb(46, 204, 113)
                                        } else {
                                            Color32::from_rgb(180, 190, 205)
                                        };
                                        ui.colored_label(color, format!("{}", w));
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                });
            });
    }

    fn render_right_inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_inspector")
            .resizable(true)
            .default_width(310.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(genome) = self.batch.get(self.selected_index).cloned() {
                        ui.heading(format!("Background #{}", genome.id));
                        ui.label(format!("Seed: {:#018X}", genome.seed));
                        ui.separator();

                        ui.heading("Aspect Ratio & Resolution");
                        let mut preset_changed = false;
                        egui::ComboBox::from_label("Aspect Preset")
                            .selected_text(self.canvas_preset.name())
                            .show_ui(ui, |ui| {
                                for &preset in AspectRatioPreset::ALL {
                                    if ui
                                        .selectable_label(self.canvas_preset == preset, preset.name())
                                        .clicked()
                                    {
                                        self.apply_preset(preset);
                                        preset_changed = true;
                                    }
                                }
                            });

                        let mut dim_changed = false;
                        ui.horizontal(|ui| {
                            ui.label("Width (px):");
                            dim_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut self.canvas_width)
                                        .range(320..=8192)
                                        .speed(10.0),
                                )
                                .changed();
                        });

                        ui.horizontal(|ui| {
                            ui.label("Height (px):");
                            dim_changed |= ui
                                .add(
                                    egui::DragValue::new(&mut self.canvas_height)
                                        .range(320..=8192)
                                        .speed(10.0),
                                )
                                .changed();
                        });

                        if dim_changed && !preset_changed {
                            self.canvas_preset = AspectRatioPreset::from_dimensions(
                                self.canvas_width,
                                self.canvas_height,
                            );
                            self.export_config.width = self.canvas_width;
                            self.export_config.height = self.canvas_height;
                            self.export_config.preset = self.canvas_preset;
                            self.grid_textures.clear();
                            self.preview_dirty = true;
                        }

                        let ar = self.canvas_width as f32 / self.canvas_height as f32;
                        ui.label(RichText::new(format!("Ratio: {:.3}:1 ({}×{})", ar, self.canvas_width, self.canvas_height)).small());

                        ui.add_space(8.0);
                        ui.separator();
                        ui.heading("Quick Single Export");
                        if ui
                            .button(
                                RichText::new(format!(
                                    "Export This Background ({}×{} PNG)",
                                    self.canvas_width, self.canvas_height
                                ))
                                .strong(),
                            )
                            .clicked()
                        {
                            match export_single_background(&genome, &self.export_config, self.layer_vis) {
                                Ok(img_p) => {
                                    self.status_message = format!("Exported {}", img_p.display());
                                }
                                Err(e) => {
                                    self.status_message = format!("Export error: {}", e);
                                }
                            }
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.heading("OKLCH Color Swatches");
                        let pal = genome.palette.resolve(genome.harmony, genome.hue_jitter);
                        ui.horizontal(|ui| {
                            for c in [
                                pal.bg_primary,
                                pal.cell_tones[0],
                                pal.cell_tones[2],
                                pal.primary_geom,
                                pal.secondary_geom,
                                pal.proximity_glow,
                                pal.highlight,
                            ] {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(32.0, 24.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    Color32::from_rgb(c[0], c[1], c[2]),
                                );
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.heading("Layer Properties");
                        egui::Grid::new("layer_summary_grid")
                            .striped(true)
                            .num_columns(2)
                            .spacing([10.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Palette:");
                                ui.label(genome.palette.name());
                                ui.end_row();

                                ui.label("Harmony:");
                                ui.label(genome.harmony.name());
                                ui.end_row();

                                ui.label("Background:");
                                ui.label(genome.background.name());
                                ui.end_row();

                                ui.label("Cellular Substrate:");
                                ui.label(genome.cellular.name());
                                ui.end_row();

                                ui.label("Seed Lattice:");
                                ui.label(genome.seed_distribution.name());
                                ui.end_row();

                                ui.label("Cell Count:");
                                ui.label(format!("{}", genome.cell_count));
                                ui.end_row();

                                ui.label("Primary Geometry:");
                                ui.label(genome.primary_geom.name());
                                ui.end_row();

                                ui.label("Secondary Geometry:");
                                ui.label(genome.secondary_geom.name());
                                ui.end_row();

                                ui.label("Proximity Web:");
                                ui.label(genome.proximity.name());
                                ui.end_row();

                                ui.label("Glitch Style:");
                                ui.label(genome.glitch.name());
                                ui.end_row();
                            });
                    }
                });
            });
    }

    fn render_center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            CenterTab::CanvasInspector => {
                ui.vertical_centered(|ui| {
                    if let Some(ref tex) = self.current_texture {
                        let avail = ui.available_size();
                        let aspect = self.canvas_width as f32 / self.canvas_height as f32;

                        let max_w = (avail.x - 16.0).max(100.0);
                        let max_h = (avail.y - 16.0).max(100.0);

                        let (disp_w, disp_h) = if max_w / max_h > aspect {
                            let h = max_h;
                            let w = h * aspect;
                            (w, h)
                        } else {
                            let w = max_w;
                            let h = w / aspect;
                            (w, h)
                        };

                        ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(disp_w, disp_h)));
                    }
                });
            }
            CenterTab::BatchGallery => {
                self.render_batch_gallery_tab(ctx, ui);
            }
            CenterTab::BatchExport => {
                self.render_batch_export_tab(ui);
            }
        });
    }

    fn render_batch_gallery_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let total = self.batch.len();
        let total_pages = ((total + self.grid_page_size - 1) / self.grid_page_size).max(1);
        if self.grid_page >= total_pages {
            self.grid_page = total_pages - 1;
        }

        ui.horizontal(|ui| {
            if ui.button("< Prev Page").clicked() && self.grid_page > 0 {
                self.grid_page -= 1;
            }
            ui.label(format!(
                "Page {} / {} ({} total backgrounds)",
                self.grid_page + 1,
                total_pages,
                total
            ));
            if ui.button("Next Page >").clicked() && self.grid_page + 1 < total_pages {
                self.grid_page += 1;
            }
        });
        ui.separator();

        let start = self.grid_page * self.grid_page_size;
        let end = (start + self.grid_page_size).min(total);
        let page_ids: Vec<u32> = (start..end)
            .filter_map(|i| self.batch.get(i).map(|g| g.id))
            .collect();

        self.ensure_grid_page_textures(ctx, &page_ids);

        let aspect = self.canvas_width as f32 / self.canvas_height as f32;
        let thumb_btn_size = if aspect >= 1.0 {
            egui::vec2(160.0, (160.0 / aspect).max(50.0))
        } else {
            egui::vec2((160.0 * aspect).max(50.0), 160.0)
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("background_batch_grid")
                .spacing([16.0, 16.0])
                .show(ui, |ui| {
                    for (idx, &id) in page_ids.iter().enumerate() {
                        let g = &self.batch[(id - 1) as usize];
                        ui.vertical(|ui| {
                            if let Some(tex) = self.grid_textures.get(&id) {
                                let resp = ui.add(
                                    egui::ImageButton::new(
                                        egui::Image::new(tex).fit_to_exact_size(thumb_btn_size),
                                    ),
                                );
                                if resp.clicked() {
                                    self.selected_index = (id - 1) as usize;
                                    self.preview_dirty = true;
                                    self.active_tab = CenterTab::CanvasInspector;
                                }
                            }
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{}", id)).strong());
                                ui.label(RichText::new(g.palette.name()).small());
                            });
                        });

                        if (idx + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_batch_export_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Batch Export Studio");
            ui.label("Render and save multiple high-resolution backgrounds simultaneously.");
            ui.add_space(8.0);

            egui::Grid::new("export_settings_grid")
                .num_columns(2)
                .spacing([14.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Batch Count:");
                    if ui
                        .add(egui::Slider::new(&mut self.batch_count, 1..=500).text("backgrounds"))
                        .changed()
                    {
                        self.regenerate_all();
                    }
                    ui.end_row();

                    ui.label("Output Directory:");
                    ui.text_edit_singleline(&mut self.export_config.output_dir);
                    ui.end_row();

                    ui.label("Filename Prefix:");
                    ui.text_edit_singleline(&mut self.export_config.filename_prefix);
                    ui.end_row();

                    ui.label("Export Resolution:");
                    ui.label(format!("{} × {} px ({})", self.export_config.width, self.export_config.height, self.export_config.preset.name()));
                    ui.end_row();
                });

            ui.add_space(14.0);
            ui.separator();

            let exporting = self.is_exporting.load(Ordering::Relaxed);
            let done_count = self.export_progress.load(Ordering::Relaxed);

            if exporting {
                let frac = done_count as f32 / self.batch_count as f32;
                ui.add(
                    egui::ProgressBar::new(frac)
                        .text(format!("Exporting {} / {} backgrounds...", done_count, self.batch_count))
                        .animate(true),
                );
            } else if ui
                .button(
                    RichText::new(format!(
                        "🚀 Start Batch Export ({} backgrounds at {}×{} PNG)",
                        self.batch_count,
                        self.export_config.width,
                        self.export_config.height
                    ))
                    .strong(),
                )
                .clicked()
            {
                let batch_clone = self.batch.clone();
                let cfg_clone = self.export_config.clone();
                let is_exp = Arc::clone(&self.is_exporting);
                let prog = Arc::clone(&self.export_progress);
                let count = self.batch_count;

                is_exp.store(true, Ordering::Relaxed);
                prog.store(0, Ordering::Relaxed);
                self.status_message = format!(
                    "Rendering {} backgrounds across CPU threads...",
                    count
                );

                thread::spawn(move || {
                    let _ = export_batch_backgrounds(&batch_clone, &cfg_clone, prog);
                    is_exp.store(false, Ordering::Relaxed);
                });
            }

            if !exporting && done_count == self.batch_count as usize {
                ui.colored_label(
                    Color32::from_rgb(46, 204, 113),
                    format!(
                        "✓ Successfully exported all {} backgrounds to {}/{}_*.png!",
                        self.batch_count, self.export_config.output_dir, self.export_config.filename_prefix
                    ),
                );
            }
        });
    }
}
