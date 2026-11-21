use crate::genome::NftGenome;
use crate::renderer::{render_nft, LayerVisibility};
use image::{ImageBuffer, Rgba};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionConfig {
    pub collection_name: String,
    pub description: String,
    pub external_url: String,
    pub images_cid: String,
    pub export_resolution: u32,
    pub output_dir: String,
}

impl Default for CollectionConfig {
    fn default() -> Self {
        Self {
            collection_name: "Neo-Art".to_string(),
            description: "7-Layer Procedural Abstract Geometry & Cellular Mathematics NFT Collection published via Octopin and Zora.".to_string(),
            external_url: "https://zora.co".to_string(),
            images_cid: "REPLACE_WITH_OCTOPIN_IMAGES_FOLDER_CID".to_string(),
            export_resolution: 2000,
            output_dir: "output".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAttribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Erc721Metadata {
    pub name: String,
    pub description: String,
    pub image: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub external_url: String,
    pub attributes: Vec<NftAttribute>,
}

pub fn build_erc721_metadata(genome: &NftGenome, config: &CollectionConfig) -> Erc721Metadata {
    let cid_clean = config
        .images_cid
        .trim()
        .trim_start_matches("ipfs://")
        .trim_end_matches('/');

    let image_uri = format!("ipfs://{}/{}.png", cid_clean, genome.token_id);

    let name = if let Some(ref title) = genome.mythic_title {
        format!("{} #{} — {}", config.collection_name, genome.token_id, title)
    } else {
        format!("{} #{}", config.collection_name, genome.token_id)
    };

    let mut attributes = vec![
        NftAttribute {
            trait_type: "Rarity Tier".to_string(),
            value: serde_json::Value::String(genome.rarity_tier.name().to_string()),
        },
        NftAttribute {
            trait_type: "Color Palette".to_string(),
            value: serde_json::Value::String(genome.palette.name().to_string()),
        },
        NftAttribute {
            trait_type: "Color Harmony".to_string(),
            value: serde_json::Value::String(genome.harmony.name().to_string()),
        },
        NftAttribute {
            trait_type: "Background".to_string(),
            value: serde_json::Value::String(genome.background.name().to_string()),
        },
        NftAttribute {
            trait_type: "Cellular Substrate".to_string(),
            value: serde_json::Value::String(genome.cellular.name().to_string()),
        },
        NftAttribute {
            trait_type: "Cellular Seed Lattice".to_string(),
            value: serde_json::Value::String(genome.seed_distribution.name().to_string()),
        },
        NftAttribute {
            trait_type: "Primary Geometry".to_string(),
            value: serde_json::Value::String(genome.primary_geom.name().to_string()),
        },
        NftAttribute {
            trait_type: "Co-occurrence Geometry".to_string(),
            value: serde_json::Value::String(genome.secondary_geom.name().to_string()),
        },
        NftAttribute {
            trait_type: "Proximity Field".to_string(),
            value: serde_json::Value::String(genome.proximity.name().to_string()),
        },
        NftAttribute {
            trait_type: "Glitch Effect".to_string(),
            value: serde_json::Value::String(genome.glitch.name().to_string()),
        },
        NftAttribute {
            trait_type: "Cellular Density".to_string(),
            value: serde_json::json!(genome.cell_count),
        },
        NftAttribute {
            trait_type: "Co-occurrence Affinity".to_string(),
            value: serde_json::json!(genome.co_occurrence_affinity),
        },
    ];

    if let Some(ref title) = genome.mythic_title {
        attributes.push(NftAttribute {
            trait_type: "Mythic Edition".to_string(),
            value: serde_json::Value::String(title.clone()),
        });
    }

    Erc721Metadata {
        name,
        description: config.description.clone(),
        image: image_uri,
        external_url: config.external_url.clone(),
        attributes,
    }
}

/// Exports a single NFT image + JSON metadata to disk
pub fn export_single_nft(
    genome: &NftGenome,
    config: &CollectionConfig,
    vis: LayerVisibility,
) -> Result<(PathBuf, PathBuf), String> {
    let base = Path::new(&config.output_dir);
    let images_dir = base.join("images");
    let metadata_dir = base.join("metadata");

    fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&metadata_dir).map_err(|e| e.to_string())?;

    let size = config.export_resolution;
    let rgba = render_nft(genome, size, vis);
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(size, size, rgba).ok_or("Failed to construct image buffer")?;

    let img_path = images_dir.join(format!("{}.png", genome.token_id));
    img.save(&img_path).map_err(|e| e.to_string())?;

    let meta = build_erc721_metadata(genome, config);
    let json_str = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
    let meta_path = metadata_dir.join(format!("{}.json", genome.token_id));
    fs::write(&meta_path, json_str).map_err(|e| e.to_string())?;

    Ok((img_path, meta_path))
}

/// Multi-threaded batch export of all 250 NFTs (PNGs + ERC-721 JSONs + collection manifest)
pub fn export_batch_collection(
    collection: &[NftGenome],
    config: &CollectionConfig,
    progress_counter: Arc<AtomicUsize>,
) -> Result<(), String> {
    let base = Path::new(&config.output_dir);
    let images_dir = base.join("images");
    let metadata_dir = base.join("metadata");

    fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&metadata_dir).map_err(|e| e.to_string())?;

    let vis = LayerVisibility::default();

    collection
        .par_iter()
        .try_for_each(|genome| -> Result<(), String> {
            export_single_nft(genome, config, vis)?;
            progress_counter.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })?;

    // Write full collection manifest
    let manifest_path = base.join("collection_manifest.json");
    let manifest_json = serde_json::to_string_pretty(collection).map_err(|e| e.to_string())?;
    fs::write(manifest_path, manifest_json).map_err(|e| e.to_string())?;

    Ok(())
}

/// Fast Octopin CID stamper: updates all 250 metadata JSON files with `ipfs://<IMAGES_CID>/<id>.png`
/// without needing to re-render the 2000x2000 PNGs.
pub fn stamp_octopin_cid_on_metadata(
    collection: &[NftGenome],
    config: &CollectionConfig,
) -> Result<usize, String> {
    let base = Path::new(&config.output_dir);
    let metadata_dir = base.join("metadata");
    fs::create_dir_all(&metadata_dir).map_err(|e| e.to_string())?;

    for genome in collection {
        let meta = build_erc721_metadata(genome, config);
        let json_str = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;
        let meta_path = metadata_dir.join(format!("{}.json", genome.token_id));
        fs::write(&meta_path, json_str).map_err(|e| e.to_string())?;
    }

    Ok(collection.len())
}
