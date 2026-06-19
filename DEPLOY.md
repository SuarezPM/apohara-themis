# Deploy Apohara VOUCH to `vouch.apohara.dev`

This repo is **deploy-ready**. The Vercel + Fly deploy was operated by
`@SuarezPM`; this document is the runbook so it can be reproduced
or handed off.

## Architecture (two surfaces)

```
Browser --> https://vouch.apohara.dev     (Vercel static UI)
              |
              | rewrites in vercel.json
              v
Backend  --> https://themis-orchestrator.fly.dev (Fly.io Rust binary)
              |
              v
         Band chat room (9 agents) + AIML API + Featherless
```

The Vercel surface serves the static `crates/vouch-frontend/`
HTML+CSS+JS as a single-page app. The backend is the
`themis-orchestrator` Rust binary running on Fly.io (the
multi-tenant registry + SSE event stream + `/seal` endpoint).

The two surfaces talk via the rewrites in `vercel.json`:
- `POST /invoices`         -> backend start
- `GET /packets/:id/pdf`   -> backend PDF download
- `GET /packets/:id/json`  -> backend JSON packet
- `GET /compliance-report/:id` -> backend compliance summary
- `GET /events`            -> backend SSE stream (rewritten by Vercel)

## What's already in this repo

- `vercel.json` — Vercel project config (build command, output
  dir, rewrites, security headers).
- `scripts/vercel-build.sh` — builds the static frontend into
  `public/`.
- `crates/vouch-frontend/static/` — the HTML+CSS+JS that
  `vercel-build.sh` copies into `public/`.
- `crates/themis-orchestrator/src/bin/themis-orchestrator.rs`
  — the backend binary that runs on Fly.
- `crates/themis-band-client/agent-config/agent_config.yaml`
  — the Band room config that all 9 agents register against.
- `Cargo.toml` — workspace root; `cargo build --release` from
  repo root builds both binaries.

## Vercel deploy steps

```bash
# 1. Create the Vercel project (one-time).
vercel link --repo SuarezPM/apohara-vouch

# 2. In the Vercel dashboard, set:
#    - Build Command:    bash scripts/vercel-build.sh
#    - Output Directory: public
#    - Install Command:  (leave default)
#    No env vars needed on Vercel side; the frontend is pure static.

# 3. Add the custom domain `vouch.apohara.dev`.
vercel domains add vouch.apohara.dev

# 4. At apohara.dev's DNS provider, point `vouch` CNAME to
#    `cname.vercel-dns.com` (Vercel will print the exact value).

# 5. Deploy:
vercel --prod
```

## Fly.io backend steps

```bash
# 1. Create the Fly app (one-time).
fly apps create themis-orchestrator
fly volumes create vouch_data --size 1

# 2. Set secrets (read from ~/.config/apohara/secrets.env).
fly secrets set \
  AIML_API_KEY=$AIML_API_KEY \
  FEATHERLESS_API_KEY=$FEATHERLESS_API_KEY \
  BAND_AGENT_ORCHESTRATOR_ID=$BAND_AGENT_ORCHESTRATOR_ID \
  BAND_AGENT_ORCHESTRATOR_API_KEY=$BAND_AGENT_ORCHESTRATOR_API_KEY \
  BAND_AGENT_INTAKE_ID=$BAND_AGENT_INTAKE_ID \
  ... (8 more agent_id/api_key pairs, see agent_config.yaml)

# 3. Deploy.
fly deploy

# 4. The backend listens on 0.0.0.0:8080 by default.
```

## Verify the deploy

```bash
# Vercel (frontend):
curl -s -o /dev/null -w "HTTP %{http_code} | %{time_total}s\n" https://vouch.apohara.dev/

# Fly (backend via Vercel rewrite):
curl -s -X POST https://vouch.apohara.dev/invoices \
  -H "Content-Type: application/json" \
  -d '{"tenant_id":"stark","invoice_id":"smoke","raw_b64":""}' \
  -w "\nHTTP %{http_code} | %{time_total}s\n"
```

The CI workflow (`.github/workflows/ci.yml` job `live-deploy`)
runs these exact curls on every push to `main`, so a broken
deploy breaks CI too.

## Local sanity check

Before pushing, simulate the production wiring locally:

```bash
cargo build --release --workspace
./target/release/themis-orchestrator &
# In another terminal:
curl http://localhost:8080/                                 # serves index.html
curl -X POST http://localhost:8080/invoices \
  -H 'Content-Type: application/json' \
  -d '{"tenant_id":"stark","invoice_id":"local-smoke","raw_b64":""}'
```

If `localhost:8080` works, `vouch.apohara.dev` will work — the only
delta is the public URL + Vercel rewrites.

## Undo the deploy

If the demo URL needs to come down (e.g. key compromise):

```bash
vercel domains rm vouch.apohara.dev     # remove the domain binding
vercel --prod --no-wait                # last deploy stays, but no traffic
fly apps destroy themis-orchestrator   # tear down the backend
```

The repo + GitHub history stay intact; the README's `[![Demo]]`
badge can be removed in a follow-up commit if the demo goes away
permanently (see audit task T#7).
