use crate::genome::SeedDistribution;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
pub struct Point2D {
    pub x: f32,
    pub y: f32,
}

impl Point2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn dist_sq(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn dist(self, other: Self) -> f32 {
        self.dist_sq(other).sqrt()
    }
}

#[derive(Debug, Clone)]
pub struct VoronoiCell {
    pub index: usize,
    pub centroid: Point2D,
    pub vertices: Vec<Point2D>,
    pub area: f32,
    pub inradius: f32,
    pub neighbors: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CellularSubstrate {
    pub sites: Vec<Point2D>,
    pub cells: Vec<VoronoiCell>,
    pub delaunay_edges: Vec<(usize, usize)>,
    pub delaunay_triangles: Vec<(usize, usize, usize)>,
}

impl CellularSubstrate {
    pub fn generate(
        seed: u64,
        count: usize,
        distribution: SeedDistribution,
        relaxation_steps: u8,
    ) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0xA5A5_5A5A_F0F0_0F0F);
        let n = count.clamp(8, 96);
        let mut sites = generate_initial_sites(&mut rng, n, distribution);

        let mut cells = Vec::new();
        for step in 0..=relaxation_steps {
            cells = compute_voronoi_cells(&sites);
            if step < relaxation_steps {
                for (i, cell) in cells.iter().enumerate() {
                    // Move site toward cell centroid (damped to keep inside margins)
                    sites[i].x = (sites[i].x * 0.25 + cell.centroid.x * 0.75).clamp(0.04, 0.96);
                    sites[i].y = (sites[i].y * 0.25 + cell.centroid.y * 0.75).clamp(0.04, 0.96);
                }
            }
        }

        // Extract dual Delaunay edges and triangles from Voronoi cell neighbor adjacencies
        let mut delaunay_edges = Vec::new();
        let mut delaunay_triangles = Vec::new();

        for i in 0..cells.len() {
            for &j in &cells[i].neighbors {
                if i < j {
                    delaunay_edges.push((i, j));
                    for &k in &cells[i].neighbors {
                        if j < k && cells[j].neighbors.contains(&k) {
                            delaunay_triangles.push((i, j, k));
                        }
                    }
                }
            }
        }

        Self {
            sites,
            cells,
            delaunay_edges,
            delaunay_triangles,
        }
    }

    /// Evaluates Worley cellular distances (F1, F2, closest_index) at normalized point (u, v)
    pub fn worley_at(&self, u: f32, v: f32) -> (f32, f32, usize) {
        let p = Point2D::new(u, v);
        let mut d1 = f32::MAX;
        let mut d2 = f32::MAX;
        let mut idx1 = 0;

        for (i, &s) in self.sites.iter().enumerate() {
            let d = p.dist_sq(s);
            if d < d1 {
                d2 = d1;
                d1 = d;
                idx1 = i;
            } else if d < d2 {
                d2 = d;
            }
        }
        (d1.sqrt(), d2.sqrt(), idx1)
    }
}

fn generate_initial_sites(
    rng: &mut ChaCha8Rng,
    n: usize,
    distribution: SeedDistribution,
) -> Vec<Point2D> {
    let mut sites = Vec::with_capacity(n);
    match distribution {
        SeedDistribution::JitteredGrid => {
            let cols = (n as f32).sqrt().ceil() as usize;
            let rows = (n + cols - 1) / cols;
            for idx in 0..n {
                let r = idx / cols;
                let c = idx % cols;
                let base_x = (c as f32 + 0.5) / cols as f32;
                let base_y = (r as f32 + 0.5) / rows as f32;
                let jx = rng.gen_range(-0.38..0.38) / cols as f32;
                let jy = rng.gen_range(-0.38..0.38) / rows as f32;
                sites.push(Point2D::new(
                    (base_x + jx).clamp(0.05, 0.95),
                    (base_y + jy).clamp(0.05, 0.95),
                ));
            }
        }
        SeedDistribution::FibonacciSpiral => {
            let golden_angle = std::f32::consts::PI * (3.0 - 5.0_f32.sqrt());
            let phase = rng.gen_range(0.0..TAU);
            for i in 0..n {
                let t = (i as f32 + 0.5) / n as f32;
                let radius = 0.06 + 0.40 * t.sqrt();
                let theta = i as f32 * golden_angle + phase + rng.gen_range(-0.08..0.08);
                let x = (0.5 + radius * theta.cos()).clamp(0.05, 0.95);
                let y = (0.5 + radius * theta.sin()).clamp(0.05, 0.95);
                sites.push(Point2D::new(x, y));
            }
        }
        SeedDistribution::ClusteredNuclei => {
            let num_clusters = rng.gen_range(3..=5);
            let mut centers = Vec::with_capacity(num_clusters);
            for _ in 0..num_clusters {
                centers.push(Point2D::new(
                    rng.gen_range(0.18..0.82),
                    rng.gen_range(0.18..0.82),
                ));
            }
            for i in 0..n {
                let c = centers[i % num_clusters];
                let angle = rng.gen_range(0.0..TAU);
                let rad = rng.gen_range(0.03..0.24);
                sites.push(Point2D::new(
                    (c.x + rad * angle.cos()).clamp(0.05, 0.95),
                    (c.y + rad * angle.sin()).clamp(0.05, 0.95),
                ));
            }
        }
        SeedDistribution::HarmonicRing => {
            let rings = 3usize;
            for i in 0..n {
                let ring_idx = (i % rings) + 1;
                let r = (ring_idx as f32 / (rings as f32 + 0.4)) * 0.43;
                let theta = (i as f32 / n as f32) * TAU * (ring_idx as f32)
                    + rng.gen_range(-0.12..0.12);
                sites.push(Point2D::new(
                    (0.5 + r * theta.cos()).clamp(0.05, 0.95),
                    (0.5 + r * theta.sin()).clamp(0.05, 0.95),
                ));
            }
        }
    }
    sites
}

/// Computes exact 2D Voronoi cells inside `[0.03, 0.97]^2` via Sutherland-Hodgman half-plane clipping
fn compute_voronoi_cells(sites: &[Point2D]) -> Vec<VoronoiCell> {
    let margin = 0.035;
    let base_box = vec![
        Point2D::new(margin, margin),
        Point2D::new(1.0 - margin, margin),
        Point2D::new(1.0 - margin, 1.0 - margin),
        Point2D::new(margin, 1.0 - margin),
    ];

    let mut cells = Vec::with_capacity(sites.len());

    for (i, &p_i) in sites.iter().enumerate() {
        let mut poly = base_box.clone();
        let mut neighbors = Vec::new();

        // Sort other sites by distance to p_i for fast clipping
        let mut order: Vec<(usize, f32)> = sites
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(j, &p_j)| (j, p_i.dist_sq(p_j)))
            .collect();
        order.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        for (j, _) in order {
            let p_j = sites[j];
            let mid = Point2D::new((p_i.x + p_j.x) * 0.5, (p_i.y + p_j.y) * 0.5);
            // Normal pointing from p_i to p_j: point q is inside if dot(q - mid, p_j - p_i) <= 0
            let nx = p_j.x - p_i.x;
            let ny = p_j.y - p_i.y;

            if nx * nx + ny * ny < 1e-9 {
                continue;
            }

            let (clipped, clipped_by_j) = clip_polygon_half_plane(&poly, mid, nx, ny);
            poly = clipped;
            if clipped_by_j {
                neighbors.push(j);
            }
            if poly.len() < 3 {
                break;
            }
        }

        // Filter true neighbors (only those whose perpendicular bisector still touches an edge of the final polygon)
        neighbors.retain(|&j| {
            let p_j = sites[j];
            let mid = Point2D::new((p_i.x + p_j.x) * 0.5, (p_i.y + p_j.y) * 0.5);
            let nx = p_j.x - p_i.x;
            let ny = p_j.y - p_i.y;
            let norm = (nx * nx + ny * ny).sqrt().max(1e-6);
            let mut on_line = 0;
            for v in &poly {
                let signed_dist = ((v.x - mid.x) * nx + (v.y - mid.y) * ny).abs() / norm;
                if signed_dist < 1e-3 {
                    on_line += 1;
                }
            }
            on_line >= 2
        });

        let (centroid, area) = polygon_centroid_and_area(&poly, p_i);
        let inradius = polygon_inradius(&poly, centroid);

        cells.push(VoronoiCell {
            index: i,
            centroid,
            vertices: poly,
            area,
            inradius,
            neighbors,
        });
    }

    cells
}

fn clip_polygon_half_plane(
    poly: &[Point2D],
    mid: Point2D,
    nx: f32,
    ny: f32,
) -> (Vec<Point2D>, bool) {
    if poly.is_empty() {
        return (Vec::new(), false);
    }

    let eval = |p: Point2D| (p.x - mid.x) * nx + (p.y - mid.y) * ny;
    let mut out = Vec::with_capacity(poly.len() + 1);
    let mut clipped_any = false;

    for idx in 0..poly.len() {
        let curr = poly[idx];
        let prev = poly[(idx + poly.len() - 1) % poly.len()];
        let d_curr = eval(curr);
        let d_prev = eval(prev);

        let curr_inside = d_curr <= 1e-6;
        let prev_inside = d_prev <= 1e-6;

        if curr_inside {
            if !prev_inside {
                let denom = d_prev - d_curr;
                let t = if denom.abs() > 1e-9 {
                    (d_prev / denom).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                out.push(Point2D::new(
                    prev.x + t * (curr.x - prev.x),
                    prev.y + t * (curr.y - prev.y),
                ));
                clipped_any = true;
            }
            out.push(curr);
        } else if prev_inside {
            let denom = d_prev - d_curr;
            let t = if denom.abs() > 1e-9 {
                (d_prev / denom).clamp(0.0, 1.0)
            } else {
                0.5
            };
            out.push(Point2D::new(
                prev.x + t * (curr.x - prev.x),
                prev.y + t * (curr.y - prev.y),
            ));
            clipped_any = true;
        } else {
            clipped_any = true;
        }
    }

    (out, clipped_any)
}

fn polygon_centroid_and_area(poly: &[Point2D], fallback: Point2D) -> (Point2D, f32) {
    if poly.len() < 3 {
        return (fallback, 0.001);
    }
    let mut signed_area = 0.0;
    let mut cx = 0.0;
    let mut cy = 0.0;

    for i in 0..poly.len() {
        let p0 = poly[i];
        let p1 = poly[(i + 1) % poly.len()];
        let cross = p0.x * p1.y - p1.x * p0.y;
        signed_area += cross;
        cx += (p0.x + p1.x) * cross;
        cy += (p0.y + p1.y) * cross;
    }

    signed_area *= 0.5;
    if signed_area.abs() < 1e-6 {
        return (fallback, 0.001);
    }
    let factor = 1.0 / (6.0 * signed_area);
    (
        Point2D::new(cx * factor, cy * factor),
        signed_area.abs(),
    )
}

fn polygon_inradius(poly: &[Point2D], center: Point2D) -> f32 {
    if poly.len() < 3 {
        return 0.02;
    }
    let mut min_dist = f32::MAX;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len_sq = dx * dx + dy * dy;
        let dist = if len_sq > 1e-8 {
            ((dy * center.x - dx * center.y + b.x * a.y - b.y * a.x).abs()) / len_sq.sqrt()
        } else {
            center.dist(a)
        };
        if dist < min_dist {
            min_dist = dist;
        }
    }
    min_dist.clamp(0.015, 0.25)
}
