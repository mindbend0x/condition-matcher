//! Example demonstrating parallel filtering with a large number of matchers.
//!
//! This simulates a real-world scenario where users have defined "watchers" -
//! custom rules to monitor asset price changes. When new asset metrics arrive,
//! we need to efficiently find which watchers are triggered.
//!
//! - 1,000 assets with various metrics
//! - 100,000 watchers with different conditions
//! - Parallel matching using rayon

use std::any::Any;
use std::time::Instant;

use condition_matcher::{JsonMatcher, Matchable, Matcher};
use rand::prelude::*;

/// Asset metrics containing price, volume, and technical indicators.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetMetrics {
    pub asset: String,
    pub current_price: f64,
    pub current_volume: Option<f64>,
    pub pct_change_1h: Option<f64>,
    pub pct_change_24h: Option<f64>,
    pub pct_change_7d: Option<f64>,
    pub volume_multiplier: Option<f64>,
    pub volatility_24h: Option<f64>,
    pub above_sma_7d: Option<bool>,
    pub above_sma_50d: Option<bool>,
    pub above_sma_200d: Option<bool>,
}

impl Matchable for AssetMetrics {
    fn get_field(&self, name: &str) -> Option<&dyn Any> {
        match name {
            "asset" => Some(&self.asset),
            "current_price" => Some(&self.current_price),
            "current_volume" => self.current_volume.as_ref().map(|v| v as &dyn Any),
            "pct_change_1h" => self.pct_change_1h.as_ref().map(|v| v as &dyn Any),
            "pct_change_24h" => self.pct_change_24h.as_ref().map(|v| v as &dyn Any),
            "pct_change_7d" => self.pct_change_7d.as_ref().map(|v| v as &dyn Any),
            "volume_multiplier" => self.volume_multiplier.as_ref().map(|v| v as &dyn Any),
            "volatility_24h" => self.volatility_24h.as_ref().map(|v| v as &dyn Any),
            "above_sma_7d" => self.above_sma_7d.as_ref().map(|v| v as &dyn Any),
            "above_sma_50d" => self.above_sma_50d.as_ref().map(|v| v as &dyn Any),
            "above_sma_200d" => self.above_sma_200d.as_ref().map(|v| v as &dyn Any),
            _ => None,
        }
    }

    fn get_field_path(&self, path: &[&str]) -> Option<&dyn Any> {
        if path.len() == 1 {
            self.get_field(path[0])
        } else {
            None
        }
    }
}

/// A user's watcher - defines conditions to monitor.
#[derive(Debug)]
pub struct Watcher {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub matcher: JsonMatcher,
}

/// Generate random asset metrics for testing.
fn generate_assets(count: usize, rng: &mut StdRng) -> Vec<AssetMetrics> {
    let symbols: Vec<String> = (0..count)
        .map(|i| format!("ASSET{:04}", i))
        .collect();

    symbols
        .into_iter()
        .map(|symbol| {
            let base_price = rng.gen_range(0.01..100000.0);
            let pct_1h = rng.gen_range(-10.0..10.0);
            let pct_24h = rng.gen_range(-25.0..50.0);
            let pct_7d = rng.gen_range(-40.0..100.0);
            
            AssetMetrics {
                asset: symbol,
                current_price: base_price,
                current_volume: Some(rng.gen_range(1_000_000.0..10_000_000_000.0)),
                pct_change_1h: Some(pct_1h),
                pct_change_24h: Some(pct_24h),
                pct_change_7d: Some(pct_7d),
                volume_multiplier: Some(rng.gen_range(0.5..3.0)),
                volatility_24h: Some(rng.gen_range(1.0..20.0)),
                above_sma_7d: Some(rng.gen_bool(0.6)),
                above_sma_50d: Some(rng.gen_bool(0.5)),
                above_sma_200d: Some(rng.gen_bool(0.4)),
            }
        })
        .collect()
}

/// Generate diverse watcher conditions that will match different subsets of assets.
fn generate_watchers(count: usize, rng: &mut StdRng) -> Vec<Watcher> {
    let condition_templates = vec![
        // Price movement watchers
        |rng: &mut StdRng| {
            let threshold = rng.gen_range(5.0..30.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "pct_change_24h", "operator": "greater_than", "value": {}}}
                ]
            }}"#, threshold)
        },
        |rng: &mut StdRng| {
            let threshold = rng.gen_range(-30.0..-5.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "pct_change_24h", "operator": "less_than", "value": {}}}
                ]
            }}"#, threshold)
        },
        // Volume spike watchers
        |rng: &mut StdRng| {
            let vol_mult = rng.gen_range(1.5..2.5);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "volume_multiplier", "operator": "greater_than", "value": {}}}
                ]
            }}"#, vol_mult)
        },
        // Volatility watchers
        |rng: &mut StdRng| {
            let vol = rng.gen_range(5.0..15.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "volatility_24h", "operator": "greater_than", "value": {}}}
                ]
            }}"#, vol)
        },
        // Price + volume combo
        |rng: &mut StdRng| {
            let pct = rng.gen_range(3.0..15.0);
            let vol = rng.gen_range(1.2..2.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "volume_multiplier", "operator": "greater_than", "value": {}}}
                ]
            }}"#, pct, vol)
        },
        // SMA-based trend watchers
        |_rng: &mut StdRng| {
            r#"{
                "mode": "AND",
                "rules": [
                    {"field": "above_sma_7d", "operator": "equals", "value": true},
                    {"field": "above_sma_50d", "operator": "equals", "value": true}
                ]
            }"#.to_string()
        },
        // Golden cross potential (above 50, below 200)
        |_rng: &mut StdRng| {
            r#"{
                "mode": "AND",
                "rules": [
                    {"field": "above_sma_50d", "operator": "equals", "value": true},
                    {"field": "above_sma_200d", "operator": "equals", "value": false}
                ]
            }"#.to_string()
        },
        // Death cross warning (below 50, above 200)
        |_rng: &mut StdRng| {
            r#"{
                "mode": "AND",
                "rules": [
                    {"field": "above_sma_50d", "operator": "equals", "value": false},
                    {"field": "above_sma_200d", "operator": "equals", "value": true}
                ]
            }"#.to_string()
        },
        // Breakout watchers (price + weekly momentum)
        |rng: &mut StdRng| {
            let pct_24h = rng.gen_range(5.0..20.0);
            let pct_7d = rng.gen_range(15.0..50.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "pct_change_7d", "operator": "greater_than", "value": {}}}
                ]
            }}"#, pct_24h, pct_7d)
        },
        // Price range watchers
        |rng: &mut StdRng| {
            let min_price = rng.gen_range(1.0..1000.0);
            let max_price = min_price * rng.gen_range(2.0..10.0);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "current_price", "operator": "greater_than", "value": {}}},
                    {{"field": "current_price", "operator": "less_than", "value": {}}}
                ]
            }}"#, min_price, max_price)
        },
        // Complex nested: pump OR (high volatility AND above SMA)
        |rng: &mut StdRng| {
            let pump_threshold = rng.gen_range(10.0..25.0);
            let vol_threshold = rng.gen_range(8.0..15.0);
            format!(r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "pct_change_24h", "operator": "greater_than", "value": {}}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "volatility_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "above_sma_7d", "operator": "equals", "value": true}}
                        ]
                    }}
                ]
            }}"#, pump_threshold, vol_threshold)
        },
        // Dump with high volume (potential capitulation)
        |rng: &mut StdRng| {
            let dump_threshold = rng.gen_range(-25.0..-10.0);
            let vol_mult = rng.gen_range(1.5..2.5);
            format!(r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "volume_multiplier", "operator": "greater_than", "value": {}}}
                ]
            }}"#, dump_threshold, vol_mult)
        },
    ];

    (0..count)
        .map(|i| {
            let template_idx = rng.gen_range(0..condition_templates.len());
            let condition_json = condition_templates[template_idx](rng);
            
            Watcher {
                id: format!("watcher_{:06}", i),
                user_id: format!("user_{:05}", rng.gen_range(0..10000)),
                name: format!("Alert Rule {}", i),
                matcher: JsonMatcher::from_json(&condition_json).unwrap(),
            }
        })
        .collect()
}

fn main() {
    println!("=== Parallel Asset Watcher Matching Example ===\n");
    
    let mut rng = StdRng::seed_from_u64(42); // Deterministic for reproducibility
    
    // Generate test data
    println!("Generating test data...");
    let start = Instant::now();
    let assets = generate_assets(1_000, &mut rng);
    println!("  Generated {} assets in {:?}", assets.len(), start.elapsed());
    
    let start = Instant::now();
    let watchers = generate_watchers(100_000, &mut rng);
    println!("  Generated {} watchers in {:?}", watchers.len(), start.elapsed());
    
    // Show some sample data
    println!("\nSample assets:");
    for asset in assets.iter().take(5) {
        println!(
            "  {} - Price: ${:.2}, 24h: {:.1}%, Vol mult: {:.2}x",
            asset.asset,
            asset.current_price,
            asset.pct_change_24h.unwrap_or(0.0),
            asset.volume_multiplier.unwrap_or(1.0)
        );
    }
    
    println!("\nSample watchers:");
    for watcher in watchers.iter().take(3) {
        println!("  {} ({}): {:?}", watcher.id, watcher.user_id, watcher.matcher.condition());
    }
    
    // Benchmark: Find all triggered watchers for each asset
    println!("\n--- Benchmark: Match each asset against all watchers ---");
    
    // Sequential matching
    println!("\nSequential matching...");
    let start = Instant::now();
    let mut total_matches_seq = 0usize;
    for asset in &assets {
        let matches: Vec<_> = watchers.iter().filter(|w| w.matcher.matches(asset)).collect();
        total_matches_seq += matches.len();
    }
    let seq_duration = start.elapsed();
    println!(
        "  Sequential: {} total matches in {:?} ({:.2} assets/sec)",
        total_matches_seq,
        seq_duration,
        assets.len() as f64 / seq_duration.as_secs_f64()
    );
    
    // Parallel matching using filter_par
    println!("\nParallel matching (using filter_par on watchers)...");
    let start = Instant::now();
    let mut total_matches_par = 0usize;
    for asset in &assets {
        // For each asset, find watchers that match in parallel
        let matches: Vec<_> = watchers
            .iter()
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|w| w.matcher.matches(asset))
            .collect();
        total_matches_par += matches.len();
    }
    let par_duration = start.elapsed();
    println!(
        "  Parallel (per-asset): {} total matches in {:?} ({:.2} assets/sec)",
        total_matches_par,
        par_duration,
        assets.len() as f64 / par_duration.as_secs_f64()
    );
    
    // Parallel matching using rayon on outer loop
    println!("\nParallel matching (using rayon on asset loop)...");
    use rayon::prelude::*;
    let start = Instant::now();
    let total_matches_rayon: usize = assets
        .par_iter()
        .map(|asset| {
            watchers.iter().filter(|w| w.matcher.matches(asset)).count()
        })
        .sum();
    let rayon_duration = start.elapsed();
    println!(
        "  Parallel (rayon): {} total matches in {:?} ({:.2} assets/sec)",
        total_matches_rayon,
        rayon_duration,
        assets.len() as f64 / rayon_duration.as_secs_f64()
    );
    
    // Verify consistency
    assert_eq!(total_matches_seq, total_matches_par);
    assert_eq!(total_matches_seq, total_matches_rayon);
    println!("\n✓ All methods produced consistent results");
    
    // Calculate speedup
    let speedup = seq_duration.as_secs_f64() / rayon_duration.as_secs_f64();
    println!("  Speedup: {:.2}x", speedup);
    
    // Detailed statistics
    println!("\n--- Match Statistics ---");
    let matches_per_asset: Vec<usize> = assets
        .par_iter()
        .map(|asset| watchers.iter().filter(|w| w.matcher.matches(asset)).count())
        .collect();
    
    let avg_matches = matches_per_asset.iter().sum::<usize>() as f64 / assets.len() as f64;
    let max_matches = *matches_per_asset.iter().max().unwrap_or(&0);
    let min_matches = *matches_per_asset.iter().min().unwrap_or(&0);
    
    println!("  Average watchers triggered per asset: {:.1}", avg_matches);
    println!("  Max watchers triggered for an asset: {}", max_matches);
    println!("  Min watchers triggered for an asset: {}", min_matches);
    
    // Find the most active asset
    let most_active_idx = matches_per_asset
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let most_active = &assets[most_active_idx];
    println!(
        "\n  Most active asset: {} ({} watchers triggered)",
        most_active.asset, matches_per_asset[most_active_idx]
    );
    println!(
        "    Price: ${:.2}, 24h: {:.1}%, 7d: {:.1}%, Vol mult: {:.2}x, Vol24h: {:.1}%",
        most_active.current_price,
        most_active.pct_change_24h.unwrap_or(0.0),
        most_active.pct_change_7d.unwrap_or(0.0),
        most_active.volume_multiplier.unwrap_or(1.0),
        most_active.volatility_24h.unwrap_or(0.0)
    );
    
    // Find watchers for a specific asset
    println!("\n--- Sample: Triggered watchers for {} ---", assets[0].asset);
    let sample_asset = &assets[0];
    let triggered: Vec<_> = watchers
        .iter()
        .filter(|w| w.matcher.matches(sample_asset))
        .take(5)
        .collect();
    
    println!("  Asset: {} - ${:.2}, 24h: {:.1}%", 
        sample_asset.asset, 
        sample_asset.current_price,
        sample_asset.pct_change_24h.unwrap_or(0.0)
    );
    println!("  First {} triggered watchers:", triggered.len());
    for w in triggered {
        println!("    - {} ({})", w.id, w.user_id);
    }
    
    println!("\n=== Example Complete ===");
}

