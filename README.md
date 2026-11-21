# Neo-Art — 7-Layer Procedural Abstract Background Studio

A high-performance Rust procedural abstract wallpaper and background generator featuring an interactive `egui` studio, adjustable aspect ratios, custom resolutions, and multi-threaded headless batch export.

## Features

- **7-Layer Procedural Pipeline**:
  1. **Layer 1: Color System (`OKLCH` Harmonies)** — 14 algorithmic palettes resolved in perceptual OKLCH color space across 5 harmonic modes (*Complementary*, *Triadic*, *Split-Complementary*, *Analogous*, *Monochrome Accent*).
  2. **Layer 2: Background Field** — *Deep Void*, *Radial Nebula Gradient*, *Linear Horizon*, *Micro Blueprint Grid*, *Quantized Noise Field*, *Dual-Tone Diagonal*, and *Concentric Pulse*.
  3. **Layer 3: Cellular Math Substrate** — Exact 2D Voronoi tessellations (with Lloyd's relaxation and 4 seed lattice distributions: Jittered Grid, Fibonacci Spiral, Clustered Nuclei, Harmonic Ring), dual Delaunay triangulation meshes, and Worley $F_2 - F_1$ distance ridge fields.
  4. **Layer 4: Primary Geometry** — Cell-anchored *Concentric Rings*, *Inscribed Regular Polygons*, *Bauhaus Rectilinear Blocks*, *Isometric Prisms*, *Radial Vector Bursts*, *Sacred Triangles*, and *Harmonic Segmented Arcs*.
  5. **Layer 5: Co-occurrence Secondary Geometry** — Governed by an explicit **Co-occurrence Compatibility Matrix** enforcing affinity and repulsion rules between primary and secondary geometric primitives (*Orbital Micro-Dots*, *Nested Sub-Fractals*, *Precision Crosshair Markers*, *Parallel Vector Hatching*, *Golden Spiral Satellites*, *Voronoi Vertex Satellites*).
  6. **Layer 6: Spatial Proximity Connections & Fields** — Euclidean distance-threshold laser bridges, *k-NN Constellation Webs*, *Gabriel Graph Networks*, *Harmonic Interference Ripples*, *Magnetic Force Contours*, and *Spatial Gravity Wells*.
  7. **Layer 7: Effects & Glitch** — Resolution-independent luminance-threshold pixel sorting (horizontal & vertical) and scanline slice displacement.
- **Isotropic Aspect Ratio Engine**: Geometry adapts seamlessly to any ratio without stretching or squishing (circles stay true circles across 16:9, 9:16, 21:9, 1:1, 4:3).
- **Interactive GUI Studio**:
  - **1. Background Canvas**: Live interactive viewport adapting cleanly to the selected aspect ratio.
  - **2. Batch Gallery**: Browse, preview, and select from generated batches.
  - **3. Batch Export**: Multi-threaded parallel PNG rendering with live progress tracking.
- **Canvas & Export Inspector**: Fast preset switching (4K, 1080p, Ultrawide, Mobile), custom Width/Height inputs, Quick Single Export button, and OKLCH swatches.

## Running the Interactive GUI Studio

```bash
cargo run --release
```

## Headless CLI Batch Export

Export batches of backgrounds at any resolution or aspect ratio:

```bash
# Export 50 4K UHD backgrounds (16:9 3840x2160)
cargo run --release -- --export-batch --count 50 --aspect 4k

# Export 20 1080p Full HD backgrounds (16:9 1920x1080)
cargo run --release -- --export-batch --count 20 --aspect 1080p

# Export 10 Mobile Story wallpapers (9:16 1080x1920)
cargo run --release -- --export-batch --count 10 --aspect 9:16 --output-dir output/mobile

# Export with custom dimensions and prefix
cargo run --release -- --export-batch --count 15 --width 3440 --height 1440 --prefix ultrawide_bg
```
