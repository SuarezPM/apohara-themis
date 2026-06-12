# Visual Verdict — MOIRAI v4 (UI/UX)

The demo is a **judge opens URL, no instructions, 4 minutes**. Every UI choice is a
**verdict** the judge makes in <10 seconds. Make those verdicts land.

## The Verdict Ladder

1. **Is the page alive?** (cold start, see AC1)
2. **Does the UI say what the product is?** (tagline above the fold)
3. **Is the live counter visible?** (token economy proof, AC9)
4. **Does the BAAAR HALT fire visibly?** (the wow moment, AC10)
5. **Can the judge download the PRC?** (the artifact, AC12)
6. **Does the compliance dashboard show 7/8 Art. 12 fields?** (the regulator signal, AC15/16)

If any step fails, the demo loses the judge.

## Visual Rules

### Above the fold (no scroll required)
- Project name: **MOIRAI v4** in large type
- Tagline: "5 Greek Fates + 1 modern + 1 kill-switch"
- PR URL input: a single text field + "Review this" button
- The live counter: total tokens, total cost, per-agent cost — must be visible without scrolling

### During the demo
- **Band room transcript on the left** (scrollable, latest at bottom, auto-scroll)
- **Live counter on the right top** (per-agent + total)
- **Compliance dashboard on the right middle** (8 Art. 12 fields, ✓ or ✗)
- **PRC download button** (right bottom, only visible after Átropos verdict)
- **BAAAR HALT**: when it fires, the whole UI flashes a red border + a modal that says
  "WORKFLOW HALTED — secret detected — judge: download the receipt to verify offline"

### Color palette
- **Background**: deep navy (#0a0e1a) — agent/dark aesthetic
- **Accent (MOIRAI brand)**: gold (#d4a017) — Greek mythology
- **HALT**: red (#dc2626) — danger, halt, stop
- **APPROVED**: green (#10b981) — go
- **REVIEW_REQUIRED**: amber (#f59e0b) — caution
- **Text**: high-contrast white (#f9fafb) on dark; dark (#1f2937) on light surfaces

### Typography
- Monospace for: token counts, cost numbers, BLAKE3 hashes, signatures
- Sans-serif for: agent names, taglines, descriptions
- Use a single font family (Inter, system-ui, or JetBrains Mono for code)

## What NOT to do

- **No emoji as primary UI**. They work in decks, not in judges' first impression.
- **No "loading spinner" without a status message**. Judges don't know if it's working or hung.
- **No modal dialogs** blocking the demo. The HALT is the ONE exception, and only because it's the wow moment.
- **No dark-on-dark or light-on-light**. Always high contrast.
- **No fonts smaller than 14px**. Judges view on a projector; small text disappears.

## What TO do

- **Show the receipts literally on screen** during AC13 demo: copy the Ed25519 signature,
  verify with `openssl`, paste the result. **The judge needs to see you verify, not trust
  you verify.**
- **Show the BAAAR HALT timestamp and reason** in big text when it fires. "HALTED at
  14:23:07 — secret detected — review: 1 file, 3 lines, line 47".
- **Show the cost per agent live**. "Clotho: $0.12. Lachesis: $0.05. Eris: $0.17. Vindex:
  $0.18. Compressor: saved $0.20. Átropos: $0.95. Total: $1.49."
- **Show the EU AI Act coverage score** in the dashboard. "7/8 fields ✓ — Article 12
  compliant".

## Visual-Verdict Test

Before submitting the demo UI, run this checklist:

1. Open the URL in an incognito tab (no cache, no cookies, fresh eyes).
2. Without reading any text, look at the page for 3 seconds. Does the BAAAR branding register?
3. Type a PR URL, click "Review this". Does the UI show feedback in <2s? (AC1 cold start + first action)
4. Watch the live counter. Does it tick up before the agents respond? (proves the cost-tracking is real, not retrospective)
5. Wait for the BAAAR HALT (or trigger it manually with the security PR). Does the red border flash? Does the modal say WHY it halted?
6. Click "Download PRC". Does the PDF arrive in <2s? (AC12)
7. Open the PRC. Are the 8 EU AI Act Art. 12 fields visible at a glance? (AC15)

If any step is a "huh?" moment, the demo loses that judge.

## [MOIRAI-specific]

- **No framework**. Vanilla HTML + JS. `EventSource` for live updates from the Axum server.
- **No CSS framework** (no Tailwind, no Bootstrap). Inline styles or a single `<style>` block.
- **Single page**. The demo URL is `/`. The compliance dashboard is `/compliance`.
- **404 page** is a Band room: "MOIRAI doesn't know that route. Maybe it was halted?"
