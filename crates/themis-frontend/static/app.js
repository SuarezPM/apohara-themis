/* THEMIS demo UI · vanilla JS · live counter + transcript + BAAAR overlay.
 *
 * Connects to /events (SSE) when the backend implements it. Until
 * then the page is fully static + functional: the submit button
 * triggers a deterministic mock run that exercises the entire UI
 * (counter ticks, transcript appends, BAAAR HALT overlay fires,
 * evidence card populates). When the orchestrator wires
 * EventSource in Phase F, replace `runMockRun()` with a real
 * `EventSource('/events')` listener.
 */

(() => {
  'use strict';

  // --- DOM helpers ---
  const $ = (sel, root = document) => root.querySelector(sel);
  const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));
  const fmt = new Intl.NumberFormat('en-US');
  const fmtUsd = (n) => `$${n.toFixed(4)}`;

  // --- 8 button states ---
  const setButtonState = (btn, state, label) => {
    btn.dataset.state = state;
    if (label !== undefined) {
      const lbl = btn.querySelector('.btn__label');
      if (lbl) lbl.textContent = label;
    }
    btn.disabled = state === 'loading' || state === 'disabled';
  };

  // --- Append a transcript line ---
  const appendTranscript = ({ from, body, tsMs, halt = false }) => {
    const list = $('#transcript-list');
    const empty = list.querySelector('.transcript__empty');
    if (empty) empty.remove();
    const li = document.createElement('li');
    li.className = 'transcript__msg' + (halt ? ' transcript__msg--halt' : '');
    const ts = new Date(tsMs).toISOString().replace('T', ' ').slice(0, 19);
    li.innerHTML = `
      <span class="transcript__ts">${ts}</span>
      <span class="transcript__from${halt ? ' transcript__from--halt' : ''}">@${from}</span>
      <p class="transcript__body" style="grid-column: 1 / -1;">${body}</p>
    `;
    list.appendChild(li);
    // Live count
    const n = $('#n6-event-count');
    n.textContent = String(parseInt(n.textContent, 10) + 1);
  };

  // --- Cell state + token/cost update ---
  const setCell = (agent, fields, state) => {
    const cell = $(`.cell[data-agent="${agent}"]`);
    if (!cell) return;
    if (state) cell.dataset.state = state;
    if (fields) {
      for (const [k, v] of Object.entries(fields)) {
        const el = cell.querySelector(`[data-k="${k}"]`);
        if (el) el.textContent = v;
      }
    }
  };

  // --- BAAAR HALT overlay ---
  const showHalt = ({ reason, trigger, agent, tenant, invoice, tsMs }) => {
    const ov = $('#halt-overlay');
    $('#halt-reason').textContent = reason || '—';
    $('#halt-trigger').textContent = trigger || '—';
    $('#halt-agent').textContent = agent || '—';
    $('#halt-tenant').textContent = tenant || '—';
    $('#halt-invoice').textContent = invoice || '—';
    $('#halt-ts').textContent = new Date(tsMs).toISOString().replace('T', ' ').slice(0, 19);
    ov.hidden = false;
  };
  const hideHalt = () => { $('#halt-overlay').hidden = true; };

  // --- Evidence card ---
  const populateEvidence = ({ status, tenant, invoice, decisions, coverage }) => {
    const card = $('#evidence-summary');
    card.dataset.state = 'ready';
    card.querySelector('[data-k="status"]').textContent = status;
    card.querySelector('[data-k="tenant"]').textContent = tenant;
    card.querySelector('[data-k="invoice"]').textContent = invoice;
    card.querySelector('[data-k="decisions"]').textContent = decisions;
    card.querySelector('[data-k="coverage"]').textContent = coverage;
    $('#download-pdf-btn').disabled = false;
    $('#download-json-btn').disabled = false;
  };

  // --- Mock run: deterministic per-fixture ---
  // Per the plan, the demo URL is read-only and stateless, so a
  // client-side mock is the right shape until the orchestrator
  // wires the real EventSource.
  const FIXTURES = {
    'clean-001': {
      tenant: 'stark', invoice: 'inv-clean-001',
      outcome: 'approved',
      agents: [
        { name: 'extractor', in: 2400, out: 180 },
        { name: 'po_matcher', in: 0, out: 0, note: 'no LLM — deterministic' },
        { name: 'fraud_auditor', in: 1200, out: 220 },
        { name: 'gaap_classifier', in: 1800, out: 200 },
        { name: 'provenance_signer', in: 0, out: 0, note: 'Ed25519' },
      ],
    },
    'gouge-001': {
      tenant: 'stark', invoice: 'inv-gouge-001',
      outcome: 'halted', halt: {
        reason: 'Risk score 0.95 exceeds 0.85 threshold',
        trigger: 'risk_score_exceeded', agent: 'fraud_auditor',
      },
      agents: [
        { name: 'extractor', in: 2400, out: 180 },
        { name: 'po_matcher', in: 0, out: 0, note: 'delta +200%' },
        { name: 'fraud_auditor', in: 1200, out: 220, halt: true },
        { name: 'gaap_classifier', in: 0, out: 0, skipped: true },
        { name: 'provenance_signer', in: 0, out: 0, skipped: true },
      ],
    },
    'phantom-001': {
      tenant: 'stark', invoice: 'inv-phantom-001',
      outcome: 'halted', halt: { reason: 'Phantom vendor — PO not in DB', trigger: 'phantom_vendor', agent: 'fraud_auditor' },
      agents: [
        { name: 'extractor', in: 2400, out: 180 },
        { name: 'po_matcher', in: 0, out: 0, note: 'no match' },
        { name: 'fraud_auditor', in: 1200, out: 220, halt: true },
        { name: 'gaap_classifier', in: 0, out: 0, skipped: true },
        { name: 'provenance_signer', in: 0, out: 0, skipped: true },
      ],
    },
    'math-001': {
      tenant: 'wayne', invoice: 'inv-math-001',
      outcome: 'halted', halt: { reason: 'Line items do not sum to total (sum=42000, total=50000)', trigger: 'math_fraud', agent: 'fraud_auditor' },
      agents: [
        { name: 'extractor', in: 2400, out: 180 },
        { name: 'po_matcher', in: 0, out: 0, note: 'matches' },
        { name: 'fraud_auditor', in: 1200, out: 220, halt: true },
        { name: 'gaap_classifier', in: 0, out: 0, skipped: true },
        { name: 'provenance_signer', in: 0, out: 0, skipped: true },
      ],
    },
    'duplicate-001': {
      tenant: 'wayne', invoice: 'inv-dup-001',
      outcome: 'halted', halt: { reason: 'Duplicate of inv-2026-05-28-7 (same vendor+amount+date)', trigger: 'duplicate', agent: 'fraud_auditor' },
      agents: [
        { name: 'extractor', in: 2400, out: 180 },
        { name: 'po_matcher', in: 0, out: 0, note: 'matches' },
        { name: 'fraud_auditor', in: 1200, out: 220, halt: true },
        { name: 'gaap_classifier', in: 0, out: 0, skipped: true },
        { name: 'provenance_signer', in: 0, out: 0, skipped: true },
      ],
    },
  };

  // Per-agent cost rates (USD per 1K tokens) — match the plan's
  // USD/run table. Order: [in_rate, out_rate].
  const COST = {
    extractor:           [0.00025, 0.0015],  // gemini-3.1-flash-lite
    po_matcher:          [0.0,     0.0],     // qwen3-coder-30b (Featherless flat)
    fraud_auditor:       [0.003,   0.015],   // claude-sonnet-4.6
    gaap_classifier:     [0.0,     0.0],     // glm-5.1 (Featherless flat)
    provenance_signer:   [0.001,   0.005],   // claude-haiku-4.5
  };
  const costOf = (agent, inT, outT) => {
    const [ir, or] = COST[agent] || [0, 0];
    return (inT / 1000) * ir + (outT / 1000) * or;
  };

  let totalIn = 0, totalOut = 0, totalCost = 0;
  const reset = () => { totalIn = totalOut = totalCost = 0; };

  const runMockRun = async (fixtureId) => {
    const fx = FIXTURES[fixtureId];
    if (!fx) return;
    const tenant = $('#tenant-switch').value;
    const btn = $('#submit-btn');
    setButtonState(btn, 'loading', 'Running…');
    reset();
    // Reset cells
    $$('.cell').forEach(c => {
      c.dataset.state = 'default';
      c.querySelectorAll('[data-k]').forEach(d => d.textContent = '—');
    });
    // Reset evidence
    const ev = $('#evidence-summary');
    ev.dataset.state = 'empty';
    ev.querySelectorAll('[data-k]').forEach(d => d.textContent = '—');
    $('#download-pdf-btn').disabled = true;
    $('#download-json-btn').disabled = true;
    hideHalt();
    // Reset transcript
    const list = $('#transcript-list');
    list.innerHTML = '<li class="transcript__empty">No events yet — submit an invoice to start the debate.</li>';
    $('#n6-event-count').textContent = '0';

    // 1. Extractor
    setCell('extractor', {}, 'running');
    await sleep(280);
    appendTranscript({ from: 'extractor', body: 'Parsed 3 line items. vendor=Acme, amount=$450.00', tsMs: nowMs() });
    const ext = fx.agents[0];
    setCell('extractor', {
      in: fmt.format(ext.in),
      out: fmt.format(ext.out),
      cost: fmtUsd(costOf('extractor', ext.in, ext.out)),
    }, 'done');
    totalIn += ext.in; totalOut += ext.out; totalCost += costOf('extractor', ext.in, ext.out);

    // 2. PO Matcher (deterministic, no LLM)
    setCell('po_matcher', {}, 'running');
    await sleep(140);
    appendTranscript({ from: 'po_matcher', body: 'PO-12345 matches. delta=0%.', tsMs: nowMs() });
    setCell('po_matcher', { in: '—', out: '—', cost: '$0.0000' }, 'done');

    // 3. Fraud Auditor
    setCell('fraud_auditor', {}, 'running');
    await sleep(320);
    const aud = fx.agents[2];
    const haltAtAuditor = !!aud.halt;
    appendTranscript({
      from: 'fraud_auditor',
      body: haltAtAuditor ? `HALT: ${fx.halt.reason}` : 'risk_score=0.42. no findings. coherence=0.81. outcome=approve.',
      tsMs: nowMs(),
      halt: haltAtAuditor,
    });
    setCell('fraud_auditor', {
      in: fmt.format(aud.in),
      out: fmt.format(aud.out),
      cost: fmtUsd(costOf('fraud_auditor', aud.in, aud.out)),
    }, haltAtAuditor ? 'halted' : 'done');
    totalIn += aud.in; totalOut += aud.out; totalCost += costOf('fraud_auditor', aud.in, aud.out);

    if (haltAtAuditor) {
      // Show BAAAR HALT overlay; skip downstream.
      showHalt({
        reason: fx.halt.reason,
        trigger: fx.halt.trigger,
        agent: 'fraud_auditor',
        tenant,
        invoice: fx.invoice,
        tsMs: nowMs(),
      });
      setCell('gaap_classifier', { in: '—', out: '—', cost: '—' }, 'default');
      setCell('provenance_signer', { in: '—', out: '—', cost: '—' }, 'default');
      populateEvidence({
        status: 'HALTED (BAAAR)',
        tenant: tenant,
        invoice: fx.invoice,
        decisions: '2 of 5 (Extractor, POMatcher) + 1 halted (FraudAuditor)',
        coverage: '7/7 frameworks (DORA/EU AI Act/NIST/OWASP) — packet is sealed but HALTED',
      });
      appendTranscript({ from: 'provenance_signer', body: 'Sealed the Evidence Packet with halt evidence; signature stored in keys/' + tenant + '.ed25519.', tsMs: nowMs() });
      appendTranscript({ from: 'audit_watchdog', body: 'BAAAR HALT detected in upstream decision. Coherence=0.78.', tsMs: nowMs() });
      appendTranscript({ from: 'regression_tester', body: 'Re-verified Ed25519+BLAKE3: signature valid, hash chain intact.', tsMs: nowMs() });
      setButtonState(btn, 'success', 'Halted · see receipt');
      setTimeout(() => setButtonState(btn, 'default', 'Run audit'), 2400);
      return;
    }

    // 4. GAAP Classifier
    setCell('gaap_classifier', {}, 'running');
    await sleep(220);
    const gaap = fx.agents[3];
    appendTranscript({ from: 'gaap_classifier', body: 'Classified 3 line items to US-GAAP 6100 (Operating Expenses).', tsMs: nowMs() });
    setCell('gaap_classifier', {
      in: fmt.format(gaap.in),
      out: fmt.format(gaap.out),
      cost: fmtUsd(costOf('gaap_classifier', gaap.in, gaap.out)),
    }, 'done');
    totalIn += gaap.in; totalOut += gaap.out; totalCost += costOf('gaap_classifier', gaap.in, gaap.out);

    // 5. Provenance Signer
    setCell('provenance_signer', {}, 'running');
    await sleep(180);
    appendTranscript({ from: 'provenance_signer', body: 'BLAKE3 hash computed. Ed25519 signed. RFC 3161 timestamp requested.', tsMs: nowMs() });
    setCell('provenance_signer', { in: '—', out: '—', cost: '$0.0000' }, 'done');

    // Shadows
    appendTranscript({ from: 'audit_watchdog', body: 'Chain coherent. mean confidence=0.86.', tsMs: nowMs() });
    appendTranscript({ from: 'demo_narrator', body: 'Invoice processed and approved.', tsMs: nowMs() });
    appendTranscript({ from: 'regression_tester', body: 'Re-verified Ed25519+BLAKE3: signature valid, hash chain intact.', tsMs: nowMs() });

    populateEvidence({
      status: 'APPROVED',
      tenant: tenant,
      invoice: fx.invoice,
      decisions: '5 of 5 + 3 shadows',
      coverage: '7/7 frameworks (DORA/EU AI Act/NIST/OWASP)',
    });
    setCell('total', {
      in: fmt.format(totalIn),
      out: fmt.format(totalOut),
      cost: fmtUsd(totalCost),
    });
    setButtonState(btn, 'success', 'Sealed · see receipt');
    setTimeout(() => setButtonState(btn, 'default', 'Run audit'), 2400);
  };

  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const nowMs = () => Date.now();

  // --- Wire up ---
  const form = $('#submit-form');
  form.addEventListener('submit', (e) => {
    e.preventDefault();
    const fixtureId = $('#invoice-fixture').value;
    runMockRun(fixtureId);
  });

  $('#halt-dismiss-btn').addEventListener('click', hideHalt);

  // --- Download buttons (mock: open a Blob URL with the card's
  //     current content; real wiring is in the orchestrator) ---
  const downloadEvidence = (fmt) => {
    const card = $('#evidence-summary');
    const payload = {
      schema: 'themis.evidence-packet.v1',
      tenant: card.querySelector('[data-k="tenant"]').textContent,
      invoice: card.querySelector('[data-k="invoice"]').textContent,
      status: card.querySelector('[data-k="status"]').textContent,
      decisions: card.querySelector('[data-k="decisions"]').textContent,
      coverage: card.querySelector('[data-k="coverage"]').textContent,
      generated_at_ms: Date.now(),
      note: 'Demo fixture — real signature in production. Verify with themis-verify.',
    };
    let blob, name;
    if (fmt === 'json') {
      blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
      name = `evidence-${payload.invoice}.json`;
    } else {
      const text = `THEMIS Evidence Packet (demo)\n\n${Object.entries(payload).map(([k, v]) => `${k}: ${v}`).join('\n')}\n`;
      blob = new Blob([text], { type: 'text/plain' });
      name = `evidence-${payload.invoice}.txt`;
    }
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = name;
    document.body.appendChild(a); a.click(); a.remove();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  $('#download-json-btn').addEventListener('click', () => downloadEvidence('json'));
  $('#download-pdf-btn').addEventListener('click', () => downloadEvidence('pdf'));
  $('#halt-download-btn').addEventListener('click', () => downloadEvidence('json'));

  // Footer version + commit (placeholder; orchestrator will inject
  // these at build time via index.html rewrite).
  const params = new URLSearchParams(window.location.search);
  if (params.has('v')) $('#ft7-version').textContent = params.get('v');
  if (params.has('sha')) $('#ft7-commit').textContent = params.get('sha').slice(0, 7);
})();
