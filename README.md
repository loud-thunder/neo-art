# Neo-Art — 7-Layer Procedural Abstract NFT Generator

A native Rust procedural abstract art NFT generator and interactive `egui` preview studio built for generating, inspecting, and publishing a 250-NFT collection to **[Octopin](https://octopin.app/)** and **[Zora](https://zora.co/)**.

## 7-Layer Procedural Pipeline

1. **Layer 1: Color System (`OKLCH` Curated Palettes & Harmonies)** — 14 curated algorithmic palettes resolved in perceptual OKLCH space across 5 harmonic modes (*Complementary*, *Triadic*, *Split-Complementary*, *Analogous*, *Monochrome Accent*).
2. **Layer 2: Background Field** — *Deep Void*, *Radial Nebula Gradient*, *Linear Horizon*, *Micro Blueprint Grid*, *Quantized Noise Field*, *Dual-Tone Diagonal*, and *Concentric Pulse*.
3. **Layer 3: Cellular Math Substrate** — Exact 2D Voronoi tessellations (with configurable Lloyd's relaxation and 4 seed lattice distributions), dual Delaunay triangulation meshes, and Worley $F_2 - F_1$ distance ridge fields.
4. **Layer 4: Primary Geometry** — Cell-anchored *Concentric Rings*, *Inscribed Regular Polygons*, *Bauhaus Rectilinear Blocks*, *Isometric Prisms*, *Radial Vector Bursts*, *Sacred Triangles*, and *Harmonic Segmented Arcs*.
5. **Layer 5: Co-occurrence Secondary Geometry** — Governed by an explicit **Co-occurrence Compatibility Matrix** enforcing affinity and repulsion rules between primary and secondary geometric primitives (*Orbital Micro-Dots*, *Nested Sub-Fractals*, *Precision Crosshair Markers*, *Parallel Vector Hatching*, *Golden Spiral Satellites*, *Voronoi Vertex Satellites*).
6. **Layer 6: Spatial Proximity Connections & Fields** — Euclidean distance-threshold laser bridges, *k-NN Constellation Webs*, *Gabriel Graph Networks*, *Harmonic Interference Ripples*, *Magnetic Force Contours*, and *Spatial Gravity Wells*.
7. **Layer 7: Effects & Glitch** — Resolution-independent luminance-threshold pixel sorting (horizontal & vertical) and scanline slice displacement.

## Running the Interactive GUI Studio

```bash
cargo run --release
```

### Headless Batch Export (All 250 NFTs at 2000x2000 PNG + ERC-721 JSON)

```bash
cargo run --release -- --export-batch
```

## Octopin & Zora Publishing Workflow

1. **Export Collection**: In the GUI (`4. Octopin & Zora Export` tab) or via `--export-batch`, render all 250 NFTs to `output/images/1.png .. 250.png` and `output/metadata/1.json .. 250.json`.
2. **Pin Images Folder on Octopin**: Upload `output/images/` to [https://octopin.app/](https://octopin.app/) and copy the returned folder CID (`bafy...`).
3. **Stamp CID into Metadata**: Paste the Images Folder CID into the GUI's **Octopin Images Folder CID** field and click **Stamp Octopin Images CID into all 250 Metadata JSONs** (updates all 250 JSON files with `"image": "ipfs://<IMAGES_CID>/<id>.png"`).
4. **Pin Metadata Folder on Octopin & Deploy on Zora**: Upload `output/metadata/` to [https://octopin.app/](https://octopin.app/), copy the Metadata Folder CID (`<METADATA_CID>`), and set `ipfs://<METADATA_CID>/` as your Metadata Base URI on [https://zora.co/](https://zora.co/).
