use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Layer 1: Curated Algorithmic Palettes in perceptual OKLCH space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorPaletteTrait {
    CyberObsidian,
    SolarFlare,
    AcidBauhaus,
    BioluminescentAbyss,
    MonolithBone,
    UltravioletBrutalism,
    InfraredQuartz,
    GlacierCarbon,
    SynthwaveRust,
    EmeraldMatrix,
    ChromaticVoid,
    GoldenRatio,
    SubzeroCobalt,
    PlasmaOpal,
}

impl ColorPaletteTrait {
    pub const ALL: &'static [ColorPaletteTrait] = &[
        ColorPaletteTrait::CyberObsidian,
        ColorPaletteTrait::SolarFlare,
        ColorPaletteTrait::AcidBauhaus,
        ColorPaletteTrait::BioluminescentAbyss,
        ColorPaletteTrait::MonolithBone,
        ColorPaletteTrait::UltravioletBrutalism,
        ColorPaletteTrait::InfraredQuartz,
        ColorPaletteTrait::GlacierCarbon,
        ColorPaletteTrait::SynthwaveRust,
        ColorPaletteTrait::EmeraldMatrix,
        ColorPaletteTrait::ChromaticVoid,
        ColorPaletteTrait::GoldenRatio,
        ColorPaletteTrait::SubzeroCobalt,
        ColorPaletteTrait::PlasmaOpal,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::CyberObsidian => "Cyber Obsidian",
            Self::SolarFlare => "Solar Flare",
            Self::AcidBauhaus => "Acid Bauhaus",
            Self::BioluminescentAbyss => "Bioluminescent Abyss",
            Self::MonolithBone => "Monolith Bone",
            Self::UltravioletBrutalism => "Ultraviolet Brutalism",
            Self::InfraredQuartz => "Infrared Quartz",
            Self::GlacierCarbon => "Glacier Carbon",
            Self::SynthwaveRust => "Synthwave Rust",
            Self::EmeraldMatrix => "Emerald Matrix",
            Self::ChromaticVoid => "Chromatic Void",
            Self::GoldenRatio => "Golden Ratio",
            Self::SubzeroCobalt => "Subzero Cobalt",
            Self::PlasmaOpal => "Plasma Opal",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::CyberObsidian | Self::SolarFlare | Self::GlacierCarbon | Self::MonolithBone => 14,
            Self::AcidBauhaus | Self::BioluminescentAbyss | Self::SynthwaveRust | Self::SubzeroCobalt => 10,
            Self::UltravioletBrutalism | Self::InfraredQuartz | Self::EmeraldMatrix => 6,
            Self::ChromaticVoid | Self::GoldenRatio | Self::PlasmaOpal => 3,
        }
    }

    /// Base OKLCH seed parameters: (bg_lightness, bg_chroma, base_hue_deg, accent_chroma)
    pub fn base_oklch(&self) -> (f32, f32, f32, f32) {
        match self {
            Self::CyberObsidian => (0.14, 0.03, 250.0, 0.23),
            Self::SolarFlare => (0.16, 0.05, 32.0, 0.24),
            Self::AcidBauhaus => (0.18, 0.04, 115.0, 0.26),
            Self::BioluminescentAbyss => (0.13, 0.05, 195.0, 0.22),
            Self::MonolithBone => (0.91, 0.02, 75.0, 0.16),
            Self::UltravioletBrutalism => (0.15, 0.07, 295.0, 0.26),
            Self::InfraredQuartz => (0.15, 0.05, 355.0, 0.25),
            Self::GlacierCarbon => (0.17, 0.02, 215.0, 0.18),
            Self::SynthwaveRust => (0.18, 0.06, 18.0, 0.22),
            Self::EmeraldMatrix => (0.13, 0.04, 152.0, 0.23),
            Self::ChromaticVoid => (0.10, 0.01, 275.0, 0.28),
            Self::GoldenRatio => (0.15, 0.03, 55.0, 0.21),
            Self::SubzeroCobalt => (0.16, 0.07, 238.0, 0.23),
            Self::PlasmaOpal => (0.88, 0.04, 315.0, 0.24),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HarmonyMode {
    Complementary,
    Triadic,
    SplitComplementary,
    Analogous,
    MonochromeAccent,
}

impl HarmonyMode {
    pub const ALL: &'static [HarmonyMode] = &[
        HarmonyMode::Complementary,
        HarmonyMode::Triadic,
        HarmonyMode::SplitComplementary,
        HarmonyMode::Analogous,
        HarmonyMode::MonochromeAccent,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Complementary => "Complementary",
            Self::Triadic => "Triadic",
            Self::SplitComplementary => "Split-Complementary",
            Self::Analogous => "Analogous",
            Self::MonochromeAccent => "Monochrome Accent",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::Complementary => 30,
            Self::Triadic => 25,
            Self::SplitComplementary => 22,
            Self::Analogous => 15,
            Self::MonochromeAccent => 8,
        }
    }

    pub fn hue_offsets(&self) -> [f32; 5] {
        match self {
            Self::Complementary => [0.0, 180.0, 15.0, 195.0, 345.0],
            Self::Triadic => [0.0, 120.0, 240.0, 60.0, 300.0],
            Self::SplitComplementary => [0.0, 150.0, 210.0, 30.0, 180.0],
            Self::Analogous => [0.0, 28.0, 332.0, 56.0, 304.0],
            Self::MonochromeAccent => [0.0, 4.0, 356.0, 8.0, 180.0],
        }
    }
}

/// Resolved RGBA Palette for rendering
#[derive(Debug, Clone, Copy)]
pub struct ResolvedPalette {
    pub bg_primary: [u8; 4],
    pub bg_secondary: [u8; 4],
    pub cell_tones: [[u8; 4]; 5],
    pub stroke_color: [u8; 4],
    pub primary_geom: [u8; 4],
    pub secondary_geom: [u8; 4],
    pub proximity_glow: [u8; 4],
    pub highlight: [u8; 4],
}

/// Convert perceptual OKLCH (L in 0..1, C in 0..0.4, H in 0..360) to sRGB [u8; 4]
pub fn oklch_to_rgba(l: f32, c: f32, h_deg: f32, alpha: u8) -> [u8; 4] {
    let h_rad = h_deg.rem_euclid(360.0).to_radians();
    let a = c * h_rad.cos();
    let b = c * h_rad.sin();

    // OKLab to linear LMS
    let l_ = l + 0.39633778 * a + 0.21580376 * b;
    let m_ = l - 0.105561346 * a - 0.06385417 * b;
    let s_ = l - 0.08948418 * a - 1.2914855 * b;

    let l_lin = l_ * l_ * l_;
    let m_lin = m_ * m_ * m_;
    let s_lin = s_ * s_ * s_;

    // Linear sRGB
    let r_lin = 4.0767417 * l_lin - 3.3077116 * m_lin + 0.23096994 * s_lin;
    let g_lin = -1.268438 * l_lin + 2.6097574 * m_lin - 0.34131938 * s_lin;
    let b_lin = -0.0041960863 * l_lin - 0.7034186 * m_lin + 1.7076147 * s_lin;

    fn linear_to_srgb(x: f32) -> u8 {
        let clamped = x.clamp(0.0, 1.0);
        let s = if clamped <= 0.0031308 {
            12.92 * clamped
        } else {
            1.055 * clamped.powf(1.0 / 2.4) - 0.055
        };
        (s.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    [
        linear_to_srgb(r_lin),
        linear_to_srgb(g_lin),
        linear_to_srgb(b_lin),
        alpha,
    ]
}

impl ColorPaletteTrait {
    pub fn resolve(&self, harmony: HarmonyMode, hue_jitter: f32) -> ResolvedPalette {
        let (bg_l, bg_c, base_h, acc_c) = self.base_oklch();
        let h0 = (base_h + hue_jitter).rem_euclid(360.0);
        let offsets = harmony.hue_offsets();
        let is_light_bg = bg_l > 0.5;

        let bg_primary = oklch_to_rgba(bg_l, bg_c, h0, 255);
        let bg_secondary = oklch_to_rgba(
            if is_light_bg { bg_l - 0.12 } else { bg_l + 0.10 },
            bg_c * 1.4,
            h0 + offsets[1] * 0.35,
            255,
        );

        let mut cell_tones = [[0u8; 4]; 5];
        for (i, d_h) in offsets.iter().enumerate() {
            let l_step = if is_light_bg {
                0.78 - (i as f32) * 0.11
            } else {
                0.24 + (i as f32) * 0.12
            };
            let c_step = acc_c * (0.55 + (i as f32 % 3.0) * 0.22);
            cell_tones[i] = oklch_to_rgba(l_step, c_step, h0 + *d_h, 225);
        }

        let stroke_color = if is_light_bg {
            oklch_to_rgba(0.20, 0.03, h0, 220)
        } else {
            oklch_to_rgba(0.86, 0.06, h0 + offsets[0], 210)
        };

        let primary_geom = oklch_to_rgba(
            if is_light_bg { 0.32 } else { 0.78 },
            acc_c * 1.15,
            h0 + offsets[1],
            245,
        );
        let secondary_geom = oklch_to_rgba(
            if is_light_bg { 0.45 } else { 0.84 },
            acc_c * 1.2,
            h0 + offsets[2],
            240,
        );
        let proximity_glow = oklch_to_rgba(
            if is_light_bg { 0.25 } else { 0.90 },
            acc_c * 1.1,
            h0 + offsets[3],
            215,
        );
        let highlight = oklch_to_rgba(
            if is_light_bg { 0.18 } else { 0.95 },
            acc_c * 0.9,
            h0 + offsets[4],
            255,
        );

        ResolvedPalette {
            bg_primary,
            bg_secondary,
            cell_tones,
            stroke_color,
            primary_geom,
            secondary_geom,
            proximity_glow,
            highlight,
        }
    }
}

/// Layer 2: Background Field Style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BackgroundStyle {
    DeepVoid,
    RadialNebulaGradient,
    LinearHorizon,
    MicroBlueprintGrid,
    QuantizedNoiseField,
    DualToneDiagonal,
    ConcentricPulse,
}

impl BackgroundStyle {
    pub const ALL: &'static [BackgroundStyle] = &[
        Self::DeepVoid,
        Self::RadialNebulaGradient,
        Self::LinearHorizon,
        Self::MicroBlueprintGrid,
        Self::QuantizedNoiseField,
        Self::DualToneDiagonal,
        Self::ConcentricPulse,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::DeepVoid => "Deep Void",
            Self::RadialNebulaGradient => "Radial Nebula Gradient",
            Self::LinearHorizon => "Linear Horizon",
            Self::MicroBlueprintGrid => "Micro Blueprint Grid",
            Self::QuantizedNoiseField => "Quantized Noise Field",
            Self::DualToneDiagonal => "Dual-Tone Diagonal",
            Self::ConcentricPulse => "Concentric Pulse",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::RadialNebulaGradient | Self::LinearHorizon => 22,
            Self::DeepVoid | Self::MicroBlueprintGrid => 16,
            Self::DualToneDiagonal | Self::QuantizedNoiseField => 9,
            Self::ConcentricPulse => 6,
        }
    }
}

/// Layer 3: Cellular Math Substrate (Voronoi / Delaunay / Worley)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellularStyle {
    VoronoiCrystalCells,
    DelaunayTriangulationMesh,
    WorleyDistanceRidges,
    DualVoronoiDelaunayLattice,
    FracturedRelaxedShards,
}

impl CellularStyle {
    pub const ALL: &'static [CellularStyle] = &[
        Self::VoronoiCrystalCells,
        Self::DelaunayTriangulationMesh,
        Self::WorleyDistanceRidges,
        Self::DualVoronoiDelaunayLattice,
        Self::FracturedRelaxedShards,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::VoronoiCrystalCells => "Voronoi Crystal Cells",
            Self::DelaunayTriangulationMesh => "Delaunay Triangulation Mesh",
            Self::WorleyDistanceRidges => "Worley F2-F1 Distance Ridges",
            Self::DualVoronoiDelaunayLattice => "Dual Voronoi-Delaunay Lattice",
            Self::FracturedRelaxedShards => "Fractured Relaxed Shards",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::VoronoiCrystalCells => 30,
            Self::FracturedRelaxedShards => 25,
            Self::DelaunayTriangulationMesh => 20,
            Self::DualVoronoiDelaunayLattice => 15,
            Self::WorleyDistanceRidges => 10,
        }
    }
}

/// Seed distribution algorithm for the Cellular Math substrate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeedDistribution {
    JitteredGrid,
    FibonacciSpiral,
    ClusteredNuclei,
    HarmonicRing,
}

impl SeedDistribution {
    pub const ALL: &'static [SeedDistribution] = &[
        Self::JitteredGrid,
        Self::FibonacciSpiral,
        Self::ClusteredNuclei,
        Self::HarmonicRing,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::JitteredGrid => "Poisson Jittered Grid",
            Self::FibonacciSpiral => "Golden Fibonacci Spiral",
            Self::ClusteredNuclei => "Clustered Nuclei",
            Self::HarmonicRing => "Harmonic Orbital Rings",
        }
    }
}

/// Layer 4: Primary Geometry anchored to Voronoi cells
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimaryGeometry {
    ConcentricRings,
    InscribedPolygons,
    BauhausRectilinear,
    IsometricPrisms,
    RadialVectorBursts,
    SacredTriangles,
    HarmonicArcs,
}

impl PrimaryGeometry {
    pub const ALL: &'static [PrimaryGeometry] = &[
        Self::ConcentricRings,
        Self::InscribedPolygons,
        Self::BauhausRectilinear,
        Self::IsometricPrisms,
        Self::RadialVectorBursts,
        Self::SacredTriangles,
        Self::HarmonicArcs,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::ConcentricRings => "Concentric Rings",
            Self::InscribedPolygons => "Inscribed Regular Polygons",
            Self::BauhausRectilinear => "Bauhaus Rectilinear Blocks",
            Self::IsometricPrisms => "Isometric Prisms",
            Self::RadialVectorBursts => "Radial Vector Bursts",
            Self::SacredTriangles => "Sacred Triangles",
            Self::HarmonicArcs => "Harmonic Segmented Arcs",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::ConcentricRings | Self::InscribedPolygons => 20,
            Self::BauhausRectilinear | Self::SacredTriangles => 16,
            Self::HarmonicArcs | Self::RadialVectorBursts => 11,
            Self::IsometricPrisms => 6,
        }
    }
}

/// Layer 5: Co-occurrence Secondary Geometry governed by affinity/repulsion matrix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecondaryGeometry {
    OrbitalMicroDots,
    NestedFractals,
    CrosshairMarkers,
    ParallelHatching,
    GoldenSpiralArcs,
    VertexSatellites,
    MinimalistVoid,
}

impl SecondaryGeometry {
    pub const ALL: &'static [SecondaryGeometry] = &[
        Self::OrbitalMicroDots,
        Self::NestedFractals,
        Self::CrosshairMarkers,
        Self::ParallelHatching,
        Self::GoldenSpiralArcs,
        Self::VertexSatellites,
        Self::MinimalistVoid,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::OrbitalMicroDots => "Orbital Micro-Dots",
            Self::NestedFractals => "Nested Sub-Fractals",
            Self::CrosshairMarkers => "Precision Crosshair Markers",
            Self::ParallelHatching => "Parallel Vector Hatching",
            Self::GoldenSpiralArcs => "Golden Spiral Satellites",
            Self::VertexSatellites => "Voronoi Vertex Satellites",
            Self::MinimalistVoid => "Pure Minimalist (None)",
        }
    }

    /// Co-occurrence conditional weight given PrimaryGeometry.
    /// Returns 0 for forbidden pairings (strict repulsion) and higher numbers for harmonic synergy.
    pub fn co_occurrence_weight(primary: PrimaryGeometry, secondary: SecondaryGeometry) -> u32 {
        use PrimaryGeometry as P;
        use SecondaryGeometry as S;
        match (primary, secondary) {
            // ConcentricRings loves OrbitalMicroDots, CrosshairMarkers, GoldenSpiralArcs; repels NestedFractals
            (P::ConcentricRings, S::OrbitalMicroDots) => 35,
            (P::ConcentricRings, S::CrosshairMarkers) => 25,
            (P::ConcentricRings, S::GoldenSpiralArcs) => 22,
            (P::ConcentricRings, S::VertexSatellites) => 12,
            (P::ConcentricRings, S::MinimalistVoid) => 6,
            (P::ConcentricRings, S::NestedFractals) => 0, // Repulsion
            (P::ConcentricRings, S::ParallelHatching) => 0, // Repulsion

            // InscribedPolygons loves NestedFractals, VertexSatellites, ParallelHatching
            (P::InscribedPolygons, S::NestedFractals) => 34,
            (P::InscribedPolygons, S::VertexSatellites) => 28,
            (P::InscribedPolygons, S::ParallelHatching) => 20,
            (P::InscribedPolygons, S::OrbitalMicroDots) => 12,
            (P::InscribedPolygons, S::MinimalistVoid) => 6,
            (P::InscribedPolygons, S::GoldenSpiralArcs) => 0, // Repulsion
            (P::InscribedPolygons, S::CrosshairMarkers) => 0,

            // BauhausRectilinear loves CrosshairMarkers, ParallelHatching, NestedFractals; repels GoldenSpiralArcs
            (P::BauhausRectilinear, S::ParallelHatching) => 36,
            (P::BauhausRectilinear, S::CrosshairMarkers) => 30,
            (P::BauhausRectilinear, S::NestedFractals) => 20,
            (P::BauhausRectilinear, S::OrbitalMicroDots) => 8,
            (P::BauhausRectilinear, S::MinimalistVoid) => 6,
            (P::BauhausRectilinear, S::GoldenSpiralArcs) => 0,
            (P::BauhausRectilinear, S::VertexSatellites) => 0,

            // IsometricPrisms loves VertexSatellites, CrosshairMarkers, OrbitalMicroDots
            (P::IsometricPrisms, S::VertexSatellites) => 38,
            (P::IsometricPrisms, S::CrosshairMarkers) => 26,
            (P::IsometricPrisms, S::OrbitalMicroDots) => 20,
            (P::IsometricPrisms, S::ParallelHatching) => 12,
            (P::IsometricPrisms, S::MinimalistVoid) => 4,
            (P::IsometricPrisms, S::GoldenSpiralArcs) => 0,
            (P::IsometricPrisms, S::NestedFractals) => 0,

            // RadialVectorBursts loves OrbitalMicroDots, GoldenSpiralArcs, VertexSatellites
            (P::RadialVectorBursts, S::OrbitalMicroDots) => 34,
            (P::RadialVectorBursts, S::GoldenSpiralArcs) => 28,
            (P::RadialVectorBursts, S::VertexSatellites) => 22,
            (P::RadialVectorBursts, S::CrosshairMarkers) => 12,
            (P::RadialVectorBursts, S::MinimalistVoid) => 4,
            (P::RadialVectorBursts, S::ParallelHatching) => 0,
            (P::RadialVectorBursts, S::NestedFractals) => 0,

            // SacredTriangles loves NestedFractals, VertexSatellites, CrosshairMarkers
            (P::SacredTriangles, S::NestedFractals) => 35,
            (P::SacredTriangles, S::VertexSatellites) => 30,
            (P::SacredTriangles, S::CrosshairMarkers) => 18,
            (P::SacredTriangles, S::ParallelHatching) => 12,
            (P::SacredTriangles, S::MinimalistVoid) => 5,
            (P::SacredTriangles, S::OrbitalMicroDots) => 0,
            (P::SacredTriangles, S::GoldenSpiralArcs) => 0,

            // HarmonicArcs loves GoldenSpiralArcs, OrbitalMicroDots, CrosshairMarkers
            (P::HarmonicArcs, S::GoldenSpiralArcs) => 38,
            (P::HarmonicArcs, S::OrbitalMicroDots) => 32,
            (P::HarmonicArcs, S::CrosshairMarkers) => 16,
            (P::HarmonicArcs, S::ParallelHatching) => 10,
            (P::HarmonicArcs, S::MinimalistVoid) => 4,
            (P::HarmonicArcs, S::NestedFractals) => 0,
            (P::HarmonicArcs, S::VertexSatellites) => 0,
        }
    }
}

/// Layer 6: Spatial Proximity Connections & Fields
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProximityStyle {
    KnnConstellationWeb,
    ThresholdLaserBridges,
    HarmonicInterferenceRipples,
    MagneticForceContours,
    GabrielGraphNetwork,
    GravityWells,
}

impl ProximityStyle {
    pub const ALL: &'static [ProximityStyle] = &[
        Self::KnnConstellationWeb,
        Self::ThresholdLaserBridges,
        Self::HarmonicInterferenceRipples,
        Self::MagneticForceContours,
        Self::GabrielGraphNetwork,
        Self::GravityWells,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::KnnConstellationWeb => "k-NN Constellation Web",
            Self::ThresholdLaserBridges => "Distance-Threshold Laser Bridges",
            Self::HarmonicInterferenceRipples => "Harmonic Interference Ripples",
            Self::MagneticForceContours => "Magnetic Force Contours",
            Self::GabrielGraphNetwork => "Gabriel Graph Network",
            Self::GravityWells => "Spatial Gravity Wells",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::KnnConstellationWeb | Self::ThresholdLaserBridges => 24,
            Self::GabrielGraphNetwork | Self::HarmonicInterferenceRipples => 18,
            Self::MagneticForceContours => 10,
            Self::GravityWells => 6,
        }
    }
}

/// Layer 7: Effects & Glitch (Pixel Sorting & Scanline Displacement)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GlitchStyle {
    PristineClean,
    HorizontalLuminancePixelSort,
    VerticalGravityPixelSort,
    ScanlineSliceDisplacement,
    DualSortAndScanlineShear,
    MicroScanlineJitter,
}

impl GlitchStyle {
    pub const ALL: &'static [GlitchStyle] = &[
        Self::PristineClean,
        Self::HorizontalLuminancePixelSort,
        Self::VerticalGravityPixelSort,
        Self::ScanlineSliceDisplacement,
        Self::DualSortAndScanlineShear,
        Self::MicroScanlineJitter,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::PristineClean => "Pristine (No Glitch)",
            Self::HorizontalLuminancePixelSort => "Luminance Pixel Sort (Horizontal)",
            Self::VerticalGravityPixelSort => "Gravity Pixel Sort (Vertical)",
            Self::ScanlineSliceDisplacement => "Scanline Slice Displacement",
            Self::DualSortAndScanlineShear => "Dual Pixel Sort + Scanline Shear",
            Self::MicroScanlineJitter => "Quantum Scanline Micro-Jitter",
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            Self::PristineClean => 28,
            Self::ScanlineSliceDisplacement => 22,
            Self::HorizontalLuminancePixelSort => 20,
            Self::MicroScanlineJitter => 14,
            Self::VerticalGravityPixelSort => 10,
            Self::DualSortAndScanlineShear => 6,
        }
    }
}

/// Uniqueness signature of the 7 discrete layers + cellular distribution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GenomeSignature {
    pub palette: ColorPaletteTrait,
    pub harmony: HarmonyMode,
    pub background: BackgroundStyle,
    pub cellular: CellularStyle,
    pub distribution: SeedDistribution,
    pub primary_geom: PrimaryGeometry,
    pub secondary_geom: SecondaryGeometry,
    pub proximity: ProximityStyle,
    pub glitch: GlitchStyle,
}

/// Complete procedural DNA of a single generated background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundGenome {
    pub id: u32,
    pub seed: u64,
    // Layer 1: Color
    pub palette: ColorPaletteTrait,
    pub harmony: HarmonyMode,
    pub hue_jitter: f32,
    // Layer 2: Background
    pub background: BackgroundStyle,
    // Layer 3: Cellular Math
    pub cellular: CellularStyle,
    pub seed_distribution: SeedDistribution,
    pub cell_count: usize,
    pub lloyd_relaxation_steps: u8,
    // Layer 4: Primary Geometry
    pub primary_geom: PrimaryGeometry,
    pub geom_density: f32,
    pub geom_scale: f32,
    // Layer 5: Co-occurrence Secondary Geometry
    pub secondary_geom: SecondaryGeometry,
    pub co_occurrence_affinity: u32,
    // Layer 6: Proximity
    pub proximity: ProximityStyle,
    pub proximity_threshold: f32,
    // Layer 7: Effects & Glitch
    pub glitch: GlitchStyle,
    pub glitch_intensity: f32,
}

impl BackgroundGenome {
    pub fn signature(&self) -> GenomeSignature {
        GenomeSignature {
            palette: self.palette,
            harmony: self.harmony,
            background: self.background,
            cellular: self.cellular,
            distribution: self.seed_distribution,
            primary_geom: self.primary_geom,
            secondary_geom: self.secondary_geom,
            proximity: self.proximity,
            glitch: self.glitch,
        }
    }
}

fn sample_weighted<T: Copy>(rng: &mut ChaCha8Rng, items: &[T], weight_fn: impl Fn(T) -> u32) -> T {
    let total: u32 = items.iter().map(|&x| weight_fn(x)).sum();
    let mut roll = rng.gen_range(0..total.max(1));
    for &item in items {
        let w = weight_fn(item);
        if roll < w {
            return item;
        }
        roll -= w;
    }
    items[0]
}

/// Deterministically generates `count` unique background genomes from a master seed.
pub fn generate_batch(master_seed: u64, count: u32) -> Vec<BackgroundGenome> {
    let mut batch = Vec::with_capacity(count as usize);
    let mut seen_signatures = HashSet::with_capacity(count as usize);

    for id in 1..=count {
        let mut attempt = 0u64;
        loop {
            let item_seed = master_seed
                .wrapping_mul(0x9E3779B97F4A7C15)
                .wrapping_add((id as u64) << 16)
                .wrapping_add(attempt);
            let mut rng = ChaCha8Rng::seed_from_u64(item_seed);

            let palette = sample_weighted(&mut rng, ColorPaletteTrait::ALL, |x| x.weight());
            let harmony = sample_weighted(&mut rng, HarmonyMode::ALL, |x| x.weight());
            let background = sample_weighted(&mut rng, BackgroundStyle::ALL, |x| x.weight());
            let cellular = sample_weighted(&mut rng, CellularStyle::ALL, |x| x.weight());
            let seed_distribution =
                SeedDistribution::ALL[rng.gen_range(0..SeedDistribution::ALL.len())];
            let primary_geom = sample_weighted(&mut rng, PrimaryGeometry::ALL, |x| x.weight());

            // Layer 5 sampled using the Co-occurrence Matrix given Layer 4 PrimaryGeometry
            let secondary_geom = sample_weighted(&mut rng, SecondaryGeometry::ALL, |sec| {
                SecondaryGeometry::co_occurrence_weight(primary_geom, sec)
            });
            let co_occurrence_affinity =
                SecondaryGeometry::co_occurrence_weight(primary_geom, secondary_geom);

            let proximity = sample_weighted(&mut rng, ProximityStyle::ALL, |x| x.weight());
            let glitch = sample_weighted(&mut rng, GlitchStyle::ALL, |x| x.weight());

            let cell_count = match cellular {
                CellularStyle::WorleyDistanceRidges => rng.gen_range(16..=32),
                CellularStyle::DualVoronoiDelaunayLattice => rng.gen_range(24..=48),
                _ => rng.gen_range(20..=56),
            };

            let lloyd_relaxation_steps = match cellular {
                CellularStyle::FracturedRelaxedShards => rng.gen_range(2..=4),
                CellularStyle::VoronoiCrystalCells => rng.gen_range(1..=3),
                _ => rng.gen_range(0..=2),
            };

            let genome = BackgroundGenome {
                id,
                seed: item_seed,
                palette,
                harmony,
                hue_jitter: rng.gen_range(-8.0..8.0),
                background,
                cellular,
                seed_distribution,
                cell_count,
                lloyd_relaxation_steps,
                primary_geom,
                geom_density: rng.gen_range(0.42..0.82),
                geom_scale: rng.gen_range(0.55..0.92),
                secondary_geom,
                co_occurrence_affinity,
                proximity,
                proximity_threshold: rng.gen_range(0.18..0.32),
                glitch,
                glitch_intensity: match glitch {
                    GlitchStyle::PristineClean => 0.0,
                    _ => rng.gen_range(0.28..0.85),
                },
            };

            let sig = genome.signature();
            if seen_signatures.insert(sig) {
                batch.push(genome);
                break;
            }
            attempt += 1;
        }
    }

    batch
}
