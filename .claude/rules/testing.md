# Testing — MOIRAI v4

## Minimum Coverage: 70% (relaxed from generic 80% for hackathon speed)

We're in a 1-day sprint. Tests are gates, not goals.

## Test Types (ALL required for ACs that need them)

1. **Unit tests** — every crate has `#[cfg(test)] mod tests` for its public API
2. **Integration tests** — `tests/` dir in each crate for cross-crate flows
3. **Property tests** (optional) — for compression and chain hashing
4. **Snapshot tests** (use `insta`) — for PRC JSON output structure

## Edge Cases to Test

Every public function must be tested with:
- [ ] Happy path with representative input
- [ ] Empty input
- [ ] Invalid input (e.g., malformed PR URL, missing field)
- [ ] Boundary values (sequence 0, sequence u64::MAX, very long PR diff)
- [ ] Error conditions (Band WS disconnected, LLM rate limited, chain already at genesis)

## Test Quality Checklist

- [ ] Tests are independent (no shared state, no global fixtures)
- [ ] Test names describe behavior (`test_halt_fires_on_risk_score_above_threshold`, not `test1`)
- [ ] Mocks used for external dependencies (Band API, LLM providers)
- [ ] Both happy path and error paths tested
- [ ] No flaky tests (no `sleep`, no timing-dependent assertions)
- [ ] Tests run in <30s total (`cargo test --workspace` should be fast)

## AC-Specific Tests

- **AC4/5 (AI slop precision/recall)**: requires the 5 demo PRs to have a known gold label. Write the labels FIRST, then the test.
- **AC6 (Security HALT deterministic)**: 10/10 runs. Use a counter and assert == 10.
- **AC7 (token reduction)**: snapshot test the input/output token counts with and without Compressor.
- **AC8 (cost per run)**: integrate with a mock LLM that returns a known token count, then assert the USD cents in `CostBreakdown.total_usd_cents`.
- **AC11 (BAAAR HALT deterministic)**: 10/10 runs of the security PR.
- **AC13 (PRC offline verification)**: integration test that calls the verify function and asserts success in <30s.
- **AC15 (EU AI Act ≥7/8 fields)**: each `PrReviewCertificate` in the test fixtures must have the field populated.

## What NOT to test in 1-day sprint

- Exhaustive fuzz testing (the HashChain is provably correct; no need)
- E2E browser tests (Playwright is WS16, but manual demo is the gate)
- Performance benchmarks at multiple sizes (one benchmark at 100 PRs is enough)
- 100% line coverage (70% is the gate)

## [MOIRAI-specific]

- **Mock LLM providers**: `rig-core` 0.38 has provider traits. Implement a `MockProvider` that returns canned responses for tests.
- **Mock BandClient**: implement against the `BandClient` trait, no network. Use `MockBandClient` in every agent test.
- **Chain determinism**: the BLAKE3 chain is fully deterministic given the inputs. Test by constructing two chains from the same input and asserting identical hashes.
- **PRC JSON shape**: snapshot the JSON output of every test PRC to detect regressions in the EU AI Act Art. 12 field set.
