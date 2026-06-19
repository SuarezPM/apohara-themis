# Contributing to Apohara VOUCH

Thanks for your interest in contributing. Apohara VOUCH is the
Band-of-Agents hackathon entry from `@SuarezPM`; the code is MIT
and we welcome PRs that improve correctness, security, or
documentation.

## Code of Conduct

This project follows the [Contributor Covenant](CODE_OF_CONDUCT.md).
By participating, you agree to its terms.

## Getting started

```bash
git clone https://github.com/SuarezPM/apohara-vouch.git
cd apohara-vouch
cargo build --release          # ~10s on first build
cargo test --workspace         # 815 Rust tests, must stay green
cargo clippy --workspace       # 0 warnings expected
cargo deny check               # license + advisory check
```

For the Python agents:

```bash
cd crates/vouch-agents
uv sync --frozen              # installs pinned deps from uv.lock
.venv/bin/python -m pytest tests/ -m "not chaos"   # 177 agent tests
.venv/bin/python -m pytest tests/ -m chaos         # 4 chaos tests
```

## Commit message format

We use [Conventional Commits](https://www.conventionalcommits.org/)
with a single concern per commit:

```
<type>(<scope>): <subject>

<body explaining the why, not the what>
```

Common types:

| Type | Use for |
|---|---|
| `feat` | New feature visible to a user |
| `fix` | Bug fix |
| `refactor` | Code change that doesn't add a feature or fix a bug |
| `docs` | Documentation only |
| `test` | New tests or test infrastructure |
| `chore` | Repo maintenance (CI, deps, gitignore) |
| `perf` | Performance improvement |
| `ci` | CI workflow changes |

Scope examples: `red_team`, `pdf`, `bench`, `frontend`,
`chaos`. When in doubt, leave the scope out.

## Pre-commit checklist

Before every commit, the pre-commit hook (configured by
`scripts/install-pre-commit.sh`) runs:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace --no-fail-fast`
- `cargo deny check`

If you want to bypass the hook for a WIP commit (don't, but if you
must), use `git commit --no-verify`. **Never push a commit that
fails these checks on `main`** — that defeats the audit-clean
contract the README cites.

## Branch strategy

- One branch per workstream (`feature/`, `fix/`, `refactor/`, `docs/`).
- Squash is fine; rebase is preferred.
- Force-push to `main` is blocked by the audit rule (commit history
  is part of the demo narrative — see `docs/SUBMISSION.md`).
- Force-push to feature branches is OK.

## Adding a new agent

If you're contributing a new Band specialist agent to
`crates/vouch-agents/`:

1. **Pick the right framework**. The 9 existing agents use 4 different
   frameworks — LangGraph (state machines), CrewAI (role-based),
   Pydantic AI (structured output), Anthropic SDK (tool use). Pick
   the one that fits the agent's role. **Do not introduce a 5th
   framework** without an entry in `docs/SUBMISSION.md` and a green
   `clippy + cargo test`.
2. **Add the agent to the Band registration** in
   `crates/themis-band-client/agent-config/agent_config.yaml`. The
   entry needs a unique UUID, an `agent_id`, an `api_key` slot (read
   from secrets.env at runtime; never commit), and a `chatroom` block.
3. **Wire the agent into the orchestrator** state machine in
   `crates/vouch-agents/src/orchestrator.py`. Follow the pattern of
   the existing 9 agents — `@mention` routing, deterministic
   decision payload, `Event::*` published to the bus.
4. **Add tests**. The bar is `cargo test --workspace` and
   `pytest crates/vouch-agents/tests/` both stay green. New agents
   should ship with at least:
   - Happy path (one valid invoice → expected decision).
   - Edge case (missing field → graceful degradation).
   - Chaos case (if the agent is on the cross-account path).
5. **Update the README** to mention the new agent in the
   "9-agent court" line and the sponsor integration table.

## Pull request template

A good PR:

- **Title** follows Conventional Commits (`feat(red_team): add
  secret-leak detection for invoice PDFs`).
- **Body** explains the why, not the what. The git diff shows the
  what.
- **Tests** are included or updated. `cargo test --workspace`
  passes locally before you open the PR.
- **AC references** for hackathon-related work: `[AC3][AC12] feat:
  ...`. The 18 acceptance criteria live in `docs/SPEC.md`.
- **No secrets** in the diff. The pre-commit hook blocks `.pem`,
  `.key`, `.env`, but please don't try to work around it.

## Reporting a security issue

See [SECURITY.md](./SECURITY.md) — please do NOT file a public
issue for a vulnerability. Email the maintainer out-of-band.

## License

By contributing, you agree that your contributions will be licensed
under the MIT License (see [LICENSE](./LICENSE)).
