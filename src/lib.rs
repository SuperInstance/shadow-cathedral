//! # shadow-cathedral
//!
//! 3-layer shadow rendering pipeline for agent event streams with
//! information-theoretic lossy compression.

use std::collections::HashMap;
use std::fmt;

/// An unprocessed agent event — the raw shadow layer.
#[derive(Debug, Clone, PartialEq)]
pub struct RawShadow {
    /// Timestamp (ms since epoch).
    pub ts: u64,
    /// Free-form event payload.
    pub payload: String,
}

/// A parsed, categorised, enriched shadow — the structured layer.
#[derive(Debug, Clone, PartialEq)]
pub struct StructuredShadow {
    pub ts: u64,
    pub category: String,
    pub summary: String,
    pub enrichment: HashMap<String, String>,
}

/// A human-readable narrative shadow — the narrative layer.
#[derive(Debug, Clone, PartialEq)]
pub struct NarrativeShadow {
    pub ts: u64,
    pub story: String,
}

/// Configurable transform function type: maps a category string and payload
/// to a summary and enrichment map.
pub type TransformFn = fn(&str, &str) -> (String, HashMap<String, String>);

/// Default transform: uses the first 40 chars of payload as summary,
/// puts payload length into enrichment.
pub fn default_transform(category: &str, payload: &str) -> (String, HashMap<String, String>) {
    let summary = if payload.len() > 40 {
        format!("{}...", &payload[..40])
    } else {
        payload.to_string()
    };
    let mut enrichment = HashMap::new();
    enrichment.insert("category".to_string(), category.to_string());
    enrichment.insert("payload_len".to_string(), payload.len().to_string());
    (summary, enrichment)
}

/// Categorise a raw shadow by simple prefix rules:
/// `ERR:` → "error", `WARN:` → "warning", otherwise "info".
pub fn categorise(payload: &str) -> &'static str {
    if payload.starts_with("ERR:") {
        "error"
    } else if payload.starts_with("WARN:") {
        "warning"
    } else {
        "info"
    }
}

/// The full 3-layer pipeline: Raw → Structured → Narrative.
#[derive(Debug, Clone)]
pub struct ShadowPipeline {
    transform: TransformFn,
}

impl ShadowPipeline {
    /// Create a pipeline with the default transform.
    pub fn new() -> Self {
        Self {
            transform: default_transform,
        }
    }

    /// Create a pipeline with a custom transform.
    pub fn with_transform(transform: TransformFn) -> Self {
        Self { transform }
    }

    /// Process a single raw shadow into a structured shadow.
    pub fn raw_to_structured(&self, raw: &RawShadow) -> StructuredShadow {
        let category = categorise(&raw.payload).to_string();
        let (summary, enrichment) = (self.transform)(&category, &raw.payload);
        StructuredShadow {
            ts: raw.ts,
            category,
            summary,
            enrichment,
        }
    }

    /// Process a single structured shadow into a narrative shadow.
    pub fn structured_to_narrative(s: &StructuredShadow) -> NarrativeShadow {
        let story = format!(
            "At t={}, a {} event occurred: {}",
            s.ts, s.category, s.summary
        );
        NarrativeShadow { ts: s.ts, story }
    }

    /// Full pipeline: raw → narrative (convenience).
    pub fn render(&self, raw: &RawShadow) -> NarrativeShadow {
        let s = self.raw_to_structured(raw);
        Self::structured_to_narrative(&s)
    }

    /// Batch-render many raw shadows.
    pub fn render_batch(&self, raws: &[RawShadow]) -> Vec<NarrativeShadow> {
        raws.iter().map(|r| self.render(r)).collect()
    }
}

impl Default for ShadowPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Information-theoretic lossy compressor for shadow data.
///
/// Uses a simple dictionary-based approach: maps repeated payloads to
/// short codes, keeping only the top-N most frequent entries.
#[derive(Debug, Clone)]
pub struct LossyCompressor {
    /// Maximum dictionary size (top-N frequent tokens to retain).
    pub max_dict: usize,
}

impl LossyCompressor {
    /// Create a new compressor that keeps up to `max_dict` unique entries.
    pub fn new(max_dict: usize) -> Self {
        Self { max_dict }
    }

    /// Compress a slice of raw shadows into a summary string.
    /// Each unique payload becomes a token; only the top-N most frequent
    /// tokens survive, the rest are replaced with `…`.
    pub fn compress(&self, shadows: &[RawShadow]) -> String {
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for s in shadows {
            *freq.entry(&s.payload).or_insert(0) += 1;
        }
        let mut entries: Vec<_> = freq.into_iter().collect();
        entries.sort_by_key(|b| std::cmp::Reverse(b.1));
        let dict: HashMap<&str, usize> = entries
            .into_iter()
            .take(self.max_dict)
            .enumerate()
            .map(|(i, (k, _))| (k, i))
            .collect();
        shadows
            .iter()
            .map(|s| {
                if let Some(&code) = dict.get(s.payload.as_str()) {
                    format!("#{}", code)
                } else {
                    "…".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("|")
    }

    /// Compute the compression ratio: compressed length / original length.
    pub fn compression_ratio(&self, shadows: &[RawShadow]) -> f64 {
        if shadows.is_empty() {
            return 1.0;
        }
        let original: usize = shadows.iter().map(|s| s.payload.len()).sum();
        let compressed = self.compress(shadows);
        if original == 0 {
            return 1.0;
        }
        compressed.len() as f64 / original as f64
    }
}

impl fmt::Display for NarrativeShadow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.story)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorise_error() {
        assert_eq!(categorise("ERR: disk full"), "error");
    }

    #[test]
    fn categorise_warning() {
        assert_eq!(categorise("WARN: low memory"), "warning");
    }

    #[test]
    fn categorise_info() {
        assert_eq!(categorise("user logged in"), "info");
    }

    #[test]
    fn pipeline_raw_to_structured() {
        let pipe = ShadowPipeline::new();
        let raw = RawShadow { ts: 100, payload: "hello world".into() };
        let s = pipe.raw_to_structured(&raw);
        assert_eq!(s.ts, 100);
        assert_eq!(s.category, "info");
        assert_eq!(s.summary, "hello world");
    }

    #[test]
    fn pipeline_structured_to_narrative() {
        let s = StructuredShadow {
            ts: 42,
            category: "error".into(),
            summary: "disk full".into(),
            enrichment: HashMap::new(),
        };
        let n = ShadowPipeline::structured_to_narrative(&s);
        assert!(n.story.contains("error"));
        assert!(n.story.contains("disk full"));
    }

    #[test]
    fn pipeline_full_render() {
        let pipe = ShadowPipeline::new();
        let raw = RawShadow { ts: 0, payload: "ERR: timeout".into() };
        let n = pipe.render(&raw);
        assert!(n.story.contains("timeout"));
        assert_eq!(n.ts, 0);
    }

    #[test]
    fn pipeline_batch() {
        let pipe = ShadowPipeline::new();
        let raws = vec![
            RawShadow { ts: 1, payload: "a".into() },
            RawShadow { ts: 2, payload: "b".into() },
        ];
        let narrs = pipe.render_batch(&raws);
        assert_eq!(narrs.len(), 2);
    }

    #[test]
    fn default_transform_enrichment() {
        let (summary, enrich) = default_transform("info", "hello");
        assert_eq!(summary, "hello");
        assert_eq!(enrich.get("payload_len").unwrap(), "5");
    }

    #[test]
    fn default_transform_truncates() {
        let long = "x".repeat(100);
        let (summary, _) = default_transform("info", &long);
        assert!(summary.ends_with("..."));
        assert_eq!(summary.len(), 43); // 40 + "..."
    }

    #[test]
    fn compressor_basic() {
        let comp = LossyCompressor::new(2);
        let shadows = vec![
            RawShadow { ts: 1, payload: "alpha".into() },
            RawShadow { ts: 2, payload: "beta".into() },
            RawShadow { ts: 3, payload: "gamma".into() },
        ];
        let result = comp.compress(&shadows);
        // alpha and beta are first two, gamma gets elided
        assert!(result.contains("#0"));
        assert!(result.contains("#1"));
        assert!(result.contains("…"));
    }

    #[test]
    fn compressor_frequent_priority() {
        let comp = LossyCompressor::new(1);
        let shadows = vec![
            RawShadow { ts: 1, payload: "rare".into() },
            RawShadow { ts: 2, payload: "common".into() },
            RawShadow { ts: 3, payload: "common".into() },
        ];
        let result = comp.compress(&shadows);
        // "common" appears 2x, so it gets a code; "rare" is elided
        assert!(result.contains("#0")); // common = #0
        assert!(result.contains("…")); // rare elided
    }

    #[test]
    fn compressor_empty() {
        let comp = LossyCompressor::new(5);
        let result = comp.compress(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn compressor_ratio() {
        let comp = LossyCompressor::new(10);
        let shadows = vec![
            RawShadow { ts: 1, payload: "abcdef".into() },
            RawShadow { ts: 2, payload: "abcdef".into() },
        ];
        let ratio = comp.compression_ratio(&shadows);
        assert!(ratio < 1.0);
    }

    #[test]
    fn compressor_ratio_empty() {
        let comp = LossyCompressor::new(5);
        assert_eq!(comp.compression_ratio(&[]), 1.0);
    }

    #[test]
    fn narrative_shadow_display() {
        let n = NarrativeShadow { ts: 99, story: "something happened".into() };
        assert_eq!(format!("{}", n), "something happened");
    }
}
