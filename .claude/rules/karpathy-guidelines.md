# Karpathy Coding Guidelines (adapted for MOIRAI)

Behavioral guidelines to reduce common LLM coding mistakes. Derived from Andrej Karpathy's
observations on LLM coding pitfalls. These principles bias toward caution over speed — for
trivial tasks, use judgment. **In a hackathon, the bar is "ship something that works,"
not "ship something perfect."**

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:
- State your assumptions explicitly. If uncertain, ask Pablo.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

For MOIRAI: when the spec says "AGORA-like compression" and you're not sure which AGORA
variant, ask before implementing. The 1-day sprint doesn't leave room for backtracking.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

For MOIRAI: 6 workstreams, 22h wall-clock, 18 ACs. **Every LOC is a cost.** Don't write a
generic `Agent` framework that supports 17 agent types. Write 6 specific agents.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

For MOIRAI: the scaffold (traits, structs, Cargo manifests) is committed. **Don't refactor
it during Phase 1 unless an AC demands it.** Add to it, don't rewrite it.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:
- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make the test pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require
constant clarification.

For MOIRAI: the 18 ACs ARE the success criteria. Each WS has its own subset. Tie commits
to ACs explicitly: `[AC2][AC7] feat(atropos): BAAAR HALT broadcast`.

## 5. Truthfulness About Completion

**If you didn't do it, don't say you did.**

Before claiming a task is done:
- Did `cargo check --workspace` exit 0? If not, the workspace is broken, not "done."
- Did `cargo test` pass? If you didn't run tests, you don't know.
- Did the ACs in scope pass? If you didn't measure, you don't know.

For MOIRAI: the 18 ACs are the contract. "Demo works" is meaningless. "AC2 passes
with the security PR in <90s" is meaningful.

## 6. The 1-Day Sprint Discipline

**Pablo said: "el tiempo no es un problema, ni una excusa, ni un impedimento."**

This doesn't mean "ship fast and break things." It means:
- **Don't pad estimates with auto-nerfing buffer.** If you think 1 hour, say 1 hour.
- **Don't hide uncertainty behind a longer timeline.** If something is hard, say so now.
- **Don't gold-plate.** Phase 0/1 is the trait surface + 6 agents working. Phase 2 is integration. Phase 3 is polish. Don't put polish in Phase 1.
- **But also: don't ship broken.** An AC that fails is not "good enough for the deadline." Plan B exists for a reason.

For MOIRAI: Plan B = drop Compressor (loses AC7), drop Vindex (loses demo richness), drop Rust SDK (loses PR narrative). The priority order is in the spec. **Use Plan B if Phase 1 doesn't finish by hour 5.**

## 7. Use Pablo's Brain (the Hackathon Brain)

The brain is `~/apohara-hackathon-brain/`. It's where research, iteration, and competitor
analysis live. **Don't redo research the brain already did.** When the spec is unclear,
search the brain first.

For MOIRAI: the brain has the deep research intel report, the stack analysis, the
competitor map, and 5+ iterations of the project idea. **Read it before writing new
analysis.**

## 8. The Compressor is Sacred

**v4's differentiator is the Compressor.** Token economy is the explicit pain. The 30%
reduction claim (AC7) is what separates MOIRAI v4 from "another 5-agent debate system."

If a change risks the Compressor's correctness or measurability, surface it before doing it.
If a Plan B decision drops the Compressor, document the cost.

## 9. The BAAAR HALT is Sacred

**The HALT is the wow moment of the demo.** If the HALT doesn't fire visibly in the
WebSocket broadcast in <90s, the demo loses its punch.

If a refactor risks the HALT firing incorrectly (false positives, false negatives), surface
it before doing it. Test AC11 (10/10 deterministic) before merging the change.
