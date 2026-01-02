//! Example demonstrating JSON matchers with complex Matchable types.
//!
//! This example shows how to implement `Matchable` for a struct that stores
//! data internally in a HashMap, allowing JSON conditions to query nested data.

use std::{any::Any, collections::HashMap};

use chrono::{DateTime, Utc};
use condition_matcher::{JsonMatcher, Matchable, Matcher, MatcherExt};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Asset metrics containing price, volume, and technical indicators.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetMetrics {
    pub asset: String,
    pub current_price: Decimal,
    pub current_volume: Option<Decimal>,
    pub current_market_cap: Option<Decimal>,
    pub last_updated: DateTime<Utc>,

    // Price change percentages
    pub pct_change_1h: Option<Decimal>,
    pub pct_change_4h: Option<Decimal>,
    pub pct_change_24h: Option<Decimal>,
    pub pct_change_7d: Option<Decimal>,
    pub pct_change_30d: Option<Decimal>,

    // Volume metrics
    pub avg_volume_24h: Option<Decimal>,
    pub volume_multiplier: Option<Decimal>,

    // Volatility
    pub volatility_24h: Option<Decimal>,

    // Moving averages
    pub sma_7d: Option<Decimal>,
    pub sma_30d: Option<Decimal>,
    pub sma_50d: Option<Decimal>,
    pub sma_200d: Option<Decimal>,

    // MA signals
    pub above_sma_7d: Option<bool>,
    pub above_sma_50d: Option<bool>,
    pub above_sma_200d: Option<bool>,

    pub computed_at: DateTime<Utc>,
}

/// A cache holding metrics for multiple assets, keyed by asset symbol.
///
/// This implements `Matchable` to allow JSON conditions to query metrics
/// for a specific asset using dot notation like "BTC.current_price".
#[derive(Debug, PartialEq)]
pub struct MetricsCache {
    metrics: HashMap<String, AssetMetrics>,
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
}

/// Helper to convert Decimal to f64 for comparison.
fn decimal_to_f64(d: &Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}

impl Matchable for MetricsCache {
    fn get_field(&self, _name: &str) -> Option<&dyn Any> {
        // For single field access, we don't support it directly
        // Use get_field_path with ["asset", "field"] instead
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
            "current_volume" => Some(&metrics.current_volume),
            "current_market_cap" => Some(&metrics.current_market_cap),
            "pct_change_1h" => Some(&metrics.pct_change_1h),
            "pct_change_4h" => Some(&metrics.pct_change_4h),
            "pct_change_24h" => Some(&metrics.pct_change_24h),
            "pct_change_7d" => Some(&metrics.pct_change_7d),
            "pct_change_30d" => Some(&metrics.pct_change_30d),
            "avg_volume_24h" => Some(&metrics.avg_volume_24h),
            "volume_multiplier" => Some(&metrics.volume_multiplier),
            "volatility_24h" => Some(&metrics.volatility_24h),
            "sma_7d" => Some(&metrics.sma_7d),
            "sma_30d" => Some(&metrics.sma_30d),
            "sma_50d" => Some(&metrics.sma_50d),
            "sma_200d" => Some(&metrics.sma_200d),
            "above_sma_7d" => Some(&metrics.above_sma_7d),
            "above_sma_50d" => Some(&metrics.above_sma_50d),
            "above_sma_200d" => Some(&metrics.above_sma_200d),
            _ => None,
        }
    }
}

/// Implement Matchable for individual AssetMetrics for direct matching.
impl Matchable for AssetMetrics {
    fn get_field(&self, name: &str) -> Option<&dyn Any> {
        match name {
            "asset" => Some(&self.asset),
            "current_price" => Some(&self.current_price),
            "current_volume" => Some(&self.current_volume),
            "current_market_cap" => Some(&self.current_market_cap),
            "pct_change_1h" => Some(&self.pct_change_1h),
            "pct_change_4h" => Some(&self.pct_change_4h),
            "pct_change_24h" => Some(&self.pct_change_24h),
            "pct_change_7d" => Some(&self.pct_change_7d),
            "pct_change_30d" => Some(&self.pct_change_30d),
            "avg_volume_24h" => Some(&self.avg_volume_24h),
            "volume_multiplier" => Some(&self.volume_multiplier),
            "volatility_24h" => Some(&self.volatility_24h),
            "sma_7d" => Some(&self.sma_7d),
            "sma_30d" => Some(&self.sma_30d),
            "sma_50d" => Some(&self.sma_50d),
            "sma_200d" => Some(&self.sma_200d),
            "above_sma_7d" => Some(&self.above_sma_7d),
            "above_sma_50d" => Some(&self.above_sma_50d),
            "above_sma_200d" => Some(&self.above_sma_200d),
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

/// Wrapper to make AssetMetrics comparable with f64 values in JSON conditions.
/// This wraps numeric fields as f64 for comparison.
#[derive(Debug, PartialEq)]
pub struct AssetMetricsView<'a> {
    metrics: &'a AssetMetrics,
    // Cache converted values for the lifetime of the view
    current_price_f64: f64,
    pct_change_1h_f64: Option<f64>,
    pct_change_24h_f64: Option<f64>,
    pct_change_7d_f64: Option<f64>,
    volume_multiplier_f64: Option<f64>,
    volatility_24h_f64: Option<f64>,
}

impl<'a> AssetMetricsView<'a> {
    pub fn new(metrics: &'a AssetMetrics) -> Self {
        Self {
            metrics,
            current_price_f64: decimal_to_f64(&metrics.current_price),
            pct_change_1h_f64: metrics.pct_change_1h.as_ref().map(decimal_to_f64),
            pct_change_24h_f64: metrics.pct_change_24h.as_ref().map(decimal_to_f64),
            pct_change_7d_f64: metrics.pct_change_7d.as_ref().map(decimal_to_f64),
            volume_multiplier_f64: metrics.volume_multiplier.as_ref().map(decimal_to_f64),
            volatility_24h_f64: metrics.volatility_24h.as_ref().map(decimal_to_f64),
        }
    }
}

impl Matchable for AssetMetricsView<'_> {
    fn get_field(&self, name: &str) -> Option<&dyn Any> {
        match name {
            "asset" => Some(&self.metrics.asset),
            "current_price" => Some(&self.current_price_f64),
            "pct_change_1h" => self.pct_change_1h_f64.as_ref().map(|v| v as &dyn Any),
            "pct_change_24h" => self.pct_change_24h_f64.as_ref().map(|v| v as &dyn Any),
            "pct_change_7d" => self.pct_change_7d_f64.as_ref().map(|v| v as &dyn Any),
            "volume_multiplier" => self.volume_multiplier_f64.as_ref().map(|v| v as &dyn Any),
            "volatility_24h" => self.volatility_24h_f64.as_ref().map(|v| v as &dyn Any),
            "above_sma_7d" => self.metrics.above_sma_7d.as_ref().map(|v| v as &dyn Any),
            "above_sma_50d" => self.metrics.above_sma_50d.as_ref().map(|v| v as &dyn Any),
            "above_sma_200d" => self.metrics.above_sma_200d.as_ref().map(|v| v as &dyn Any),
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

fn main() {
    println!("=== JSON Complex Matching Example ===\n");

    // Create sample asset metrics
    let now = Utc::now();
    
    let btc = AssetMetrics {
        asset: "BTC".to_string(),
        current_price: dec!(67500.50),
        current_volume: Some(dec!(28_000_000_000)),
        current_market_cap: Some(dec!(1_320_000_000_000)),
        last_updated: now,
        pct_change_1h: Some(dec!(0.5)),
        pct_change_4h: Some(dec!(1.2)),
        pct_change_24h: Some(dec!(3.5)),
        pct_change_7d: Some(dec!(8.2)),
        pct_change_30d: Some(dec!(-2.1)),
        avg_volume_24h: Some(dec!(25_000_000_000)),
        volume_multiplier: Some(dec!(1.12)),
        volatility_24h: Some(dec!(2.5)),
        sma_7d: Some(dec!(65000)),
        sma_30d: Some(dec!(62000)),
        sma_50d: Some(dec!(58000)),
        sma_200d: Some(dec!(45000)),
        above_sma_7d: Some(true),
        above_sma_50d: Some(true),
        above_sma_200d: Some(true),
        computed_at: now,
    };

    let eth = AssetMetrics {
        asset: "ETH".to_string(),
        current_price: dec!(3450.25),
        current_volume: Some(dec!(15_000_000_000)),
        current_market_cap: Some(dec!(415_000_000_000)),
        last_updated: now,
        pct_change_1h: Some(dec!(-0.3)),
        pct_change_4h: Some(dec!(-1.5)),
        pct_change_24h: Some(dec!(-2.1)),
        pct_change_7d: Some(dec!(5.0)),
        pct_change_30d: Some(dec!(12.0)),
        avg_volume_24h: Some(dec!(14_000_000_000)),
        volume_multiplier: Some(dec!(1.07)),
        volatility_24h: Some(dec!(3.2)),
        sma_7d: Some(dec!(3400)),
        sma_30d: Some(dec!(3200)),
        sma_50d: Some(dec!(3000)),
        sma_200d: Some(dec!(2500)),
        above_sma_7d: Some(true),
        above_sma_50d: Some(true),
        above_sma_200d: Some(true),
        computed_at: now,
    };

    let sol = AssetMetrics {
        asset: "SOL".to_string(),
        current_price: dec!(145.80),
        current_volume: Some(dec!(2_500_000_000)),
        current_market_cap: Some(dec!(63_000_000_000)),
        last_updated: now,
        pct_change_1h: Some(dec!(2.1)),
        pct_change_4h: Some(dec!(5.5)),
        pct_change_24h: Some(dec!(12.3)),
        pct_change_7d: Some(dec!(25.0)),
        pct_change_30d: Some(dec!(45.0)),
        avg_volume_24h: Some(dec!(2_000_000_000)),
        volume_multiplier: Some(dec!(1.25)),
        volatility_24h: Some(dec!(8.5)),
        sma_7d: Some(dec!(130)),
        sma_30d: Some(dec!(110)),
        sma_50d: Some(dec!(95)),
        sma_200d: Some(dec!(75)),
        above_sma_7d: Some(true),
        above_sma_50d: Some(true),
        above_sma_200d: Some(true),
        computed_at: now,
    };

    // Create views for matching (converts Decimal to f64)
    let assets = vec![
        AssetMetricsView::new(&btc),
        AssetMetricsView::new(&eth),
        AssetMetricsView::new(&sol),
    ];

    println!("Sample assets:");
    println!("  BTC: ${}, 24h change: {}%", btc.current_price, btc.pct_change_24h.unwrap());
    println!("  ETH: ${}, 24h change: {}%", eth.current_price, eth.pct_change_24h.unwrap());
    println!("  SOL: ${}, 24h change: {}%", sol.current_price, sol.pct_change_24h.unwrap());

    // Example 1: Find assets with positive 24h change
    println!("\n--- Example 1: Assets with positive 24h change ---");
    let condition = r#"{
        "mode": "AND",
        "rules": [
            {"field": "pct_change_24h", "operator": "greater_than", "value": 0}
        ]
    }"#;
    let matcher = JsonMatcher::from_json(condition).unwrap();
    let matches: Vec<_> = assets.iter().filter(|a| matcher.matches(*a)).collect();
    println!("Found {} assets with positive 24h change:", matches.len());
    for m in &matches {
        println!("  - {} ({}%)", m.metrics.asset, m.metrics.pct_change_24h.unwrap());
    }

    // Example 2: Find assets pumping (high 24h change + high volume)
    println!("\n--- Example 2: Assets pumping (>10% 24h + volume multiplier > 1.2) ---");
    let condition = r#"{
        "mode": "AND",
        "rules": [
            {"field": "pct_change_24h", "operator": "greater_than", "value": 10},
            {"field": "volume_multiplier", "operator": "greater_than", "value": 1.2}
        ]
    }"#;
    let matcher = JsonMatcher::from_json(condition).unwrap();
    let matches: Vec<_> = assets.iter().filter(|a| matcher.matches(*a)).collect();
    println!("Found {} pumping assets:", matches.len());
    for m in &matches {
        println!(
            "  - {} (24h: {}%, vol mult: {}x)",
            m.metrics.asset,
            m.metrics.pct_change_24h.unwrap(),
            m.metrics.volume_multiplier.unwrap()
        );
    }

    // Example 3: Find large caps above all SMAs (bullish trend)
    println!("\n--- Example 3: Assets above all moving averages (bullish) ---");
    let condition = r#"{
        "mode": "AND",
        "rules": [
            {"field": "above_sma_7d", "operator": "equals", "value": true},
            {"field": "above_sma_50d", "operator": "equals", "value": true},
            {"field": "above_sma_200d", "operator": "equals", "value": true}
        ]
    }"#;
    let matcher = JsonMatcher::from_json(condition).unwrap();
    let matches: Vec<_> = assets.iter().filter(|a| matcher.matches(*a)).collect();
    println!("Found {} bullish assets (above all SMAs):", matches.len());
    for m in &matches {
        println!("  - {}", m.metrics.asset);
    }

    // Example 4: Complex nested conditions - either pumping OR (dipping but above 200 SMA)
    println!("\n--- Example 4: Nested conditions (pumping OR dipping above 200 SMA) ---");
    let condition = r#"{
        "mode": "OR",
        "nested": [
            {
                "mode": "AND",
                "rules": [
                    {"field": "pct_change_24h", "operator": "greater_than", "value": 5}
                ]
            },
            {
                "mode": "AND",
                "rules": [
                    {"field": "pct_change_24h", "operator": "less_than", "value": 0},
                    {"field": "above_sma_200d", "operator": "equals", "value": true}
                ]
            }
        ]
    }"#;
    let matcher = JsonMatcher::from_json(condition).unwrap();
    let matches: Vec<_> = assets.iter().filter(|a| matcher.matches(*a)).collect();
    println!("Found {} assets (pumping OR dipping but bullish):", matches.len());
    for m in &matches {
        let reason = if decimal_to_f64(&m.metrics.pct_change_24h.unwrap()) > 5.0 {
            "pumping"
        } else {
            "dipping but above 200 SMA"
        };
        println!("  - {} ({})", m.metrics.asset, reason);
    }

    // Example 5: Using filter extension method
    println!("\n--- Example 5: Using filter() extension method ---");
    let condition = r#"{
        "mode": "AND",
        "rules": [
            {"field": "current_price", "operator": "greater_than", "value": 1000}
        ]
    }"#;
    let matcher = JsonMatcher::from_json(condition).unwrap();
    let expensive_assets = matcher.filter(&assets);
    println!("Assets with price > $1000: {}", expensive_assets.len());
    for a in expensive_assets {
        println!("  - {}: ${}", a.metrics.asset, a.metrics.current_price);
    }

    println!("\n=== Example Complete ===");
}

