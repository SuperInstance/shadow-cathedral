# shadow-cathedral

> **Raw event → Structured shadow → Narrative. Three layers of rendering.**

[![crates.io](https://img.shields.io/crates/v/shadow-cathedral.svg)](https://crates.io/crates/shadow-cathedral)
[![docs.rs](https://docs.rs/shadow-cathedral/badge.svg)](https://docs.rs/shadow-cathedral)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust library implementing a three-layer shadow rendering pipeline for agent cognition. Raw events pass through **RawShadow → StructuredShadow → NarrativeShadow** transformations with configurable processing and lossy compression at each stage. Like a cathedral's stained glass transforming raw light into patterns into stories.

---

## Table of Contents

- [What is the Shadow Cathedral?](#what-is-the-shadow-cathedral)
- [Why Does This Matter?](#why-does-this-matter)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Technical Background](#technical-background)
- [Installation](#installation)
- [Related Crates](#related-crates)
- [License](#license)

---

## What is the Shadow Cathedral?

Imagine light streaming through a cathedral's stained glass windows:

```
Raw Sunlight  ──►  Stained Glass  ──►  Colored Patterns  ──►  Stories on the Floor
(unfiltered)       (filtering)         (structured)            (narrative)
```

The **Shadow Cathedral** applies this metaphor to agent event streams:

```
Raw Event    ──►  RawShadow     ──►  StructuredShadow  ──►  NarrativeShadow
(what happened)    (timestamped)      (categorized)         (human-readable story)
```

Three layers, each adding structure and losing raw detail:

1. **RawShadow** — The unprocessed event: timestamp + free-form payload. No interpretation, no filtering.
2. **StructuredShadow** — Parsed, categorized, enriched. The raw text becomes a typed record with metadata.
3. **NarrativeShadow** — A human-readable story. The structured data becomes prose that a person can understand.

At each stage, information is **compressed** — not all details survive the transformation. This is by design: a useful summary is not a perfect reproduction. The lossy compression preserves what matters and discards noise.

## Why Does This Matter?

**For agent observability**: Agents generate enormous event streams. Humans can't read raw logs. The Shadow Cathedral transforms machine events into human narratives in real-time.

**For memory management**: Lossy compression (implemented via dictionary-based encoding) reduces memory footprint while preserving essential information. Agents remember stories, not byte streams.

**For multi-modal reasoning**: Different consumers need different representations: machines read StructuredShadows, humans read NarrativeShadows, archives store compressed RawShadows.

**For debugging**: The three-layer pipeline makes it easy to trace where information was lost. If a narrative doesn't match reality, check the StructuredShadow. If that's wrong, check the RawShadow. Three layers = three debug checkpoints.

## Architecture

```
shadow-cathedral
│
├── RawShadow                   ← Layer 1: Unprocessed events
│   ├── ts: u64                     Timestamp (ms since epoch)
│   ├── payload: String             Free-form event data
│   └── (constructed from any event)
│
├── StructuredShadow            ← Layer 2: Parsed and enriched
│   ├── ts: u64                     Timestamp (preserved)
│   ├── category: String            "error" / "warning" / "info"
│   ├── summary: String             Extracted summary (≤40 chars)
│   └── enrichment: HashMap         Key-value metadata
│
├── NarrativeShadow             ← Layer 3: Human-readable
│   ├── ts: u64                     Timestamp (preserved)
│   └── story: String               Natural language narrative
│
├── Transform Functions         ← Layer-crossing logic
│   ├── default_transform()         Category + payload → summary + enrichment
│   └── categorise(payload)         "ERR:" → error, "WARN:" → warning, else info
│
├── ShadowPipeline              ← End-to-end processing
│   ├── new()                       Default pipeline
│   ├── with_transform(fn)          Custom transform function
│   ├── raw_to_structured(raw)      Layer 1 → Layer 2
│   ├── structured_to_narrative(s)  Layer 2 → Layer 3
│   ├── render(raw)                 Full pipeline: Raw → Narrative
│   └── render_batch(raws)          Process multiple events
│
└── LossyCompressor             ← Dictionary-based compression
    ├── new(max_dict_size)          Initialize with max dictionary entries
    ├── compress(shadows)           Compress to dictionary-encoded string
    └── compression_ratio(shadows)  Original / compressed size
```

## Quick Start

```rust
use shadow_cathedral::{
    RawShadow, ShadowPipeline, LossyCompressor,
};

// Create a pipeline (uses default transform)
let pipeline = ShadowPipeline::new();

// Feed a raw event
let raw = RawShadow {
    ts: 1717700000,
    payload: "ERR:Connection refused to database server at 192.168.1.50:5432".into(),
};

// Render through all three layers
let narrative = pipeline.render(&raw);
println!("Timestamp: {}", narrative.ts);
println!("Story: {}", narrative.story);

// Step through each layer individually
let structured = pipeline.raw_to_structured(&raw);
println!("Category: {}", structured.category);    // "error"
println!("Summary: {}", structured.summary);       // "ERR:Connection refused to database ..."

// Batch processing
let events = vec![
    RawShadow { ts: 100, payload: "User logged in".into() },
    RawShadow { ts: 101, payload: "WARN:Disk space low on /tmp".into() },
    RawShadow { ts: 102, payload: "ERR:Service timeout".into() },
];
let narratives = pipeline.render_batch(&events);
for n in &narratives {
    println!("[{}] {}", n.ts, n.story);
}

// Lossy compression
let compressor = LossyCompressor::new(256);
let compressed = compressor.compress(&events);
let ratio = compressor.compression_ratio(&events);
println!("Compressed: {} bytes (ratio: {:.2}x)", compressed.len(), ratio);
```

## API Reference

### Shadow Types

| Type | Fields | Description |
|------|--------|-------------|
| `RawShadow` | `ts: u64`, `payload: String` | Unprocessed event |
| `StructuredShadow` | `ts`, `category`, `summary`, `enrichment: HashMap` | Parsed, categorized |
| `NarrativeShadow` | `ts: u64`, `story: String` | Human-readable narrative |

### ShadowPipeline

| Method | Returns | Description |
|--------|---------|-------------|
| `new()` | `Self` | Default pipeline with standard transform |
| `with_transform(fn)` | `Self` | Custom transform function |
| `raw_to_structured(&raw)` | `StructuredShadow` | Layer 1 → Layer 2 |
| `structured_to_narrative(&s)` | `NarrativeShadow` | Layer 2 → Layer 3 |
| `render(&raw)` | `NarrativeShadow` | Full pipeline: Raw → Narrative |
| `render_batch(&raws)` | `Vec<NarrativeShadow>` | Process multiple events |

### Transform & Categorization

| Function | Returns | Description |
|----------|---------|-------------|
| `default_transform(cat, payload)` | `(String, HashMap)` | Extract summary + enrichment |
| `categorise(payload)` | `&str` | "ERR:" → error, "WARN:" → warning, else info |

### LossyCompressor

| Method | Returns | Description |
|--------|---------|-------------|
| `new(max_dict_size)` | `Self` | Initialize compressor |
| `compress(&shadows)` | `String` | Dictionary-encoded output |
| `compression_ratio(&shadows)` | `f64` | Original size / compressed size |

## Technical Background

### Three-Layer Architecture

The pipeline follows the **Information Bottleneck** principle: at each layer, the representation should be maximally informative about the output (narrative) while being maximally compressed relative to the input (raw event).

```
I(Raw; Structured) ≥ I(Raw; Narrative)    (information decreases)
H(Structured) ≤ H(Raw)                    (entropy decreases)
```

This is the same principle used in:
- **Deep learning**: autoencoder bottleneck layers
- **Signal processing**: progressive encoding (JPEG, MP3)
- **Cognitive science**: levels of processing in memory (Craik & Lockhart, 1972)

### Categorization Rules

The default categorizer uses simple prefix rules:

```
"ERR:..."  → "error"
"WARN:..." → "warning"
otherwise  → "info"
```

Custom transform functions allow arbitrary categorization and enrichment logic:

```rust
type TransformFn = fn(category: &str, payload: &str) -> (String, HashMap<String, String>);
```

### Dictionary-Based Compression

The lossy compressor builds a dictionary of common substrings and replaces them with short tokens:

```
Original:  "ERR:Connection refused to database server"
Compressed: "ERR:#{1} to #{2}"  where #{1}="Connection refused", #{2}="database server"
```

Compression ratio = original_size / compressed_size. Higher dictionary capacity = better compression but more memory overhead.

### Narrative Generation

StructuredShadow → NarrativeShadow conversion creates human-readable stories:

```
Structured: { category: "error", summary: "Connection refused..." }
Narrative:  "At 1717700000, an error occurred: Connection refused..."
```

The narrative layer is designed for human consumption — status dashboards, alert systems, and debugging logs. It prioritizes readability over completeness.

## Installation

```bash
cargo add shadow-cathedral
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
shadow-cathedral = "0.1"
```

## Related Crates

Part of the **SuperInstance Exocortex** ecosystem:

- **[dream-cycle](https://github.com/SuperInstance/dream-cycle)** — Sleep consolidation for agent memory
- **[forgetting-curve](https://github.com/SuperInstance/forgetting-curve)** — Ebbinghaus forgetting and spaced repetition
- **[cortex-bus-protocol](https://github.com/SuperInstance/cortex-bus-protocol)** — CQRS event bus for agents
- **[signal-transduction](https://github.com/SuperInstance/signal-transduction)** — Signal cascading for agents
- **[active-inference](https://github.com/SuperInstance/active-inference)** — Action as surprise minimization

## License

MIT © [SuperInstance](https://github.com/SuperInstance)

Part of the [Exocortex](https://github.com/SuperInstance/exocortex) project.
