//! Example demonstrating parallel filtering with a cached metrics store.
//!
//! This builds on the parallel_asset_watchers example but uses a single
//! cache struct that holds all AssetMetrics in a HashMap. Each watcher
//! specifies which asset it monitors using dot notation: "ASSET_ID.field"
//!
//! This models a real-world scenario where:
//! - A central cache holds metrics for all tracked assets
//! - Each user's watcher targets a specific asset by ID
//! - When metrics update, we check which watchers are triggered
//!
//! - 1,000 assets in a single cache
//! - 100,000 watchers targeting specific assets
//! - Parallel matching using rayon

use std::any::Any;
use std::collections::HashMap;
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

/// A cache holding metrics for all assets, keyed by asset symbol.
///
/// Implements `Matchable` to allow JSON conditions to query metrics
/// for a specific asset using dot notation like "BTC.current_price".
#[derive(Debug, Clone, PartialEq)]
pub struct MetricsCache {
    pub metrics: HashMap<String, AssetMetrics>,
}

impl MetricsCache {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
        }
    }

    pub fn insert(&mut self, metrics: AssetMetrics) {
        self.metrics.insert(metrics.asset.clone(), metrics);
    }

    pub fn get(&self, asset: &str) -> Option<&AssetMetrics> {
        self.metrics.get(asset)
    }

    pub fn len(&self) -> usize {
        self.metrics.len()
    }
}

impl Matchable for MetricsCache {
    fn get_field(&self, _name: &str) -> Option<&dyn Any> {
        // Single field access not supported - use get_field_path with "ASSET.field"
        None
    }

    fn get_field_path(&self, path: &[&str]) -> Option<&dyn Any> {
        if path.len() < 2 {
            return None;
        }

        let asset_key = path[0];
        let field_name = path[1];

        let metrics = self.metrics.get(asset_key)?;

        match field_name {
            "asset" => Some(&metrics.asset),
            "current_price" => Some(&metrics.current_price),
            "current_volume" => metrics.current_volume.as_ref().map(|v| v as &dyn Any),
            "pct_change_1h" => metrics.pct_change_1h.as_ref().map(|v| v as &dyn Any),
            "pct_change_24h" => metrics.pct_change_24h.as_ref().map(|v| v as &dyn Any),
            "pct_change_7d" => metrics.pct_change_7d.as_ref().map(|v| v as &dyn Any),
            "volume_multiplier" => metrics.volume_multiplier.as_ref().map(|v| v as &dyn Any),
            "volatility_24h" => metrics.volatility_24h.as_ref().map(|v| v as &dyn Any),
            "above_sma_7d" => metrics.above_sma_7d.as_ref().map(|v| v as &dyn Any),
            "above_sma_50d" => metrics.above_sma_50d.as_ref().map(|v| v as &dyn Any),
            "above_sma_200d" => metrics.above_sma_200d.as_ref().map(|v| v as &dyn Any),
            _ => None,
        }
    }
}

/// A user's watcher - defines conditions to monitor for one or more assets.
#[derive(Debug)]
pub struct Watcher {
    pub id: String,
    pub user_id: String,
    pub asset_ids: Vec<String>, // Can watch multiple assets
    pub name: String,
    pub matcher: JsonMatcher,
}

/// Generate random asset metrics for testing.
fn generate_cache(count: usize, rng: &mut StdRng) -> (MetricsCache, Vec<String>) {
    let mut cache = MetricsCache::new();
    let mut asset_ids = Vec::with_capacity(count);

    for i in 0..count {
        let asset_id = format!("ASSET{:04}", i);
        let base_price = rng.gen_range(0.01..100000.0);
        let pct_1h = rng.gen_range(-10.0..10.0);
        let pct_24h = rng.gen_range(-25.0..50.0);
        let pct_7d = rng.gen_range(-40.0..100.0);

        let metrics = AssetMetrics {
            asset: asset_id.clone(),
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
        };

        cache.insert(metrics);
        asset_ids.push(asset_id);
    }

    (cache, asset_ids)
}

/// Generate diverse watcher conditions that target specific assets.
/// Each watcher uses "ASSET_ID.field" notation in its conditions.
/// These are complex matchers with multiple rules and nested conditions.
fn generate_watchers(count: usize, asset_ids: &[String], rng: &mut StdRng) -> Vec<Watcher> {
    // Complex condition templates with multiple rules and nested conditions
    let condition_templates: Vec<Box<dyn Fn(&str, &mut StdRng) -> String>> = vec![
        // 1. Bullish momentum: price up + volume spike + above short-term SMA
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let pct_24h = rng.gen_range(5.0..20.0);
            let pct_7d = rng.gen_range(10.0..40.0);
            let vol_mult = rng.gen_range(1.3..2.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}}
                ]
            }}"#,
                asset_id, pct_24h, asset_id, pct_7d, asset_id, vol_mult, asset_id
            )
        }),
        // 2. Bearish reversal warning: price dropping + high volatility + below SMAs
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let pct_24h = rng.gen_range(-25.0..-5.0);
            let volatility = rng.gen_range(8.0..15.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.volatility_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": false}},
                    {{"field": "{}.above_sma_200d", "operator": "equals", "value": false}}
                ]
            }}"#,
                asset_id, pct_24h, asset_id, volatility, asset_id, asset_id
            )
        }),
        // 3. Nested: Strong breakout OR (accumulation phase with low volatility)
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let breakout_pct = rng.gen_range(15.0..30.0);
            let breakout_vol = rng.gen_range(1.8..2.5);
            let low_vol = rng.gen_range(2.0..5.0);
            format!(
                r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.volatility_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                            {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}}
                        ]
                    }}
                ]
            }}"#,
                asset_id, breakout_pct, asset_id, breakout_vol, asset_id,
                asset_id, low_vol, asset_id, asset_id
            )
        }),
        // 4. Triple nested: (Pump AND volume) OR (Dip AND oversold) OR (Stable AND bullish trend)
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let pump_pct = rng.gen_range(10.0..25.0);
            let pump_vol = rng.gen_range(1.5..2.2);
            let dip_pct = rng.gen_range(-20.0..-8.0);
            let stable_vol = rng.gen_range(1.0..4.0);
            format!(
                r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.pct_change_1h", "operator": "greater_than", "value": 1.0}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}},
                            {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.2}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.volatility_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                            {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                            {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}}
                        ]
                    }}
                ]
            }}"#,
                asset_id, pump_pct, asset_id, pump_vol, asset_id,
                asset_id, dip_pct, asset_id, asset_id,
                asset_id, stable_vol, asset_id, asset_id, asset_id
            )
        }),
        // 5. Golden cross setup: above 50 SMA + approaching 200 SMA + momentum building
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let pct_7d = rng.gen_range(5.0..20.0);
            let pct_24h = rng.gen_range(2.0..10.0);
            let vol_mult = rng.gen_range(1.1..1.6);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_200d", "operator": "equals", "value": false}},
                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}}
                ]
            }}"#,
                asset_id, asset_id, asset_id, pct_7d, asset_id, pct_24h, asset_id, vol_mult
            )
        }),
        // 6. Deep nested: ((High vol AND pump) OR (Low vol AND stable)) AND above all SMAs
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let high_vol = rng.gen_range(10.0..18.0);
            let pump_pct = rng.gen_range(8.0..20.0);
            let low_vol = rng.gen_range(1.0..4.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}}
                ],
                "nested": [
                    {{
                        "mode": "OR",
                        "nested": [
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.volatility_24h", "operator": "greater_than", "value": {}}},
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}}
                                ]
                            }},
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.volatility_24h", "operator": "less_than", "value": {}}},
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": -2.0}},
                                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": 5.0}}
                                ]
                            }}
                        ]
                    }}
                ]
            }}"#,
                asset_id, asset_id, asset_id,
                asset_id, high_vol, asset_id, pump_pct,
                asset_id, low_vol, asset_id, asset_id
            )
        }),
        // 7. Capitulation signal: sharp drop + extreme volume + below all SMAs
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let drop_24h = rng.gen_range(-30.0..-15.0);
            let drop_7d = rng.gen_range(-40.0..-20.0);
            let extreme_vol = rng.gen_range(2.0..3.0);
            let high_volatility = rng.gen_range(12.0..20.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.pct_change_7d", "operator": "less_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volatility_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": false}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": false}}
                ]
            }}"#,
                asset_id, drop_24h, asset_id, drop_7d, asset_id, extreme_vol,
                asset_id, high_volatility, asset_id, asset_id
            )
        }),
        // 8. Nested with price range: (price in range AND bullish) OR (breakout from range)
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let min_price = rng.gen_range(100.0..10000.0);
            let max_price = min_price * rng.gen_range(1.5..3.0);
            let breakout_pct = rng.gen_range(10.0..20.0);
            format!(
                r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.current_price", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.current_price", "operator": "less_than", "value": {}}},
                            {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": 0}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.current_price", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.5}}
                        ]
                    }}
                ]
            }}"#,
                asset_id, min_price, asset_id, max_price, asset_id, asset_id,
                asset_id, max_price, asset_id, breakout_pct, asset_id
            )
        }),
        // 9. Multi-timeframe confirmation: 1h + 24h + 7d all positive with volume
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let pct_1h = rng.gen_range(0.5..3.0);
            let pct_24h = rng.gen_range(3.0..15.0);
            let pct_7d = rng.gen_range(8.0..30.0);
            let vol_mult = rng.gen_range(1.2..1.8);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_1h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}}
                ]
            }}"#,
                asset_id, pct_1h, asset_id, pct_24h, asset_id, pct_7d, asset_id, vol_mult, asset_id
            )
        }),
        // 10. Complex divergence: price up but volume down OR price down but holding above SMA
        Box::new(|asset_id: &str, rng: &mut StdRng| {
            let up_pct = rng.gen_range(5.0..15.0);
            let low_vol = rng.gen_range(0.5..0.9);
            let down_pct = rng.gen_range(-15.0..-3.0);
            format!(
                r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.volume_multiplier", "operator": "less_than", "value": {}}},
                            {{"field": "{}.volatility_24h", "operator": "less_than", "value": 5.0}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                            {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}},
                            {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 0}}
                        ]
                    }}
                ]
            }}"#,
                asset_id, up_pct, asset_id, low_vol, asset_id,
                asset_id, down_pct, asset_id, asset_id, asset_id
            )
        }),
    ];

    // 2-asset condition templates
    let two_asset_templates: Vec<Box<dyn Fn(&str, &str, &mut StdRng) -> String>> = vec![
        // 11. Correlation check: both assets pumping together
        Box::new(|a1: &str, a2: &str, rng: &mut StdRng| {
            let pct1 = rng.gen_range(5.0..15.0);
            let pct2 = rng.gen_range(5.0..15.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}}
                ]
            }}"#,
                a1, pct1, a2, pct2, a1, a2
            )
        }),
        // 12. Divergence alert: one asset up, another down (potential rotation)
        Box::new(|a1: &str, a2: &str, rng: &mut StdRng| {
            let up_pct = rng.gen_range(5.0..20.0);
            let down_pct = rng.gen_range(-20.0..-5.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.2}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.2}}
                ]
            }}"#,
                a1, up_pct, a2, down_pct, a1, a2
            )
        }),
        // 13. Nested multi-asset: (Asset1 pumping AND Asset2 stable) OR (both dumping)
        Box::new(|a1: &str, a2: &str, rng: &mut StdRng| {
            let pump_pct = rng.gen_range(10.0..25.0);
            let dump_pct = rng.gen_range(-25.0..-10.0);
            format!(
                r#"{{
                "mode": "OR",
                "nested": [
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                            {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": -3.0}},
                            {{"field": "{}.pct_change_24h", "operator": "less_than", "value": 3.0}},
                            {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.3}}
                        ]
                    }},
                    {{
                        "mode": "AND",
                        "rules": [
                            {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                            {{"field": "{}.volatility_24h", "operator": "greater_than", "value": 8.0}}
                        ]
                    }}
                ]
            }}"#,
                a1, pump_pct, a2, a2, a1,
                a1, dump_pct, a2, dump_pct, a1
            )
        }),
        // 14. Leader-follower: Asset1 breaking out, check if Asset2 follows
        Box::new(|a1: &str, a2: &str, rng: &mut StdRng| {
            let leader_pct = rng.gen_range(15.0..30.0);
            let follower_pct = rng.gen_range(5.0..15.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.5}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.2}}
                ]
            }}"#,
                a1, leader_pct, a2, follower_pct, a1, a2, a1, a2
            )
        }),
        // 15. Deep nested multi-asset: complex correlation check
        Box::new(|a1: &str, a2: &str, rng: &mut StdRng| {
            let threshold = rng.gen_range(8.0..18.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_200d", "operator": "equals", "value": true}}
                ],
                "nested": [
                    {{
                        "mode": "OR",
                        "nested": [
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}}
                                ]
                            }},
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 20.0}},
                                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 15.0}},
                                    {{"field": "{}.volatility_24h", "operator": "less_than", "value": 8.0}}
                                ]
                            }}
                        ]
                    }}
                ]
            }}"#,
                a1, a2, a1, threshold, a2, threshold, a1, a2, a1
            )
        }),
    ];

    // 3-asset condition templates
    let three_asset_templates: Vec<Box<dyn Fn(&str, &str, &str, &mut StdRng) -> String>> = vec![
        // 16. Sector momentum: all three assets showing strength
        Box::new(|a1: &str, a2: &str, a3: &str, rng: &mut StdRng| {
            let pct = rng.gen_range(3.0..10.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_50d", "operator": "equals", "value": true}}
                ]
            }}"#,
                a1, pct, a2, pct, a3, pct, a1, a2, a3
            )
        }),
        // 17. Risk-off signal: multiple assets dumping with high volatility
        Box::new(|a1: &str, a2: &str, a3: &str, rng: &mut StdRng| {
            let dump_pct = rng.gen_range(-20.0..-8.0);
            let high_vol = rng.gen_range(10.0..18.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": {}}},
                    {{"field": "{}.volatility_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volatility_24h", "operator": "greater_than", "value": {}}}
                ]
            }}"#,
                a1, dump_pct, a2, dump_pct, a3, dump_pct, a1, high_vol, a2, high_vol
            )
        }),
        // 18. Rotation signal: one pumping while others consolidate
        Box::new(|a1: &str, a2: &str, a3: &str, rng: &mut StdRng| {
            let pump_pct = rng.gen_range(12.0..25.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                    {{"field": "{}.volume_multiplier", "operator": "greater_than", "value": 1.8}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": -5.0}},
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": 5.0}},
                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": -5.0}},
                    {{"field": "{}.pct_change_24h", "operator": "less_than", "value": 5.0}}
                ]
            }}"#,
                a1, pump_pct, a1, a2, a2, a3, a3
            )
        }),
        // 19. Triple nested: all three bullish with varying strengths
        Box::new(|a1: &str, a2: &str, a3: &str, rng: &mut StdRng| {
            let strong_pct = rng.gen_range(15.0..30.0);
            let med_pct = rng.gen_range(8.0..15.0);
            let weak_pct = rng.gen_range(3.0..8.0);
            format!(
                r#"{{
                "mode": "AND",
                "rules": [
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}},
                    {{"field": "{}.above_sma_7d", "operator": "equals", "value": true}}
                ],
                "nested": [
                    {{
                        "mode": "OR",
                        "nested": [
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}},
                                    {{"field": "{}.pct_change_24h", "operator": "greater_than", "value": {}}}
                                ]
                            }},
                            {{
                                "mode": "AND",
                                "rules": [
                                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 20.0}},
                                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 15.0}},
                                    {{"field": "{}.pct_change_7d", "operator": "greater_than", "value": 10.0}}
                                ]
                            }}
                        ]
                    }}
                ]
            }}"#,
                a1, a2, a3,
                a1, strong_pct, a2, med_pct, a3, weak_pct,
                a1, a2, a3
            )
        }),
    ];

    (0..count)
        .map(|i| {
            // 30% of watchers use multi-asset conditions
            let use_multi_asset = rng.gen_bool(0.3);
            
            if use_multi_asset {
                // 60% use 2 assets, 40% use 3 assets
                let use_three = rng.gen_bool(0.4);
                
                if use_three {
                    // Pick 3 random distinct assets
                    let mut selected: Vec<String> = Vec::new();
                    while selected.len() < 3 {
                        let asset = asset_ids[rng.gen_range(0..asset_ids.len())].clone();
                        if !selected.contains(&asset) {
                            selected.push(asset);
                        }
                    }
                    
                    let template_idx = rng.gen_range(0..three_asset_templates.len());
                    let condition_json = three_asset_templates[template_idx](
                        &selected[0], &selected[1], &selected[2], rng
                    );
                    
                    Watcher {
                        id: format!("watcher_{:06}", i),
                        user_id: format!("user_{:05}", rng.gen_range(0..10000)),
                        asset_ids: selected,
                        name: format!("Multi-Asset Alert {}", i),
                        matcher: JsonMatcher::from_json(&condition_json).unwrap(),
                    }
                } else {
                    // Pick 2 random distinct assets
                    let mut selected: Vec<String> = Vec::new();
                    while selected.len() < 2 {
                        let asset = asset_ids[rng.gen_range(0..asset_ids.len())].clone();
                        if !selected.contains(&asset) {
                            selected.push(asset);
                        }
                    }
                    
                    let template_idx = rng.gen_range(0..two_asset_templates.len());
                    let condition_json = two_asset_templates[template_idx](
                        &selected[0], &selected[1], rng
                    );
                    
                    Watcher {
                        id: format!("watcher_{:06}", i),
                        user_id: format!("user_{:05}", rng.gen_range(0..10000)),
                        asset_ids: selected,
                        name: format!("Multi-Asset Alert {}", i),
                        matcher: JsonMatcher::from_json(&condition_json).unwrap(),
                    }
                }
            } else {
                // Single asset watcher
                let asset_id = asset_ids[rng.gen_range(0..asset_ids.len())].clone();
                let template_idx = rng.gen_range(0..condition_templates.len());
                let condition_json = condition_templates[template_idx](&asset_id, rng);

                Watcher {
                    id: format!("watcher_{:06}", i),
                    user_id: format!("user_{:05}", rng.gen_range(0..10000)),
                    asset_ids: vec![asset_id],
                    name: format!("Alert Rule {}", i),
                    matcher: JsonMatcher::from_json(&condition_json).unwrap(),
                }
            }
        })
        .collect()
}

fn main() {
    println!("=== Parallel Cache Watcher Matching Example ===\n");

    let mut rng = StdRng::seed_from_u64(42); // Deterministic for reproducibility

    // Generate test data
    println!("Generating test data...");
    let start = Instant::now();
    let (cache, asset_ids) = generate_cache(1_000, &mut rng);
    println!(
        "  Generated cache with {} assets in {:?}",
        cache.len(),
        start.elapsed()
    );

    let start = Instant::now();
    let watchers = generate_watchers(100_000, &asset_ids, &mut rng);
    println!(
        "  Generated {} watchers in {:?}",
        watchers.len(),
        start.elapsed()
    );

    // Show some sample data
    println!("\nSample assets in cache:");
    for asset_id in asset_ids.iter().take(5) {
        if let Some(metrics) = cache.get(asset_id) {
            println!(
                "  {} - Price: ${:.2}, 24h: {:.1}%, Vol mult: {:.2}x",
                metrics.asset,
                metrics.current_price,
                metrics.pct_change_24h.unwrap_or(0.0),
                metrics.volume_multiplier.unwrap_or(1.0)
            );
        }
    }

    println!("\nSample watchers (with asset-specific conditions):");
    for watcher in watchers.iter().take(5) {
        let assets_str = watcher.asset_ids.join(", ");
        println!(
            "  {} watching [{}] ({}): {:?}",
            watcher.id,
            assets_str,
            watcher.user_id,
            watcher.matcher.condition()
        );
    }

    // Demonstrate field path access
    println!("\n--- Field Path Access Demo ---");
    let demo_asset = &asset_ids[0];
    if let Some(metrics) = cache.get(demo_asset) {
        println!("  Direct access to {}: price=${:.2}", demo_asset, metrics.current_price);
    }
    // This is how the matcher accesses it internally via get_field_path
    println!(
        "  Via Matchable path '{}.current_price': {:?}",
        demo_asset,
        cache.get_field_path(&[demo_asset, "current_price"])
            .and_then(|v| v.downcast_ref::<f64>())
    );

    // Benchmark: Check all watchers against the single cache
    println!("\n--- Benchmark: Match all watchers against cache ---");

    // Sequential matching
    println!("\nSequential matching...");
    let start = Instant::now();
    let triggered_seq: Vec<_> = watchers
        .iter()
        .filter(|w| w.matcher.matches(&cache))
        .collect();
    let seq_duration = start.elapsed();
    println!(
        "  Sequential: {} triggered watchers in {:?} ({:.0} watchers/sec)",
        triggered_seq.len(),
        seq_duration,
        watchers.len() as f64 / seq_duration.as_secs_f64()
    );

    // Parallel matching using rayon
    println!("\nParallel matching (using rayon)...");
    use rayon::prelude::*;
    let start = Instant::now();
    let triggered_par: Vec<_> = watchers
        .par_iter()
        .filter(|w| w.matcher.matches(&cache))
        .collect();
    let par_duration = start.elapsed();
    println!(
        "  Parallel: {} triggered watchers in {:?} ({:.0} watchers/sec)",
        triggered_par.len(),
        par_duration,
        watchers.len() as f64 / par_duration.as_secs_f64()
    );

    // Verify consistency
    assert_eq!(triggered_seq.len(), triggered_par.len());
    println!("\n✓ Both methods produced consistent results");

    // Calculate speedup
    let speedup = seq_duration.as_secs_f64() / par_duration.as_secs_f64();
    println!("  Speedup: {:.2}x", speedup);

    // Detailed statistics
    println!("\n--- Triggered Watcher Statistics ---");
    let total_triggered = triggered_par.len();
    let trigger_rate = total_triggered as f64 / watchers.len() as f64 * 100.0;
    println!("  Total triggered: {} / {} ({:.1}%)", total_triggered, watchers.len(), trigger_rate);

    // Count single vs multi-asset watchers
    let multi_asset_triggered: Vec<_> = triggered_par.iter().filter(|w| w.asset_ids.len() > 1).collect();
    let single_asset_triggered = triggered_par.len() - multi_asset_triggered.len();
    println!("  Single-asset watchers triggered: {}", single_asset_triggered);
    println!("  Multi-asset watchers triggered: {}", multi_asset_triggered.len());

    // Group by primary asset (first asset in the list)
    let mut triggers_by_asset: HashMap<&str, usize> = HashMap::new();
    for w in &triggered_par {
        for asset_id in &w.asset_ids {
            *triggers_by_asset.entry(asset_id.as_str()).or_insert(0) += 1;
        }
    }

    // Find most referenced asset in triggered watchers
    if let Some((asset_id, count)) = triggers_by_asset.iter().max_by_key(|(_, c)| *c) {
        println!("\n  Most referenced asset in triggered watchers: {} ({} times)", asset_id, count);
        if let Some(metrics) = cache.get(*asset_id) {
            println!(
                "    Price: ${:.2}, 24h: {:.1}%, 7d: {:.1}%, Vol mult: {:.2}x",
                metrics.current_price,
                metrics.pct_change_24h.unwrap_or(0.0),
                metrics.pct_change_7d.unwrap_or(0.0),
                metrics.volume_multiplier.unwrap_or(1.0)
            );
        }
    }

    // Distribution of triggers per asset
    let assets_with_triggers = triggers_by_asset.len();
    let avg_triggers = if assets_with_triggers > 0 {
        triggers_by_asset.values().sum::<usize>() as f64 / assets_with_triggers as f64
    } else {
        0.0
    };
    println!("\n  Unique assets in triggered watchers: {}", assets_with_triggers);
    println!("  Average references per asset: {:.1}", avg_triggers);

    // Sample of triggered single-asset watchers
    println!("\n--- Sample Triggered Single-Asset Watchers ---");
    for w in triggered_par.iter().filter(|w| w.asset_ids.len() == 1).take(3) {
        let asset_id = &w.asset_ids[0];
        if let Some(metrics) = cache.get(asset_id) {
            println!(
                "  {} -> {} (24h: {:.1}%, price: ${:.2})",
                w.id,
                asset_id,
                metrics.pct_change_24h.unwrap_or(0.0),
                metrics.current_price
            );
        }
    }

    // Sample of triggered multi-asset watchers
    println!("\n--- Sample Triggered Multi-Asset Watchers ---");
    for w in multi_asset_triggered.iter().take(3) {
        let assets_str = w.asset_ids.join(", ");
        println!("  {} -> [{}]", w.id, assets_str);
        for asset_id in &w.asset_ids {
            if let Some(metrics) = cache.get(asset_id) {
                println!(
                    "    {} - 24h: {:.1}%, price: ${:.2}",
                    asset_id,
                    metrics.pct_change_24h.unwrap_or(0.0),
                    metrics.current_price
                );
            }
        }
    }

    // Group watchers by user and show which have triggered alerts
    println!("\n--- Sample User Alerts ---");
    let mut alerts_by_user: HashMap<&str, Vec<&Watcher>> = HashMap::new();
    for w in &triggered_par {
        alerts_by_user.entry(&w.user_id).or_default().push(w);
    }

    // Show first 3 users with alerts
    for (user_id, alerts) in alerts_by_user.iter().take(3) {
        println!("  User {}: {} alerts triggered", user_id, alerts.len());
        for alert in alerts.iter().take(2) {
            let assets_str = alert.asset_ids.join(", ");
            println!("    - {} watching [{}]", alert.id, assets_str);
        }
        if alerts.len() > 2 {
            println!("    - ... and {} more", alerts.len() - 2);
        }
    }

    println!("\n=== Example Complete ===");
}

