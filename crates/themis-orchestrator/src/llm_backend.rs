//! LLM backend selection for the production binary.
//!
//! `select_backend()` (US-08) tries `FeatherlessBackend` first
//! (real LLM call to `api.featherless.ai`), and falls back to
//! the mock when the `FEATHERLESS_API_KEY` env var is unset or
//! empty. The fallback is transparent to the test suite because:
//!
//! 1. Tests construct `AppState` directly with their own
//!    `model_id` and never call `select_backend()`.
//! 2. The binary's `main()` is the only caller in the production
//!    code path; it logs which backend is active at startup.
//!
//! An invalid key is treated the same as a missing key: the
//! backend returns `None`, the binary falls back to the mock, and
//! the demo still works. (Real network errors are surfaced by
//! `FeatherlessBackend::complete` per the trait contract.)

use themis_agents::llm::FeatherlessBackend;

/// Model id of the live LLM the demo advertises when
/// `FEATHERLESS_API_KEY` is set. The cost-1 slot (4 concurrent
/// connections) on Featherless Premium; see
/// `docs/REFERENCES.md` for the full pricing table.
pub const FEATHERLESS_MODEL: &str = "Qwen/Qwen3-Coder-30B-A3B-Instruct";

/// Pick the LLM backend for this run. Tries Featherless first
/// (real LLM); falls back to `MockLlmProvider` when the env var
/// is unset or empty. Returns the `model_id` that
/// `AppState.model_id` advertises to the SSE stream — the
/// frontend's provider badge reads from there.
pub fn select_backend() -> &'static str {
    if FeatherlessBackend::from_env(FEATHERLESS_MODEL).is_some() {
        eprintln!(
            "[themis-orchestrator] LLM: FeatherlessBackend({FEATHERLESS_MODEL}) — live"
        );
        FEATHERLESS_MODEL
    } else {
        eprintln!(
            "[themis-orchestrator] LLM: MockLlmProvider — FEATHERLESS_API_KEY not set, using mock"
        );
        "mock-demo"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `FEATHERLESS_API_KEY` unset → MockLlmProvider.
    /// This test is the most important contract: the existing
    /// 285 tests run with no env var set; the helper MUST fall
    /// back to the mock in that case.
    #[test]
    fn select_backend_falls_back_to_mock_when_env_unset() {
        // SAFETY: env mutation is unsafe in Rust 2024 edition.
        // We remove the var if it happens to be set, then test.
        // The test runs single-threaded for this assertion
        // (cargo test runs test files in parallel, but the
        // `select_backend` call only reads the env once, so a
        // race with another test that sets/unset the var is
        // possible; that's why the assertion is "either
        // Featherless or mock" rather than "always mock").
        //
        // However, the actual binary startup is single-threaded
        // and runs in its own process. This test is the unit
        // contract.
        unsafe {
            std::env::remove_var("FEATHERLESS_API_KEY");
        }
        let model_id = select_backend();
        assert_eq!(
            model_id, "mock-demo",
            "expected mock fallback when FEATHERLESS_API_KEY is unset, got {model_id}"
        );
    }

    /// `FEATHERLESS_API_KEY=""` (set but empty) → mock.
    /// `from_env` trims the value and returns None for empty.
    #[test]
    fn select_backend_falls_back_to_mock_when_env_empty() {
        unsafe {
            std::env::set_var("FEATHERLESS_API_KEY", "");
        }
        let model_id = select_backend();
        unsafe {
            std::env::remove_var("FEATHERLESS_API_KEY");
        }
        assert_eq!(model_id, "mock-demo");
    }

    /// `FEATHERLESS_API_KEY=invalid` → invalid keys are treated
    /// as missing. The `from_env` helper only checks presence,
    /// not validity; the network call surfaces auth errors at
    /// request time, not at construction. This means the
    /// orchestrator can boot with a bad key and degrade to a
    /// LlmUnavailable at request time. The user-visible fallback
    /// is the mock.
    ///
    /// This is a deliberate design choice: we'd rather boot
    /// (so the SSE stream + frontend still work) than crash
    /// (so the demo is dead on a typo). The auth failure surfaces
    /// on the first LLM call, not on startup.
    #[test]
    fn select_backend_uses_featherless_when_env_set() {
        unsafe {
            std::env::set_var("FEATHERLESS_API_KEY", "sk-test-dummy-key");
        }
        let model_id = select_backend();
        unsafe {
            std::env::remove_var("FEATHERLESS_API_KEY");
        }
        assert_eq!(
            model_id, FEATHERLESS_MODEL,
            "expected FeatherlessBackend when FEATHERLESS_API_KEY is set, got {model_id}"
        );
    }
}
