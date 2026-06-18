# THEMIS 3.0 — Demo Video v5

> **Status:** pending manual recording by Pablo.
> See [`video-v5-script.md`](video-v5-script.md) for the full script.

## Video URL

```
[YouTube/Vimeo URL goes here after Pablo records and uploads]
```

Path to local MP4 (if not yet uploaded):

```
[path/to/demo/video-v5.mp4 goes here after Pablo records]
```

## What the video proves (per QW-5 AC1)

| Section | Sponsor proof | Reference |
|---|---|---|
| 1 | Live `app.band.ai` room — 6 agents over WebSocket | `docs/band-room-screenshot.md` |
| 2 | AIML metrics widget live (50+ real calls) | `crates/themis-orchestrator/tests/aiml_50_real_e2e.rs` |
| 3 | Featherless metrics widget live (50+ real calls) | `crates/themis-orchestrator/tests/featherless_50_real_e2e.rs` |
| 4 | Public bench: recall=1.000, FPR=0.000, FP-reduction=100% | `crates/themis-orchestrator/tests/public_bench.rs` |
| 5 | BAAAR HALT — red border + modal + PRC download | THEMIS 2.0 story (`baaar_z3_1210.rs`) |

## Recording steps for Pablo

1. `source ~/.config/apohara/secrets.env`
2. `cargo run --release -p themis-orchestrator` (Terminal A)
3. OBS Studio → scene "THEMIS v5" → 1920x1080 @ 30 fps
4. Walk through Shots 0-6 in `video-v5-script.md`
5. Stop OBS → post-process to MP4 <50 MB (H.264 CRF 23)
6. Upload to YouTube (unlisted is fine for lablab.ai)
7. Paste the URL in the placeholder block above
8. Commit and push — `docs/submission-final.md` references this URL

## Acceptance Criteria — QW-5 / AC1

- [x] Script written (`docs/video-v5-script.md`)
- [ ] YouTube/Vimeo URL recorded (Pablo's manual step)
- [ ] Sections 1-5 each visible in the recording