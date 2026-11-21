mod app;
mod cellular;
mod export;
mod genome;
mod glitch;
mod renderer;

use app::NeoArtApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Support optional CLI headless batch export (`cargo run --release -- --export-batch`)
    if args.iter().any(|a| a == "--export-batch") {
        let seed = 2026_11_21;
        let supply = 250;
        println!("Generating {} unique 7-layer NFT genomes (seed={})...", supply, seed);
        let collection = genome::generate_collection(seed, supply);
        let config = export::CollectionConfig::default();
        let progress = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        println!(
            "Rendering {} NFTs at {}x{} to {}/...",
            supply, config.export_resolution, config.export_resolution, config.output_dir
        );
        export::export_batch_collection(&collection, &config, progress)
            .expect("Batch export succeeded");
        println!("Done! Exported {} PNGs and ERC-721 JSON files.", supply);
        return Ok(());
    }

    // Support optional CLI headless sample export (`cargo run -- --export-samples`)
    if args.iter().any(|a| a == "--export-samples") {
        let seed = 2026_11_21;
        let collection = genome::generate_collection(seed, 250);
        let config = export::CollectionConfig {
            export_resolution: 1024,
            ..Default::default()
        };
        let vis = renderer::LayerVisibility::default();
        for &id in &[1usize, 42, 125, 250] {
            let (img_p, meta_p) =
                export::export_single_nft(&collection[id - 1], &config, vis).unwrap();
            println!("Exported sample #{} -> {} & {}", id, img_p.display(), meta_p.display());
        }
        return Ok(());
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1420.0, 900.0])
            .with_min_inner_size([1024.0, 680.0])
            .with_title("Neo-Art — 7-Layer Procedural Abstract NFT Generator (Octopin & Zora)"),
        ..Default::default()
    };

    eframe::run_native(
        "Neo-Art NFT Studio",
        native_options,
        Box::new(|cc| Ok(Box::new(NeoArtApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests {
    use crate::export::{build_erc721_metadata, stamp_octopin_cid_on_metadata, CollectionConfig};
    use crate::genome::{generate_collection, SecondaryGeometry};
    use crate::renderer::{render_nft, LayerVisibility};
    use std::collections::HashSet;

    #[test]
    fn test_250_collection_uniqueness_and_co_occurrence_rules() {
        let collection = generate_collection(2026_11_21, 250);
        assert_eq!(collection.len(), 250);

        let mut signatures = HashSet::new();
        for g in &collection {
            assert!(
                signatures.insert(g.signature()),
                "Duplicate trait signature detected for token #{}",
                g.token_id
            );

            // Verify Co-occurrence repulsion rules are strictly respected
            let affinity =
                SecondaryGeometry::co_occurrence_weight(g.primary_geom, g.secondary_geom);
            assert!(
                affinity > 0,
                "Forbidden co-occurrence pairing ({:?}, {:?}) on token #{}",
                g.primary_geom,
                g.secondary_geom,
                g.token_id
            );
            assert_eq!(affinity, g.co_occurrence_affinity);
        }
    }

    #[test]
    fn test_deterministic_7_layer_rendering() {
        let collection = generate_collection(999_888_777, 10);
        let genome = &collection[3];
        let vis = LayerVisibility::default();

        let render_a = render_nft(genome, 256, vis);
        let render_b = render_nft(genome, 256, vis);

        assert_eq!(render_a.len(), 256 * 256 * 4);
        assert_eq!(
            render_a, render_b,
            "Procedural render must be 100% deterministic for identical seed and genome"
        );
    }

    #[test]
    fn test_octopin_zora_metadata_and_cid_stamping() {
        let collection = generate_collection(42, 5);
        let tmp_dir = std::env::temp_dir().join("neo_art_test_metadata");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        let config = CollectionConfig {
            collection_name: "Neo-Art".to_string(),
            description: "Abstract Cellular Math Collection".to_string(),
            external_url: "https://zora.co".to_string(),
            images_cid: "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi".to_string(),
            export_resolution: 512,
            output_dir: tmp_dir.to_string_lossy().to_string(),
        };

        let meta1 = build_erc721_metadata(&collection[0], &config);
        assert_eq!(
            meta1.image,
            "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi/1.png"
        );
        assert!(meta1.attributes.len() >= 12);

        let count = stamp_octopin_cid_on_metadata(&collection, &config).unwrap();
        assert_eq!(count, 5);

        let file_content = std::fs::read_to_string(tmp_dir.join("metadata").join("1.json")).unwrap();
        assert!(file_content.contains(
            "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi/1.png"
        ));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}
