// scripts/verify-themis.mjs — ad-hoc visual + axe verify for themis-frontend
//
// Mirrors scripts/verify-argus.mjs. Boots its own python http.server on 8774,
// navigates 2 viewports, captures screenshots into .qa/, runs axe-core wcag2aa,
// and prints structural-element checks. Verifies that the app.js DOM contract
// (cells, transcript, halt overlay, evidence card, etc.) is intact.

import { chromium } from 'playwright';
import AxeBuilder from '@axe-core/playwright';
import { spawn } from 'node:child_process';
import { mkdir } from 'node:fs/promises';
import { setTimeout as wait } from 'node:timers/promises';

const PORT = 8774;
const BASE = `http://127.0.0.1:${PORT}`;
const QA_DIR = 'public/.qa';
const VIEWPORTS = [
  { w: 1440, h: 900,  tag: '1440' },
  { w: 375,  h: 812,  tag: '375'  },
];

async function waitForServer(url, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(url, { method: 'HEAD' });
      if (res.status < 500) return true;
    } catch { /* not up yet */ }
    await wait(150);
  }
  return false;
}

await mkdir(QA_DIR, { recursive: true });
const server = spawn('python3', ['-m', 'http.server', String(PORT), '--directory', 'public'], {
  stdio: 'ignore',
  detached: true,
});

try {
  if (!await waitForServer(`${BASE}/`)) {
    console.error(`[verify-themis] server failed to start on :${PORT}`);
    process.exit(2);
  }
  console.log(`[verify-themis] server up on :${PORT}`);

  let exitCode = 0;

  for (const vp of VIEWPORTS) {
    const browser = await chromium.launch();
    const ctx = await browser.newContext({ viewport: { width: vp.w, height: vp.h } });
    const page = await ctx.newPage();
    const errs = [];
    page.on('pageerror', e => errs.push(`PAGE: ${e.message}`));
    page.on('console', m => { if (m.type() === 'error') errs.push(`CON: ${m.text()}`); });

    await page.goto(`${BASE}/`, { waitUntil: 'networkidle' });
    await page.waitForSelector('[data-qa-ready]');
    // wait extra for web fonts so axe sees resolved OKLCH
    await wait(1500);
    await page.screenshot({ path: `${QA_DIR}/${vp.tag}.png`, fullPage: true });

    const axe = await new AxeBuilder({ page }).withTags(['wcag2a', 'wcag2aa']).analyze();
    const blocking = axe.violations.filter(v => ['serious', 'critical'].includes(v.impact));
    console.log(`[${vp.tag}] axe: ${axe.violations.length} total · ${blocking.length} blocking`);
    for (const v of axe.violations) {
      console.log(`  [${v.impact}] ${v.id}: ${v.help} (${v.nodes.length} nodes)`);
    }
    if (errs.length) {
      console.log(`[${vp.tag}] CONSOLE ERRORS:`);
      errs.forEach(e => console.log('  ' + e));
      exitCode = 1;
    }

    const horiz = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth);
    const checks = {
      'no horizontal scroll':          !horiz,
      'nav-island':                    !!(await page.$('.nav-island')),
      '4-framework stat-led':          (await page.$$('.stat-row .stat-cell')).length === 4,
      'submit form':                   !!(await page.$('#submit-form')),
      'tenant switch':                 !!(await page.$('#tenant-switch')),
      '5 agent cells':                 (await page.$$('[id^="cell-"]')).length === 6,
      'transcript list':               !!(await page.$('#transcript-list')),
      'halt overlay (hidden)':         !!(await page.$('#halt-overlay')),
      'evidence card':                 !!(await page.$('#evidence-summary')),
      'download buttons':              (await page.$$('#download-pdf-btn, #download-json-btn')).length === 2,
      'compliance link':               !!(await page.$('#compliance-link')),
      'n6 sticky (tenant + live)':     !!(await page.$('nav.n6')),
      'shared footer with circuit':    !!(await page.$('footer.footer .circuit')),
      'aria-current on themis':        !!(await page.$('.island-links a[aria-current="page"]')),
    };
    for (const [k, v] of Object.entries(checks)) console.log(`  ${v ? '✓' : '✕'} ${k}`);

    const accent = await page.$eval('body', () => getComputedStyle(document.body).getPropertyValue('--accent').trim());
    console.log(`  --accent: ${accent}`);

    // also verify compliance page (vercel cleanUrls resolves /compliance → compliance.html;
    // python http.server needs the .html suffix explicitly)
    if (vp.tag === '1440') {
      const page2 = await ctx.newPage();
      await page2.goto(`${BASE}/compliance.html`, { waitUntil: 'networkidle' });
      await page2.waitForSelector('[data-qa-ready]');
      await wait(1500);
      await page2.screenshot({ path: `${QA_DIR}/compliance-1440.png`, fullPage: true });
      const axe2 = await new AxeBuilder({ page: page2 }).withTags(['wcag2a', 'wcag2aa']).analyze();
      const blocking2 = axe2.violations.filter(v => ['serious', 'critical'].includes(v.impact));
      console.log(`[compliance] axe: ${axe2.violations.length} total · ${blocking2.length} blocking`);
      for (const v of axe2.violations) {
        console.log(`  [${v.impact}] ${v.id}: ${v.help} (${v.nodes.length} nodes)`);
      }
      const frameCount = (await page2.$$('.framework')).length;
      console.log(`  compliance frameworks: ${frameCount} (expect 5)`);
      await page2.close();
    }

    await browser.close();
  }

  process.exit(exitCode);
} finally {
  try { process.kill(-server.pid, 'SIGTERM'); } catch { /* already gone */ }
}
