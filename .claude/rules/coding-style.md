# Coding Style — Rust-specific

## Immutability (CRITICAL)

Prefer immutable ownership. In Rust this is the default. The escape hatches (`Cell`, `RefCell`, `Mutex`, interior mutability) require deliberate justification, not convenience.

```rust
// GOOD: data is owned, the chain is append-only, the only mutation is via &mut self on the chain.
pub struct HashChain { entries: Vec<ChainEntry> }
impl HashChain {
    pub fn append(&mut self, ...) -> ChainEntry { ... }  // mut is local to the method
}

// BAD: hand out a mutable reference to the inner Vec
impl HashChain {
    pub fn entries_mut(&mut self) -> &mut Vec<ChainEntry> { &mut self.entries }  // NO
}
```

## File Organization

- Many small files > few large files. **200-400 lines typical, 800 max.**
- One module per concern. `chain.rs` for the chain, `certificate.rs` for the PRC, `traits.rs` for the contracts.
- Organize by **feature/agent**, not by file type. `crates/moirai-clotho/src/` is everything Clotho needs; do not split into `models/`, `views/`, `controllers/`.
- Re-exports at the top of `lib.rs` so the public surface is one place.

## Error Handling

Use `thiserror` for typed errors, `anyhow` for ad-hoc (rarely).

```rust
#[derive(thiserror::Error, Debug)]
pub enum BandError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Rate limited: retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },
}
```

- `?` propagation. **Never `.unwrap()` in production code.** `.expect("reason")` is OK only at startup with a clear message; tests can `.unwrap()`.
- `BandResult<T>` for fallible APIs. Don't return `Option` when an error message helps.

## Input Validation

Use `serde` with strict types. Reject anything unexpected at the deserialization boundary, not deep in business logic.

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrReviewRequest {
    pub pr_url: String,
    #[serde(default)]
    pub re_attest: bool,
}
```

Validate at the system edge (the Band room, the HTTP handler), not inside agents.

## Traits over Inheritance

Define traits for contracts. Implement them explicitly. Don't rely on `Deref` or `Any`.

```rust
#[async_trait]
pub trait BandClient: Send + Sync {
    async fn create_room(&self, task_id: Option<&str>) -> BandResult<RoomId>;
    // ...
}
```

## Hex encoding for crypto

`ed25519_dalek::Signature` and `blake3::Hash` do NOT implement `serde::Serialize`/`Deserialize` by default. Store and transmit them as hex strings (`String`).

## Code Quality Checklist

Before marking work complete:
- [ ] `cargo check --workspace` exits 0
- [ ] `cargo clippy --workspace --all-targets` produces no new warnings
- [ ] `cargo test --workspace` passes
- [ ] No `unwrap()` outside tests or startup-with-expect
- [ ] No `println!` in production code (`tracing` instead)
- [ ] No hardcoded values (use constants, env, or `serde` deserialization)
- [ ] Public types have doc comments (run with `#![warn(missing_docs)]` at lib level)
- [ ] Single binary stays under 30 MB

## [MOIRAI-specific]

- **Hex strings for all crypto primitives** in serde-serialized types. Bit-by-bit proof: `PrReviewCertificate` uses `signature_hex: Option<String>` not `Option<Signature>`.
- **Sequence-monotonic chains**: every `ChainEntry` has a `sequence: u64` assigned at enqueue time. Use a 500ms re-ordering buffer if accepting events from async sources (SCEPTRE v2 design).
- **Band `@mention` routing**: messages that don't mention an agent are NOT delivered to it. This is Band's context-isolation primitive. Compressor intercepts EVERY message between two agents — `@Clotho → @Lachesis` becomes `@Clotho → @Compressor → @Lachesis`.
- **BAAAR events**: only emitted by Átropos. Other agents emit `EventKind::Receipt` or `EventKind::ChainLink` only.
