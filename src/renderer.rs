use crate::cellular::{CellularSubstrate, Point2D, VoronoiCell};
use crate::genome::{
    BackgroundGenome, BackgroundStyle, CellularStyle, PrimaryGeometry, ProximityStyle,
    ResolvedPalette, SecondaryGeometry,
};
use crate::glitch::apply_glitch_layer;
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::f32::consts::{PI, TAU};
use tiny_skia::{
    Color, FillRule, GradientStop, LineCap, LineJoin, LinearGradient, Paint, PathBuilder, Pixmap,
    Point, RadialGradient, Rect, SpreadMode, Stroke, Transform,
};

#[derive(Debug, Clone, Copy)]
pub struct LayerVisibility {
    pub layer1_color_override: bool, // Always active; when toggled false uses monochrome debug gray
    pub layer2_background: bool,
    pub layer3_cellular: bool,
    pub layer4_primary_geometry: bool,
    pub layer5_co_occurrence: bool,
    pub layer6_proximity: bool,
    pub layer7_glitch: bool,
}

impl Default for LayerVisibility {
    fn default() -> Self {
        Self {
            layer1_color_override: true,
            layer2_background: true,
            layer3_cellular: true,
            layer4_primary_geometry: true,
            layer5_co_occurrence: true,
            layer6_proximity: true,
            layer7_glitch: true,
        }
    }
}

#[derive(Debug, Clone)]
struct GeometryAnchor {
    pub center: Point2D,
    pub radius: f32,
    pub cell_index: usize,
    pub rotation: f32,
    pub color: [u8; 4],
}

fn skia_color(rgba: [u8; 4], alpha_scale: f32) -> Color {
    let a = ((rgba[3] as f32) * alpha_scale.clamp(0.0, 1.0)).round() as u8;
    Color::from_rgba8(rgba[0], rgba[1], rgba[2], a)
}

fn grayscale_palette() -> ResolvedPalette {
    ResolvedPalette {
        bg_primary: [24, 26, 30, 255],
        bg_secondary: [42, 46, 54, 255],
        cell_tones: [
            [65, 70, 78, 220],
            [95, 102, 112, 220],
            [128, 134, 145, 220],
            [162, 168, 178, 220],
            [196, 202, 210, 220],
        ],
        stroke_color: [220, 224, 232, 210],
        primary_geom: [235, 238, 245, 240],
        secondary_geom: [185, 192, 205, 235],
        proximity_glow: [245, 248, 255, 215],
        highlight: [255, 255, 255, 255],
    }
}

/// Renders a procedural background at arbitrary resolution (`width x height`) into an RGBA8 buffer.
/// Isotropically scales shapes and coordinates to keep geometry undistorted.
pub fn render_background(
    genome: &BackgroundGenome,
    width: u32,
    height: u32,
    vis: LayerVisibility,
) -> Vec<u8> {
    let mut pixmap = Pixmap::new(width, height).expect("Valid canvas dimensions");
    let w = width as f32;
    let h = height as f32;
    let s = w.min(h);
    let aspect_ratio = w / h;

    // Layer 1: Color System Resolution
    let pal = if vis.layer1_color_override {
        genome.palette.resolve(genome.harmony, genome.hue_jitter)
    } else {
        grayscale_palette()
    };

    // Layer 2: Background Field
    if vis.layer2_background {
        render_layer2_background(&mut pixmap, w, h, s, genome, &pal);
    } else {
        pixmap.fill(skia_color(pal.bg_primary, 1.0));
    }

    // Compute Cellular Math Substrate with isotropic aspect ratio
    let substrate = CellularSubstrate::generate(
        genome.seed,
        genome.cell_count,
        genome.seed_distribution,
        genome.lloyd_relaxation_steps,
        aspect_ratio,
    );

    // Layer 3: Cellular Math Substrate
    if vis.layer3_cellular {
        render_layer3_cellular(&mut pixmap, s, genome, &pal, &substrate);
    }

    // Select active geometry anchors from Voronoi cells
    let anchors = select_geometry_anchors(genome, &pal, &substrate);

    // Layer 6: Proximity Connections & Fields (drawn beneath/around primary geometry for visual depth)
    if vis.layer6_proximity {
        render_layer6_proximity(&mut pixmap, s, genome, &pal, &substrate, &anchors);
    }

    // Layer 4: Primary Geometry
    if vis.layer4_primary_geometry {
        render_layer4_primary_geometry(&mut pixmap, s, genome, &pal, &substrate, &anchors);
    }

    // Layer 5: Co-occurrence Secondary Geometry
    if vis.layer5_co_occurrence {
        render_layer5_co_occurrence(&mut pixmap, s, genome, &pal, &substrate, &anchors);
    }

    // Framing subtle border
    render_subtle_frame(&mut pixmap, w, h, &pal);

    // Convert premultiplied tiny-skia pixels to straight RGBA8
    let mut out = vec![0u8; (width * height * 4) as usize];
    for (i, px) in pixmap.pixels().iter().enumerate() {
        let c = px.demultiply();
        let idx = i * 4;
        out[idx] = c.red();
        out[idx + 1] = c.green();
        out[idx + 2] = c.blue();
        out[idx + 3] = c.alpha();
    }

    // Layer 7: Effects & Glitch (Pixel Sorting & Scanline Displacement)
    if vis.layer7_glitch {
        apply_glitch_layer(
            &mut out,
            width,
            height,
            genome.glitch,
            genome.glitch_intensity,
            genome.seed,
        );
    }

    out
}

fn render_layer2_background(
    pixmap: &mut Pixmap,
    w: f32,
    h: f32,
    s: f32,
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
) {
    pixmap.fill(skia_color(pal.bg_primary, 1.0));

    match genome.background {
        BackgroundStyle::DeepVoid => {
            // Subtle vignette radial gradient spanning canvas diagonal
            let diag = w.hypot(h);
            if let Some(shader) = RadialGradient::new(
                Point::from_xy(w * 0.5, h * 0.5),
                Point::from_xy(w * 0.5, h * 0.5),
                diag * 0.65,
                vec![
                    GradientStop::new(0.0, skia_color(pal.bg_secondary, 0.55)),
                    GradientStop::new(1.0, skia_color(pal.bg_primary, 1.0)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let mut paint = Paint::default();
                paint.shader = shader;
                if let Some(rect) = Rect::from_xywh(0.0, 0.0, w, h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }
        BackgroundStyle::RadialNebulaGradient => {
            let diag = w.hypot(h);
            if let Some(shader) = RadialGradient::new(
                Point::from_xy(w * 0.42, h * 0.38),
                Point::from_xy(w * 0.5, h * 0.5),
                diag * 0.70,
                vec![
                    GradientStop::new(0.0, skia_color(pal.cell_tones[0], 0.45)),
                    GradientStop::new(0.55, skia_color(pal.bg_secondary, 0.85)),
                    GradientStop::new(1.0, skia_color(pal.bg_primary, 1.0)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let mut paint = Paint::default();
                paint.shader = shader;
                if let Some(rect) = Rect::from_xywh(0.0, 0.0, w, h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }
        BackgroundStyle::LinearHorizon => {
            if let Some(shader) = LinearGradient::new(
                Point::from_xy(0.0, 0.0),
                Point::from_xy(0.0, h),
                vec![
                    GradientStop::new(0.0, skia_color(pal.bg_secondary, 1.0)),
                    GradientStop::new(0.55, skia_color(pal.bg_primary, 1.0)),
                    GradientStop::new(1.0, skia_color(pal.cell_tones[1], 0.35)),
                ],
                SpreadMode::Pad,
                Transform::identity(),
            ) {
                let mut paint = Paint::default();
                paint.shader = shader;
                if let Some(rect) = Rect::from_xywh(0.0, 0.0, w, h) {
                    pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                }
            }
        }
        BackgroundStyle::MicroBlueprintGrid => {
            let mut paint = Paint::default();
            paint.set_color(skia_color(pal.stroke_color, 0.14));
            paint.anti_alias = true;

            let mut stroke = Stroke::default();
            stroke.width = (s * 0.0012).max(1.0);

            let grid_size = s / 24.0;
            let mut pb = PathBuilder::new();
            let mut x = grid_size;
            while x < w {
                pb.move_to(x, 0.0);
                pb.line_to(x, h);
                x += grid_size;
            }
            let mut y = grid_size;
            while y < h {
                pb.move_to(0.0, y);
                pb.line_to(w, y);
                y += grid_size;
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        BackgroundStyle::QuantizedNoiseField => {
            let perlin = Perlin::new((genome.seed & 0xFFFF_FFFF) as u32);
            let cell_size = (s / 32.0).max(4.0);
            let cols = (w / cell_size).ceil() as usize;
            let rows = (h / cell_size).ceil() as usize;
            let mut paint = Paint::default();
            for gy in 0..rows {
                for gx in 0..cols {
                    let u = (gx as f64 * cell_size as f64) / w as f64;
                    let v = (gy as f64 * cell_size as f64) / h as f64;
                    let n = (perlin.get([u * 3.8, v * 3.8, 0.5]) + 1.0) * 0.5;
                    let tone_idx = ((n * 4.99) as usize).min(4);
                    paint.set_color(skia_color(pal.cell_tones[tone_idx], 0.15));
                    if let Some(rect) = Rect::from_xywh(
                        gx as f32 * cell_size,
                        gy as f32 * cell_size,
                        cell_size + 0.5,
                        cell_size + 0.5,
                    ) {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
                    }
                }
            }
        }
        BackgroundStyle::DualToneDiagonal => {
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, 0.0);
            pb.line_to(w, 0.0);
            pb.line_to(0.0, h);
            pb.close();
            if let Some(path) = pb.finish() {
                let mut paint = Paint::default();
                paint.set_color(skia_color(pal.bg_secondary, 0.85));
                paint.anti_alias = true;
                pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            }
        }
        BackgroundStyle::ConcentricPulse => {
            let mut paint = Paint::default();
            paint.anti_alias = true;
            let mut stroke = Stroke::default();
            stroke.width = (s * 0.002).max(1.0);
            let diag = w.hypot(h);
            for r_idx in 1..=14 {
                let rad = (r_idx as f32 / 15.0) * diag * 0.55;
                paint.set_color(skia_color(
                    pal.cell_tones[r_idx % 5],
                    0.18 + (r_idx as f32 * 0.01),
                ));
                if let Some(path) = build_circle_path(w * 0.5, h * 0.5, rad) {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
        }
    }
}

fn render_layer3_cellular(
    pixmap: &mut Pixmap,
    s: f32,
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
    substrate: &CellularSubstrate,
) {
    match genome.cellular {
        CellularStyle::VoronoiCrystalCells | CellularStyle::FracturedRelaxedShards => {
            draw_voronoi_cells(
                pixmap,
                s,
                pal,
                &substrate.cells,
                genome.cellular == CellularStyle::FracturedRelaxedShards,
            );
        }
        CellularStyle::DelaunayTriangulationMesh => {
            draw_delaunay_mesh(pixmap, s, pal, substrate, 0.65);
        }
        CellularStyle::DualVoronoiDelaunayLattice => {
            draw_voronoi_cells(pixmap, s, pal, &substrate.cells, false);
            draw_delaunay_mesh(pixmap, s, pal, substrate, 0.45);
        }
        CellularStyle::WorleyDistanceRidges => {
            draw_worley_ridges(pixmap, s, pal, substrate);
            draw_voronoi_cells_wireframe(pixmap, s, pal, &substrate.cells);
        }
    }
}

fn draw_voronoi_cells(
    pixmap: &mut Pixmap,
    s: f32,
    pal: &ResolvedPalette,
    cells: &[VoronoiCell],
    inset_shards: bool,
) {
    let mut fill_paint = Paint::default();
    fill_paint.anti_alias = true;

    let mut stroke_paint = Paint::default();
    stroke_paint.set_color(skia_color(pal.stroke_color, 0.58));
    stroke_paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = (s * 0.0022).max(1.0);
    stroke.line_join = LineJoin::Round;

    for cell in cells {
        if cell.vertices.len() < 3 {
            continue;
        }
        let shrink = if inset_shards { 0.88 } else { 1.0 };
        let mut pb = PathBuilder::new();
        for (k, v) in cell.vertices.iter().enumerate() {
            let vx = cell.centroid.x + (v.x - cell.centroid.x) * shrink;
            let vy = cell.centroid.y + (v.y - cell.centroid.y) * shrink;
            if k == 0 {
                pb.move_to(vx * s, vy * s);
            } else {
                pb.line_to(vx * s, vy * s);
            }
        }
        pb.close();

        if let Some(path) = pb.finish() {
            let tone = pal.cell_tones[(cell.index + cell.neighbors.len()) % 5];
            let alpha = (0.32 + (cell.area * 8.5).clamp(0.0, 0.42)).clamp(0.25, 0.74);
            fill_paint.set_color(skia_color(tone, alpha));
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_voronoi_cells_wireframe(
    pixmap: &mut Pixmap,
    s: f32,
    pal: &ResolvedPalette,
    cells: &[VoronoiCell],
) {
    let mut stroke_paint = Paint::default();
    stroke_paint.set_color(skia_color(pal.stroke_color, 0.45));
    stroke_paint.anti_alias = true;
    let mut stroke = Stroke::default();
    stroke.width = (s * 0.0018).max(1.0);

    for cell in cells {
        if cell.vertices.len() < 3 {
            continue;
        }
        let mut pb = PathBuilder::new();
        for (k, v) in cell.vertices.iter().enumerate() {
            if k == 0 {
                pb.move_to(v.x * s, v.y * s);
            } else {
                pb.line_to(v.x * s, v.y * s);
            }
        }
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
        }
    }
}

fn draw_delaunay_mesh(
    pixmap: &mut Pixmap,
    s: f32,
    pal: &ResolvedPalette,
    substrate: &CellularSubstrate,
    opacity: f32,
) {
    let mut fill_paint = Paint::default();
    fill_paint.anti_alias = true;

    for (idx, &(i, j, k)) in substrate.delaunay_triangles.iter().enumerate() {
        let a = substrate.cells[i].centroid;
        let b = substrate.cells[j].centroid;
        let c = substrate.cells[k].centroid;

        let mut pb = PathBuilder::new();
        pb.move_to(a.x * s, a.y * s);
        pb.line_to(b.x * s, b.y * s);
        pb.line_to(c.x * s, c.y * s);
        pb.close();

        if let Some(path) = pb.finish() {
            let tone = pal.cell_tones[idx % 5];
            fill_paint.set_color(skia_color(tone, opacity * 0.48));
            pixmap.fill_path(
                &path,
                &fill_paint,
                FillRule::Winding,
                Transform::identity(),
                None,
            );
        }
    }

    let mut stroke_paint = Paint::default();
    stroke_paint.set_color(skia_color(pal.highlight, opacity * 0.75));
    stroke_paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = (s * 0.002).max(1.0);

    let mut pb = PathBuilder::new();
    for &(i, j) in &substrate.delaunay_edges {
        let a = substrate.cells[i].centroid;
        let b = substrate.cells[j].centroid;
        pb.move_to(a.x * s, a.y * s);
        pb.line_to(b.x * s, b.y * s);
    }
    if let Some(path) = pb.finish() {
        pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
    }
}

fn draw_worley_ridges(
    pixmap: &mut Pixmap,
    s: f32,
    pal: &ResolvedPalette,
    substrate: &CellularSubstrate,
) {
    let grid_res = 96.0;
    let step = s / grid_res;
    let cols = (substrate.bound_x * grid_res).ceil() as usize;
    let rows = (substrate.bound_y * grid_res).ceil() as usize;
    let mut paint = Paint::default();

    for gy in 0..rows {
        let v = (gy as f32 + 0.5) / grid_res;
        for gx in 0..cols {
            let u = (gx as f32 + 0.5) / grid_res;
            let (d1, d2, idx) = substrate.worley_at(u, v);
            let ridge = (d2 - d1).clamp(0.0, 0.18) / 0.18;
            // Quantize into contour bands
            let band = ((ridge * 6.0).floor() as usize) % 5;
            let tone = pal.cell_tones[(idx + band) % 5];
            let alpha = 0.25 + (1.0 - ridge) * 0.45;
            paint.set_color(skia_color(tone, alpha));
            if let Some(rect) =
                Rect::from_xywh(gx as f32 * step, gy as f32 * step, step + 0.6, step + 0.6)
            {
                pixmap.fill_rect(rect, &paint, Transform::identity(), None);
            }
        }
    }
}

fn select_geometry_anchors(
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
    substrate: &CellularSubstrate,
) -> Vec<GeometryAnchor> {
    let mut rng = ChaCha8Rng::seed_from_u64(genome.seed ^ 0x4444_8888_1234_5678);
    let mut anchors = Vec::new();

    for (i, cell) in substrate.cells.iter().enumerate() {
        if rng.gen::<f32>() <= genome.geom_density {
            let color = match i % 4 {
                0 => pal.primary_geom,
                1 => pal.secondary_geom,
                2 => pal.cell_tones[(i + 2) % 5],
                _ => pal.highlight,
            };
            anchors.push(GeometryAnchor {
                center: cell.centroid,
                radius: (cell.inradius * genome.geom_scale * 1.15).clamp(0.02, 0.16),
                cell_index: i,
                rotation: rng.gen_range(0.0..TAU),
                color,
            });
        }
    }

    // Ensure at least 6 anchors for rich composition
    if anchors.len() < 6 {
        for (i, cell) in substrate.cells.iter().take(8).enumerate() {
            anchors.push(GeometryAnchor {
                center: cell.centroid,
                radius: (cell.inradius * genome.geom_scale).clamp(0.025, 0.14),
                cell_index: i,
                rotation: (i as f32) * 0.7,
                color: pal.primary_geom,
            });
        }
    }

    anchors
}

fn render_layer4_primary_geometry(
    pixmap: &mut Pixmap,
    s: f32,
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
    _substrate: &CellularSubstrate,
    anchors: &[GeometryAnchor],
) {
    let mut fill_paint = Paint::default();
    fill_paint.anti_alias = true;

    let mut stroke_paint = Paint::default();
    stroke_paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.width = (s * 0.0032).max(1.2);
    stroke.line_cap = LineCap::Round;
    stroke.line_join = LineJoin::Round;

    for (idx, a) in anchors.iter().enumerate() {
        let cx = a.center.x * s;
        let cy = a.center.y * s;
        let r = a.radius * s;

        fill_paint.set_color(skia_color(a.color, 0.78));
        stroke_paint.set_color(skia_color(pal.highlight, 0.90));

        match genome.primary_geom {
            PrimaryGeometry::ConcentricRings => {
                if let Some(outer) = build_circle_path(cx, cy, r) {
                    pixmap.fill_path(
                        &outer,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap.stroke_path(
                        &outer,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                if let Some(mid) = build_circle_path(cx, cy, r * 0.62) {
                    fill_paint.set_color(skia_color(pal.bg_primary, 0.85));
                    pixmap.fill_path(
                        &mid,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap.stroke_path(&mid, &stroke_paint, &stroke, Transform::identity(), None);
                }
                if let Some(core) = build_circle_path(cx, cy, r * 0.28) {
                    fill_paint.set_color(skia_color(pal.secondary_geom, 0.95));
                    pixmap.fill_path(
                        &core,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            PrimaryGeometry::InscribedPolygons => {
                let sides = 5 + (idx % 3); // Pentagon, Hexagon, Heptagon
                if let Some(poly) = build_regular_polygon_path(cx, cy, r, sides, a.rotation) {
                    pixmap.fill_path(
                        &poly,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap.stroke_path(
                        &poly,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                if let Some(inner) =
                    build_regular_polygon_path(cx, cy, r * 0.52, sides, a.rotation + PI / sides as f32)
                {
                    stroke_paint.set_color(skia_color(pal.bg_primary, 0.90));
                    pixmap.stroke_path(
                        &inner,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
            PrimaryGeometry::BauhausRectilinear => {
                if let Some(poly) = build_regular_polygon_path(cx, cy, r, 4, a.rotation) {
                    pixmap.fill_path(
                        &poly,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap.stroke_path(
                        &poly,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                // Offset contrasting bar
                if let Some(bar) =
                    build_regular_polygon_path(cx + r * 0.22, cy - r * 0.22, r * 0.45, 4, a.rotation)
                {
                    fill_paint.set_color(skia_color(pal.secondary_geom, 0.88));
                    pixmap.fill_path(
                        &bar,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            PrimaryGeometry::IsometricPrisms => {
                // Draw 3 isometric cube faces
                let angles = [a.rotation, a.rotation + TAU / 3.0, a.rotation + 2.0 * TAU / 3.0];
                let face_colors = [a.color, pal.secondary_geom, pal.cell_tones[idx % 5]];
                for f in 0..3 {
                    let a1 = angles[f];
                    let a2 = angles[(f + 1) % 3];
                    let mut pb = PathBuilder::new();
                    pb.move_to(cx, cy);
                    pb.line_to(cx + r * a1.cos(), cy + r * a1.sin());
                    pb.line_to(
                        cx + r * (a1.cos() + a2.cos()) * 0.72,
                        cy + r * (a1.sin() + a2.sin()) * 0.72,
                    );
                    pb.line_to(cx + r * a2.cos(), cy + r * a2.sin());
                    pb.close();
                    if let Some(path) = pb.finish() {
                        fill_paint.set_color(skia_color(face_colors[f], 0.85));
                        pixmap.fill_path(
                            &path,
                            &fill_paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                        pixmap.stroke_path(
                            &path,
                            &stroke_paint,
                            &stroke,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            PrimaryGeometry::RadialVectorBursts => {
                let rays = 12;
                let mut pb = PathBuilder::new();
                for k in 0..rays {
                    let theta = a.rotation + (k as f32 / rays as f32) * TAU;
                    let r_in = r * 0.22;
                    let r_out = if k % 2 == 0 { r } else { r * 0.68 };
                    pb.move_to(cx + r_in * theta.cos(), cy + r_in * theta.sin());
                    pb.line_to(cx + r_out * theta.cos(), cy + r_out * theta.sin());
                }
                if let Some(path) = pb.finish() {
                    stroke_paint.set_color(skia_color(a.color, 0.92));
                    pixmap.stroke_path(
                        &path,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                if let Some(core) = build_circle_path(cx, cy, r * 0.24) {
                    fill_paint.set_color(skia_color(pal.highlight, 0.92));
                    pixmap.fill_path(
                        &core,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
            }
            PrimaryGeometry::SacredTriangles => {
                if let Some(tri1) = build_regular_polygon_path(cx, cy, r, 3, a.rotation) {
                    pixmap.fill_path(
                        &tri1,
                        &fill_paint,
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                    pixmap.stroke_path(
                        &tri1,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                // Dual inverted triangle (Merkaba / hexagram geometry)
                if let Some(tri2) = build_regular_polygon_path(cx, cy, r * 0.85, 3, a.rotation + PI)
                {
                    stroke_paint.set_color(skia_color(pal.secondary_geom, 0.92));
                    pixmap.stroke_path(
                        &tri2,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
            PrimaryGeometry::HarmonicArcs => {
                let mut arc_stroke = stroke.clone();
                arc_stroke.width = (r * 0.22).max(2.0);
                for ring in 0..3 {
                    let rad = r * (0.42 + ring as f32 * 0.28);
                    let start = a.rotation + ring as f32 * 1.1;
                    let sweep = PI * (0.85 + (ring as f32) * 0.25);
                    if let Some(arc) = build_arc_path(cx, cy, rad, start, sweep, 24) {
                        stroke_paint.set_color(skia_color(
                            if ring % 2 == 0 {
                                a.color
                            } else {
                                pal.secondary_geom
                            },
                            0.88,
                        ));
                        pixmap.stroke_path(
                            &arc,
                            &stroke_paint,
                            &arc_stroke,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
        }
    }
}

fn render_layer5_co_occurrence(
    pixmap: &mut Pixmap,
    s: f32,
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
    substrate: &CellularSubstrate,
    anchors: &[GeometryAnchor],
) {
    if genome.secondary_geom == SecondaryGeometry::MinimalistVoid {
        return;
    }

    let mut fill_paint = Paint::default();
    fill_paint.anti_alias = true;
    fill_paint.set_color(skia_color(pal.secondary_geom, 0.90));

    let mut stroke_paint = Paint::default();
    stroke_paint.anti_alias = true;
    stroke_paint.set_color(skia_color(pal.highlight, 0.88));

    let mut stroke = Stroke::default();
    stroke.width = (s * 0.0022).max(1.0);

    for (idx, a) in anchors.iter().enumerate() {
        let cx = a.center.x * s;
        let cy = a.center.y * s;
        let r = a.radius * s;

        match genome.secondary_geom {
            SecondaryGeometry::MinimalistVoid => {}
            SecondaryGeometry::OrbitalMicroDots => {
                let satellites = 5 + (idx % 4);
                let orbit_r = r * 1.26;
                if let Some(orbit_path) = build_circle_path(cx, cy, orbit_r) {
                    stroke_paint.set_color(skia_color(pal.stroke_color, 0.35));
                    pixmap.stroke_path(
                        &orbit_path,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
                for k in 0..satellites {
                    let theta = a.rotation + (k as f32 / satellites as f32) * TAU;
                    let sx = cx + orbit_r * theta.cos();
                    let sy = cy + orbit_r * theta.sin();
                    let dot_r = (r * 0.14).max(2.0);
                    if let Some(dot) = build_circle_path(sx, sy, dot_r) {
                        fill_paint.set_color(skia_color(pal.highlight, 0.95));
                        pixmap.fill_path(
                            &dot,
                            &fill_paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            SecondaryGeometry::NestedFractals => {
                let branches = 3;
                for b in 0..branches {
                    let theta = a.rotation + (b as f32 / branches as f32) * TAU;
                    let bx = cx + r * 0.88 * theta.cos();
                    let by = cy + r * 0.88 * theta.sin();
                    if let Some(sub) =
                        build_regular_polygon_path(bx, by, r * 0.34, 3 + (idx % 3), theta)
                    {
                        fill_paint.set_color(skia_color(pal.secondary_geom, 0.82));
                        pixmap.fill_path(
                            &sub,
                            &fill_paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                        stroke_paint.set_color(skia_color(pal.highlight, 0.85));
                        pixmap.stroke_path(
                            &sub,
                            &stroke_paint,
                            &stroke,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
            SecondaryGeometry::CrosshairMarkers => {
                let mut pb = PathBuilder::new();
                let ext = r * 1.32;
                let gap = r * 0.45;
                // 4 axial ticks
                pb.move_to(cx - ext, cy);
                pb.line_to(cx - gap, cy);
                pb.move_to(cx + gap, cy);
                pb.line_to(cx + ext, cy);
                pb.move_to(cx, cy - ext);
                pb.line_to(cx, cy - gap);
                pb.move_to(cx, cy + gap);
                pb.line_to(cx, cy + ext);
                if let Some(path) = pb.finish() {
                    stroke_paint.set_color(skia_color(pal.highlight, 0.92));
                    pixmap.stroke_path(
                        &path,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
            SecondaryGeometry::ParallelHatching => {
                let mut pb = PathBuilder::new();
                let lines = 7;
                let dir_x = a.rotation.cos();
                let dir_y = a.rotation.sin();
                let perp_x = -dir_y;
                let perp_y = dir_x;
                for l in 0..lines {
                    let t = ((l as f32 / (lines - 1) as f32) - 0.5) * 2.0 * (r * 0.85);
                    let half_chord = ((r * r - t * t).max(0.0)).sqrt();
                    let mx = cx + perp_x * t;
                    let my = cy + perp_y * t;
                    pb.move_to(mx - dir_x * half_chord, my - dir_y * half_chord);
                    pb.line_to(mx + dir_x * half_chord, my + dir_y * half_chord);
                }
                if let Some(path) = pb.finish() {
                    stroke_paint.set_color(skia_color(pal.highlight, 0.78));
                    pixmap.stroke_path(
                        &path,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
            SecondaryGeometry::GoldenSpiralArcs => {
                let mut pb = PathBuilder::new();
                let steps = 36;
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    let angle = a.rotation + t * TAU * 1.5;
                    let rad = r * 0.2 + t * r * 1.15;
                    let px = cx + rad * angle.cos();
                    let py = cy + rad * angle.sin();
                    if step == 0 {
                        pb.move_to(px, py);
                    } else {
                        pb.line_to(px, py);
                    }
                }
                if let Some(path) = pb.finish() {
                    stroke_paint.set_color(skia_color(pal.secondary_geom, 0.92));
                    pixmap.stroke_path(
                        &path,
                        &stroke_paint,
                        &stroke,
                        Transform::identity(),
                        None,
                    );
                }
            }
            SecondaryGeometry::VertexSatellites => {
                if let Some(cell) = substrate.cells.get(a.cell_index) {
                    for v in &cell.vertices {
                        let vx = v.x * s;
                        let vy = v.y * s;
                        if let Some(node) = build_regular_polygon_path(
                            vx,
                            vy,
                            (s * 0.0085).max(2.0),
                            4,
                            PI * 0.25,
                        ) {
                            fill_paint.set_color(skia_color(pal.highlight, 0.90));
                            pixmap.fill_path(
                                &node,
                                &fill_paint,
                                FillRule::Winding,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn render_layer6_proximity(
    pixmap: &mut Pixmap,
    s: f32,
    genome: &BackgroundGenome,
    pal: &ResolvedPalette,
    _substrate: &CellularSubstrate,
    anchors: &[GeometryAnchor],
) {
    let mut stroke_paint = Paint::default();
    stroke_paint.anti_alias = true;

    let mut stroke = Stroke::default();
    stroke.line_cap = LineCap::Round;

    let thresh = genome.proximity_threshold;

    match genome.proximity {
        ProximityStyle::ThresholdLaserBridges => {
            for i in 0..anchors.len() {
                for j in (i + 1)..anchors.len() {
                    let d = anchors[i].center.dist(anchors[j].center);
                    if d <= thresh {
                        let strength = (1.0 - d / thresh).clamp(0.1, 1.0);
                        stroke.width = (s * 0.0038 * strength).max(1.0);
                        stroke_paint.set_color(skia_color(pal.proximity_glow, strength * 0.85));

                        let mut pb = PathBuilder::new();
                        pb.move_to(anchors[i].center.x * s, anchors[i].center.y * s);
                        pb.line_to(anchors[j].center.x * s, anchors[j].center.y * s);
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(
                                &path,
                                &stroke_paint,
                                &stroke,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
        ProximityStyle::KnnConstellationWeb => {
            stroke.width = (s * 0.0022).max(1.0);
            stroke_paint.set_color(skia_color(pal.proximity_glow, 0.72));
            let k_neighbors = 3usize;

            let mut pb = PathBuilder::new();
            for (i, a) in anchors.iter().enumerate() {
                let mut dists: Vec<(usize, f32)> = anchors
                    .iter()
                    .enumerate()
                    .filter(|&(j, _)| j != i)
                    .map(|(j, b)| (j, a.center.dist_sq(b.center)))
                    .collect();
                dists.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));

                for &(j, _) in dists.iter().take(k_neighbors) {
                    let b = &anchors[j];
                    pb.move_to(a.center.x * s, a.center.y * s);
                    pb.line_to(b.center.x * s, b.center.y * s);
                }
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &stroke_paint, &stroke, Transform::identity(), None);
            }
        }
        ProximityStyle::GabrielGraphNetwork => {
            // Edge (i, j) is in the Gabriel graph iff no other anchor k lies inside the circle with diameter (p_i, p_j)
            stroke.width = (s * 0.0028).max(1.0);
            stroke_paint.set_color(skia_color(pal.proximity_glow, 0.82));

            for i in 0..anchors.len() {
                for j in (i + 1)..anchors.len() {
                    let pi = anchors[i].center;
                    let pj = anchors[j].center;
                    let mid = Point2D::new((pi.x + pj.x) * 0.5, (pi.y + pj.y) * 0.5);
                    let rad_sq = pi.dist_sq(pj) * 0.25;

                    let mut blocked = false;
                    for (k, ak) in anchors.iter().enumerate() {
                        if k != i && k != j && ak.center.dist_sq(mid) < rad_sq * 0.98 {
                            blocked = true;
                            break;
                        }
                    }

                    if !blocked {
                        let mut pb = PathBuilder::new();
                        pb.move_to(pi.x * s, pi.y * s);
                        pb.line_to(pj.x * s, pj.y * s);
                        if let Some(path) = pb.finish() {
                            pixmap.stroke_path(
                                &path,
                                &stroke_paint,
                                &stroke,
                                Transform::identity(),
                                None,
                            );
                        }
                        if let Some(circle) = build_circle_path(mid.x * s, mid.y * s, rad_sq.sqrt() * s)
                        {
                            let mut ring_paint = Paint::default();
                            ring_paint.anti_alias = true;
                            ring_paint.set_color(skia_color(pal.proximity_glow, 0.20));
                            let mut thin = Stroke::default();
                            thin.width = (s * 0.0012).max(1.0);
                            pixmap.stroke_path(
                                &circle,
                                &ring_paint,
                                &thin,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
            }
        }
        ProximityStyle::HarmonicInterferenceRipples => {
            stroke.width = (s * 0.0016).max(1.0);
            for a in anchors {
                for ring in 1..=4 {
                    let rad = (a.radius + ring as f32 * thresh * 0.22) * s;
                    let alpha = (0.48 - ring as f32 * 0.09).clamp(0.1, 0.5);
                    stroke_paint.set_color(skia_color(pal.proximity_glow, alpha));
                    if let Some(circle) = build_circle_path(a.center.x * s, a.center.y * s, rad) {
                        pixmap.stroke_path(
                            &circle,
                            &stroke_paint,
                            &stroke,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
        }
        ProximityStyle::MagneticForceContours => {
            // Draws curved quadratic Bezier flux arcs between proximate anchors
            stroke.width = (s * 0.0022).max(1.0);
            for i in 0..anchors.len() {
                for j in (i + 1)..anchors.len() {
                    let pi = anchors[i].center;
                    let pj = anchors[j].center;
                    let d = pi.dist(pj);
                    if d <= thresh * 1.15 {
                        let strength = (1.0 - d / (thresh * 1.15)).clamp(0.15, 0.9);
                        stroke_paint.set_color(skia_color(pal.proximity_glow, strength * 0.75));
                        let mx = (pi.x + pj.x) * 0.5;
                        let my = (pi.y + pj.y) * 0.5;
                        let nx = -(pj.y - pi.y);
                        let ny = pj.x - pi.x;

                        for side in [-0.35_f32, 0.35_f32] {
                            let mut pb = PathBuilder::new();
                            pb.move_to(pi.x * s, pi.y * s);
                            pb.quad_to(
                                (mx + nx * side) * s,
                                (my + ny * side) * s,
                                pj.x * s,
                                pj.y * s,
                            );
                            if let Some(path) = pb.finish() {
                                pixmap.stroke_path(
                                    &path,
                                    &stroke_paint,
                                    &stroke,
                                    Transform::identity(),
                                    None,
                                );
                            }
                        }
                    }
                }
            }
        }
        ProximityStyle::GravityWells => {
            for a in anchors {
                let cx = a.center.x * s;
                let cy = a.center.y * s;
                let well_r = (a.radius + thresh * 0.55) * s;
                if let Some(shader) = RadialGradient::new(
                    Point::from_xy(cx, cy),
                    Point::from_xy(cx, cy),
                    well_r,
                    vec![
                        GradientStop::new(0.0, skia_color(pal.proximity_glow, 0.42)),
                        GradientStop::new(1.0, skia_color(pal.proximity_glow, 0.0)),
                    ],
                    SpreadMode::Pad,
                    Transform::identity(),
                ) {
                    let mut paint = Paint::default();
                    paint.shader = shader;
                    paint.anti_alias = true;
                    if let Some(circle) = build_circle_path(cx, cy, well_r) {
                        pixmap.fill_path(
                            &circle,
                            &paint,
                            FillRule::Winding,
                            Transform::identity(),
                            None,
                        );
                    }
                }
            }
        }
    }
}

fn render_subtle_frame(pixmap: &mut Pixmap, w: f32, h: f32, pal: &ResolvedPalette) {
    let s = w.min(h);
    let margin = s * 0.025;
    let mut pb = PathBuilder::new();
    pb.move_to(margin, margin);
    pb.line_to(w - margin, margin);
    pb.line_to(w - margin, h - margin);
    pb.line_to(margin, h - margin);
    pb.close();

    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(skia_color(pal.stroke_color, 0.35));
        paint.anti_alias = true;
        let mut stroke = Stroke::default();
        stroke.width = (s * 0.002).max(1.0);
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

fn build_circle_path(cx: f32, cy: f32, r: f32) -> Option<tiny_skia::Path> {
    PathBuilder::from_circle(cx, cy, r.max(0.5))
}

fn build_regular_polygon_path(
    cx: f32,
    cy: f32,
    r: f32,
    sides: usize,
    rotation: f32,
) -> Option<tiny_skia::Path> {
    let n = sides.max(3);
    let mut pb = PathBuilder::new();
    for i in 0..n {
        let theta = rotation + (i as f32 / n as f32) * TAU;
        let px = cx + r * theta.cos();
        let py = cy + r * theta.sin();
        if i == 0 {
            pb.move_to(px, py);
        } else {
            pb.line_to(px, py);
        }
    }
    pb.close();
    pb.finish()
}

fn build_arc_path(
    cx: f32,
    cy: f32,
    r: f32,
    start_angle: f32,
    sweep_angle: f32,
    segments: usize,
) -> Option<tiny_skia::Path> {
    let segs = segments.max(4);
    let mut pb = PathBuilder::new();
    for i in 0..=segs {
        let t = i as f32 / segs as f32;
        let theta = start_angle + t * sweep_angle;
        let px = cx + r * theta.cos();
        let py = cy + r * theta.sin();
        if i == 0 {
            pb.move_to(px, py);
        } else {
            pb.line_to(px, py);
        }
    }
    pb.finish()
}
