use crate::export::{
    build_erc721_metadata, export_batch_collection, export_single_nft,
    stamp_octopin_cid_on_metadata, CollectionConfig,
};
use crate::genome::{
    generate_collection, BackgroundStyle, CellularStyle, ColorPaletteTrait, GlitchStyle,
    HarmonyMode, NftGenome, PrimaryGeometry, ProximityStyle, RarityTier, SecondaryGeometry,
    SeedDistribution,
};
use crate::renderer::{render_nft, LayerVisibility};
use eframe::egui::{self, Color32, ColorImage, RichText, TextureHandle, TextureOptions};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CenterTab {
    SingleInspector,
    CollectionGrid,
    RarityAndCoOccurrence,
    OctopinZoraPublish,
}

pub struct NeoArtApp {
    pub master_seed: u64,
    pub supply: u32,
    pub collection: Vec<NftGenome>,
    pub selected_index: usize,
    pub layer_vis: LayerVisibility,
    pub preview_res: u32,
    pub active_tab: CenterTab,

    // Texture cache for single inspector
    pub current_texture: Option<TextureHandle>,
    pub preview_dirty: bool,

    // Thumbnail cache for the 250 grid browser
    pub grid_textures: HashMap<u32, TextureHandle>,
    pub grid_page: usize,
    pub grid_page_size: usize,
    pub rarity_filter: Option<RarityTier>,

    // Export & Octopin/Zora state
    pub export_config: CollectionConfig,
    pub is_exporting: Arc<AtomicBool>,
    pub export_progress: Arc<AtomicUsize>,
    pub status_message: String,
}

impl NeoArtApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let master_seed = 2026_11_21;
        let supply = 250;
        let collection = generate_collection(master_seed, supply);

        Self {
            master_seed,
            supply,
            collection,
            selected_index: 0,
            layer_vis: LayerVisibility::default(),
            preview_res: 512,
            active_tab: CenterTab::SingleInspector,
            current_texture: None,
            preview_dirty: true,
            grid_textures: HashMap::new(),
            grid_page: 0,
            grid_page_size: 25,
            rarity_filter: None,
            export_config: CollectionConfig::default(),
            is_exporting: Arc::new(AtomicBool::new(false)),
            export_progress: Arc::new(AtomicUsize::new(0)),
            status_message: "Ready — 250 unique 7-layer procedural NFTs initialized.".to_string(),
        }
    }

    fn regenerate_all(&mut self) {
        self.collection = generate_collection(self.master_seed, self.supply);
        self.selected_index = 0;
        self.grid_textures.clear();
        self.preview_dirty = true;
        self.status_message = format!(
            "Regenerated {} unique NFTs from seed {}.",
            self.supply, self.master_seed
        );
    }

    fn refresh_current_preview(&mut self, ctx: &egui::Context) {
        if let Some(genome) = self.collection.get(self.selected_index) {
            let size = self.preview_res;
            let rgba = render_nft(genome, size, self.layer_vis);
            let color_image = ColorImage::from_rgba_unmultiplied([size as usize, size as usize], &rgba);
            self.current_texture = Some(ctx.load_texture(
                format!("nft_preview_{}", genome.token_id),
                color_image,
                TextureOptions::LINEAR,
            ));
            self.preview_dirty = false;
        }
    }

    fn ensure_grid_page_textures(&mut self, ctx: &egui::Context, visible_ids: &[u32]) {
        let missing: Vec<NftGenome> = visible_ids
            .iter()
            .filter(|id| !self.grid_textures.contains_key(id))
            .filter_map(|&id| self.collection.get((id - 1) as usize).cloned())
            .collect();

        if missing.is_empty() {
            return;
        }

        let thumb_size = 160u32;
        let vis = self.layer_vis;
        let rendered: Vec<(u32, Vec<u8>)> = missing
            .par_iter()
            .map(|g| (g.token_id, render_nft(g, thumb_size, vis)))
            .collect();

        for (token_id, rgba) in rendered {
            let img = ColorImage::from_rgba_unmultiplied(
                [thumb_size as usize, thumb_size as usize],
                &rgba,
            );
            let tex = ctx.load_texture(
                format!("thumb_{}", token_id),
                img,
                TextureOptions::LINEAR,
            );
            self.grid_textures.insert(token_id, tex);
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
        self.render_right_metadata_panel(ctx);
        self.render_center_panel(ctx);
    }
}

impl NeoArtApp {
    fn render_top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_nav").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("NEO-ART // 7-LAYER NFT STUDIO").strong());
                ui.separator();

                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::SingleInspector,
                    "1. Live NFT Canvas",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::CollectionGrid,
                    "2. 250 Collection Grid",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::RarityAndCoOccurrence,
                    "3. Rarity & Co-occurrence Matrix",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    CenterTab::OctopinZoraPublish,
                    "4. Octopin & Zora Export",
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
            .default_width(320.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Collection Seed & Token");
                    ui.horizontal(|ui| {
                        ui.label("Master Seed:");
                        if ui
                            .add(egui::DragValue::new(&mut self.master_seed).speed(1.0))
                            .changed()
                        {
                            self.regenerate_all();
                        }
                        if ui.button("Randomize Seed").clicked() {
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
                        let mut token_num = (self.selected_index + 1) as u32;
                        if ui
                            .add(
                                egui::Slider::new(&mut token_num, 1..=self.supply)
                                    .text("Token #"),
                            )
                            .changed()
                        {
                            self.selected_index = (token_num - 1) as usize;
                            self.preview_dirty = true;
                        }
                        if ui.button("Next >>").clicked()
                            && self.selected_index + 1 < self.collection.len()
                        {
                            self.selected_index += 1;
                            self.preview_dirty = true;
                        }
                    });

                    ui.separator();
                    ui.heading("7-Layer Visibility Pipeline");
                    ui.label(
                        RichText::new("Toggle individual layers to inspect their contribution:")
                            .small(),
                    );

                    let mut vis_changed = false;
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer1_color_override,
                            "Layer 1: Color System (OKLCH Harmonies)",
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
                            "Layer 3: Cellular Math (Voronoi/Delaunay)",
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
                            "Layer 5: Co-occurrence Secondary Geometry",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer6_proximity,
                            "Layer 6: Proximity Connections & Fields",
                        )
                        .changed();
                    vis_changed |= ui
                        .checkbox(
                            &mut self.layer_vis.layer7_glitch,
                            "Layer 7: Glitch (Pixel Sort & Scanlines)",
                        )
                        .changed();

                    ui.horizontal(|ui| {
                        if ui.button("Enable All 7 Layers").clicked() {
                            self.layer_vis = LayerVisibility::default();
                            vis_changed = true;
                        }
                        if ui
                            .selectable_label(self.preview_res == 512, "512px Fast")
                            .clicked()
                        {
                            self.preview_res = 512;
                            vis_changed = true;
                        }
                        if ui
                            .selectable_label(self.preview_res == 1024, "1024px HD")
                            .clicked()
                        {
                            self.preview_res = 1024;
                            vis_changed = true;
                        }
                    });

                    if vis_changed {
                        self.preview_dirty = true;
                        self.grid_textures.clear();
                    }

                    ui.separator();
                    ui.heading("Live Trait & Math Editor");
                    if let Some(genome) = self.collection.get_mut(self.selected_index) {
                        let mut changed = false;

                        egui::ComboBox::from_label("L1 Palette")
                            .selected_text(genome.palette.name())
                            .show_ui(ui, |ui| {
                                for &p in ColorPaletteTrait::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.palette, p, p.name())
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

                        egui::ComboBox::from_label("L2 Background")
                            .selected_text(genome.background.name())
                            .show_ui(ui, |ui| {
                                for &b in BackgroundStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.background, b, b.name())
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L3 Cellular")
                            .selected_text(genome.cellular.name())
                            .show_ui(ui, |ui| {
                                for &c in CellularStyle::ALL {
                                    changed |= ui
                                        .selectable_value(&mut genome.cellular, c, c.name())
                                        .changed();
                                }
                            });

                        egui::ComboBox::from_label("L3 Seed Lattice")
                            .selected_text(genome.seed_distribution.name())
                            .show_ui(ui, |ui| {
                                for &d in SeedDistribution::ALL {
                                    changed |= ui
                                        .selectable_value(
                                            &mut genome.seed_distribution,
                                            d,
                                            d.name(),
                                        )
                                        .changed();
                                }
                            });

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.cell_count, 10..=80)
                                    .text("Voronoi Cells"),
                            )
                            .changed();

                        changed |= ui
                            .add(
                                egui::Slider::new(&mut genome.lloyd_relaxation_steps, 0..=5)
                                    .text("Lloyd Relaxation"),
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
                                egui::Slider::new(&mut genome.geom_scale, 0.35..=1.15)
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
                            genome.evaluate_rarity();
                            let tid = genome.token_id;
                            self.grid_textures.remove(&tid);
                            self.preview_dirty = true;
                        }
                    }
                });
            });
    }

    fn render_right_metadata_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("right_metadata")
            .resizable(true)
            .default_width(310.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if let Some(genome) = self.collection.get(self.selected_index).cloned() {
                        let [r, g, b] = genome.rarity_tier.badge_color();
                        ui.horizontal(|ui| {
                            ui.heading(format!("Token #{}", genome.token_id));
                            ui.colored_label(
                                Color32::from_rgb(r, g, b),
                                RichText::new(format!("[{}]", genome.rarity_tier.name())).strong(),
                            );
                        });

                        if let Some(ref title) = genome.mythic_title {
                            ui.colored_label(
                                Color32::from_rgb(255, 105, 180),
                                RichText::new(format!("1-of-1 Mythic: {}", title)).strong(),
                            );
                        }

                        ui.separator();
                        ui.label(RichText::new("Resolved Color Swatches (OKLCH)").strong());
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
                                    egui::vec2(26.0, 22.0),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    4.0,
                                    Color32::from_rgb(c[0], c[1], c[2]),
                                );
                            }
                        });

                        ui.separator();
                        ui.label(RichText::new("Quick Single Export (2000x2000 PNG + JSON)").strong());
                        if ui.button("Export This NFT to disk").clicked() {
                            match export_single_nft(&genome, &self.export_config, self.layer_vis) {
                                Ok((img_p, meta_p)) => {
                                    self.status_message = format!(
                                        "Exported {} and {}",
                                        img_p.display(),
                                        meta_p.display()
                                    );
                                }
                                Err(e) => {
                                    self.status_message = format!("Export error: {}", e);
                                }
                            }
                        }

                        ui.separator();
                        ui.label(RichText::new("Live ERC-721 Metadata JSON (Octopin / Zora)").strong());
                        let meta = build_erc721_metadata(&genome, &self.export_config);
                        if let Ok(pretty_json) = serde_json::to_string_pretty(&meta) {
                            ui.code(pretty_json);
                        }
                    }
                });
            });
    }

    fn render_center_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| match self.active_tab {
            CenterTab::SingleInspector => {
                ui.vertical_centered(|ui| {
                    if let Some(ref tex) = self.current_texture {
                        let avail = ui.available_size();
                        let side = avail.x.min(avail.y - 16.0).clamp(256.0, 820.0);
                        ui.add(egui::Image::new(tex).fit_to_exact_size(egui::vec2(side, side)));
                    }
                });
            }
            CenterTab::CollectionGrid => {
                self.render_collection_grid_tab(ctx, ui);
            }
            CenterTab::RarityAndCoOccurrence => {
                self.render_rarity_matrix_tab(ui);
            }
            CenterTab::OctopinZoraPublish => {
                self.render_publish_tab(ui);
            }
        });
    }

    fn render_collection_grid_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter by Rarity:");
            if ui
                .selectable_label(self.rarity_filter.is_none(), "All (250)")
                .clicked()
            {
                self.rarity_filter = None;
                self.grid_page = 0;
            }
            for tier in [
                RarityTier::Common,
                RarityTier::Uncommon,
                RarityTier::Rare,
                RarityTier::Legendary,
                RarityTier::MythicOneOfOne,
            ] {
                if ui
                    .selectable_label(self.rarity_filter == Some(tier), tier.name())
                    .clicked()
                {
                    self.rarity_filter = Some(tier);
                    self.grid_page = 0;
                }
            }
        });

        let filtered_ids: Vec<u32> = self
            .collection
            .iter()
            .filter(|g| self.rarity_filter.map_or(true, |r| g.rarity_tier == r))
            .map(|g| g.token_id)
            .collect();

        let total_pages = ((filtered_ids.len() + self.grid_page_size - 1) / self.grid_page_size).max(1);
        if self.grid_page >= total_pages {
            self.grid_page = total_pages - 1;
        }

        ui.horizontal(|ui| {
            if ui.button("< Prev Page").clicked() && self.grid_page > 0 {
                self.grid_page -= 1;
            }
            ui.label(format!(
                "Page {} / {} ({} matching NFTs)",
                self.grid_page + 1,
                total_pages,
                filtered_ids.len()
            ));
            if ui.button("Next Page >").clicked() && self.grid_page + 1 < total_pages {
                self.grid_page += 1;
            }
        });
        ui.separator();

        let start = self.grid_page * self.grid_page_size;
        let end = (start + self.grid_page_size).min(filtered_ids.len());
        let page_ids = filtered_ids[start..end].to_vec();

        self.ensure_grid_page_textures(ctx, &page_ids);

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("nft_collection_grid")
                .spacing([14.0, 14.0])
                .show(ui, |ui| {
                    for (idx, &token_id) in page_ids.iter().enumerate() {
                        let g = &self.collection[(token_id - 1) as usize];
                        let [r, gr, b] = g.rarity_tier.badge_color();
                        ui.vertical(|ui| {
                            if let Some(tex) = self.grid_textures.get(&token_id) {
                                let resp = ui.add(
                                    egui::ImageButton::new(
                                        egui::Image::new(tex)
                                            .fit_to_exact_size(egui::vec2(135.0, 135.0)),
                                    ),
                                );
                                if resp.clicked() {
                                    self.selected_index = (token_id - 1) as usize;
                                    self.preview_dirty = true;
                                    self.active_tab = CenterTab::SingleInspector;
                                }
                            }
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("#{}", token_id)).strong());
                                ui.colored_label(
                                    Color32::from_rgb(r, gr, b),
                                    g.rarity_tier.name(),
                                );
                            });
                            ui.label(RichText::new(g.palette.name()).small());
                        });

                        if (idx + 1) % 5 == 0 {
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn render_rarity_matrix_tab(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Collection Rarity Distribution (250 NFTs)");
            ui.add_space(6.0);

            egui::Grid::new("rarity_tier_counts")
                .striped(true)
                .min_col_width(140.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("Rarity Tier").strong());
                    ui.label(RichText::new("Count").strong());
                    ui.label(RichText::new("Share (%)").strong());
                    ui.end_row();

                    for tier in [
                        RarityTier::Common,
                        RarityTier::Uncommon,
                        RarityTier::Rare,
                        RarityTier::Legendary,
                        RarityTier::MythicOneOfOne,
                    ] {
                        let count = self
                            .collection
                            .iter()
                            .filter(|g| g.rarity_tier == tier)
                            .count();
                        let [r, g, b] = tier.badge_color();
                        ui.colored_label(Color32::from_rgb(r, g, b), tier.name());
                        ui.label(format!("{}", count));
                        ui.label(format!("{:.1}%", (count as f32 / self.supply as f32) * 100.0));
                        ui.end_row();
                    }
                });

            ui.add_space(16.0);
            ui.separator();
            ui.heading("Layer 4 x Layer 5: Geometry Co-occurrence Matrix");
            ui.label(
                "Shows the conditional affinity weight (and actual observed count in the 250 collection) between Primary Geometry (rows) and Co-occurrence Secondary Geometry (columns). A 0 weight indicates strict geometric repulsion.",
            );
            ui.add_space(8.0);

            egui::Grid::new("co_occurrence_grid")
                .striped(true)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Primary \\ Secondary").strong());
                    for &sec in SecondaryGeometry::ALL {
                        ui.label(RichText::new(sec.name()).small().strong());
                    }
                    ui.end_row();

                    for &prim in PrimaryGeometry::ALL {
                        ui.label(RichText::new(prim.name()).strong());
                        for &sec in SecondaryGeometry::ALL {
                            let w = SecondaryGeometry::co_occurrence_weight(prim, sec);
                            let observed = self
                                .collection
                                .iter()
                                .filter(|g| g.primary_geom == prim && g.secondary_geom == sec)
                                .count();
                            let color = if w == 0 {
                                Color32::from_rgb(200, 75, 75)
                            } else if w >= 30 {
                                Color32::from_rgb(46, 204, 113)
                            } else {
                                Color32::from_rgb(180, 190, 205)
                            };
                            ui.colored_label(color, format!("w:{} ({}x)", w, observed));
                        }
                        ui.end_row();
                    }
                });
        });
    }

    fn render_publish_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Batch Export & Octopin / Zora Publishing Pipeline");
            ui.add_space(8.0);

            egui::Grid::new("export_settings_grid")
                .num_columns(2)
                .spacing([12.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Collection Name:");
                    ui.text_edit_singleline(&mut self.export_config.collection_name);
                    ui.end_row();

                    ui.label("Collection Description:");
                    ui.text_edit_singleline(&mut self.export_config.description);
                    ui.end_row();

                    ui.label("External / Project URL:");
                    ui.text_edit_singleline(&mut self.export_config.external_url);
                    ui.end_row();

                    ui.label("Export Output Folder:");
                    ui.text_edit_singleline(&mut self.export_config.output_dir);
                    ui.end_row();

                    ui.label("PNG Resolution (px):");
                    ui.add(
                        egui::DragValue::new(&mut self.export_config.export_resolution)
                            .range(512..=4096),
                    );
                    ui.end_row();

                    ui.label("Octopin Images Folder CID:");
                    ui.text_edit_singleline(&mut self.export_config.images_cid);
                    ui.end_row();
                });

            ui.add_space(12.0);
            ui.separator();

            ui.heading("Step 1: Render All 250 High-Res PNGs & Metadata");
            let exporting = self.is_exporting.load(Ordering::Relaxed);
            let done_count = self.export_progress.load(Ordering::Relaxed);

            if exporting {
                let frac = done_count as f32 / self.supply as f32;
                ui.add(
                    egui::ProgressBar::new(frac)
                        .text(format!("Rendering {} / {} NFTs...", done_count, self.supply))
                        .animate(true),
                );
            } else if ui
                .button(
                    RichText::new(format!(
                        "Batch Export All {} NFTs ({}x{} PNG + JSON)",
                        self.supply,
                        self.export_config.export_resolution,
                        self.export_config.export_resolution
                    ))
                    .strong(),
                )
                .clicked()
            {
                let col_clone = self.collection.clone();
                let cfg_clone = self.export_config.clone();
                let is_exp = Arc::clone(&self.is_exporting);
                let prog = Arc::clone(&self.export_progress);

                is_exp.store(true, Ordering::Relaxed);
                prog.store(0, Ordering::Relaxed);
                self.status_message = "Batch rendering 250 NFTs across all CPU cores...".to_string();

                thread::spawn(move || {
                    let _ = export_batch_collection(&col_clone, &cfg_clone, prog);
                    is_exp.store(false, Ordering::Relaxed);
                });
            }

            if !exporting && done_count == self.supply as usize {
                ui.colored_label(
                    Color32::from_rgb(46, 204, 113),
                    format!(
                        "Completed batch export of {} PNGs to {}/images/ and JSONs to {}/metadata/!",
                        self.supply, self.export_config.output_dir, self.export_config.output_dir
                    ),
                );
            }

            ui.add_space(14.0);
            ui.separator();
            ui.heading("Step 2: Pin Images to Octopin & Stamp IPFS CID into Metadata");
            ui.label("1. Upload `output/images/` folder to https://octopin.app/ and copy the resulting root Images CID (e.g. `bafybeig...`).");
            ui.label("2. Paste that CID into 'Octopin Images Folder CID' above and click the button below to stamp `ipfs://<CID>/<id>.png` into all 250 JSON files:");

            if ui
                .button(RichText::new("Stamp Octopin Images CID into all 250 Metadata JSONs").strong())
                .clicked()
            {
                match stamp_octopin_cid_on_metadata(&self.collection, &self.export_config) {
                    Ok(n) => {
                        self.status_message = format!(
                            "Stamped ipfs://{}/<id>.png into {} JSON files in {}/metadata/",
                            self.export_config.images_cid, n, self.export_config.output_dir
                        );
                    }
                    Err(e) => {
                        self.status_message = format!("CID stamp error: {}", e);
                    }
                }
            }

            ui.add_space(14.0);
            ui.separator();
            ui.heading("Step 3: Pin Metadata to Octopin & Publish on Zora");
            ui.label("3. Upload the updated `output/metadata/` folder to https://octopin.app/ to receive your Metadata Base CID (`<METADATA_CID>`).");
            ui.label("4. On https://zora.co/ create your ERC-721 Edition / Drop and set the Metadata Base URI to `ipfs://<METADATA_CID>/`.");
        });
    }
}
