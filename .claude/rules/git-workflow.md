# Git Workflow — MOIRAI v4

## Commit Message Format

```
<type>: <description>

<optional body>
```

Types: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`, `perf`, `ci`

Examples from this repo:
- `feat(atropos): add BAAAR HALT emit on risk_score > 0.85`
- `fix(chain): use hex encoding for blake3 Hash to support serde`
- `docs(spec): persist v4 spec to docs/SPEC.md`
- `chore: cargo update -p turbovec`

## Branch Naming

- `feature/atropos-verdict` — new feature
- `fix/chain-reorder-buffer` — bug fix
- `refactor/compressor-agora` — refactor
- `docs/spec-v5` — doc update
- `pr/rust-sdk-band` — PR work to Band (branch lives in our fork until merged)

## Workstream → Branch Strategy

Phase 1 has 6 workstreams (WS1-WS6 + WS7 + WS8). Each can be a branch:

- `ws/band-client` (WS1)
- `ws/clotho` (WS2)
- `ws/lachesis` (WS3)
- `ws/eris` (WS4)
- `ws/compressor` (WS5)
- `ws/vindex` (WS6)
- `ws/atropos` (WS7)
- `ws/evidence` (WS8)

Integration happens on `main` after each workstream merges its PR.

## Daily Commits (Pablo's rule from Synthex v2)

The jury looks at commit history. **All commits visible from Day 1, not a single push at the end.**

- Each workstream produces 5-10 commits per day
- Each commit message links the AC it advances: `[AC2][AC7] feat(atropos): add BAAAR HALT broadcast`
- "all commits on last day" is a signal of last-minute scramble. Avoid.

## Pre-commit Checklist

- [ ] `cargo check --workspace` exits 0
- [ ] `cargo clippy --workspace --all-targets` no new warnings
- [ ] `cargo test --workspace` passes
- [ ] No secrets in diff (`.env`, `*.pem`, `*.key`)
- [ ] Commit message references AC if applicable

## PR to Band workflow

1. Create the PR branch from `main`
2. Push to a personal fork (e.g., `SuarezPM/band-sdk-rs`)
3. Open PR to `thenvoi/codeband` or `band-ai/band-sdk-rs`
4. **Unmerged PR IS the story** (per SCEPTRE v2 pattern). Merge is bonus, not blocker.
5. Document the PR URL in `docs/REFERENCES.md` for the submission

## Git identity

- `user.name "Pablo"`
- `user.email "suarezpm@csnat.unt.edu.ar"` (real, SuarezPM-attributed)
- **NEVER use `dimensionequix@gmail.com`** — that email is registered on the old `pms008` GitHub account, attributes commits to the wrong identity.

## [MOIRAI-specific]

- **No `force push`** on `main`. The commit history is the demo's audit trail.
- **No squashing** of workstream PRs. Each commit is a step in the spec.
- **Tags** at major milestones: `v0.1.0-pre-kickoff`, `v0.2.0-demo-deployable`, `v0.3.0-submission`, `v0.4.0-post-hackathon`.
