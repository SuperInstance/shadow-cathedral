# shadow-cathedral

> **Raw event → Structured shadow → Narrative. Three layers of rendering.**

[![crates.io](https://img.shields.io/crates/v/shadow-cathedral.svg)](https://crates.io/crates/shadow-cathedral)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Three-layer shadow rendering pipeline for agent cognition. Events pass through Raw → Structured → Narrative transformations with configurable lossy compression at each stage.

## Architecture

```
Raw Events ──► RawShadow ──► StructuredShadow ──► NarrativeShadow
                 (stream)      (parsed/tagged)       (human story)
```

The **Shadow Cathedral** metaphor: like a cathedral's stained glass transforming raw light into colored patterns into stories on the floor, this pipeline transforms raw agent events into human-readable narratives.

## Part of [Exocortex](https://github.com/SuperInstance/exocortex)

Named after the Gemma 4 31B design competition entry that proposed the 3-layer shadow architecture.

## License

MIT © [SuperInstance](https://github.com/SuperInstance)
