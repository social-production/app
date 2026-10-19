// How long until a new person shows up on someone else's People page (docs/NETWORK.md).
//   node tests/e2e/people-lag.mjs https://ui-production-333c.up.railway.app
import { chromium } from 'playwright';

const BASE = process.argv[2] ?? 'http://localhost:4174';
const browser = await chromium.launch();

async function signUp(name) {
  const page = await (await browser.newContext()).newPage();
  await page.goto(BASE);
  await page.getByRole('tab', { name: 'Sign up' }).click();
  await page.getByLabel("Username").fill(name);
  await page.getByRole('button', { name: 'Next' }).click();
  const words = (await page.locator('ol.grid li').allInnerTexts()).map((t) => t.split(/\s+/).pop());
  await page.getByRole('button', { name: "I've written them down" }).click();
  for (const label of await page.locator('label').allInnerTexts()) {
    const n = Number(label.match(/Word #(\d+)/)?.[1]);
    if (n) await page.getByLabel(`Word #${n}`, { exact: true }).fill(words[n - 1]);
  }
  await page.getByRole('button', { name: 'Create account' }).click();
  await page.getByRole('link', { name: 'Settings' }).waitFor({ timeout: 120_000 });
  return page;
}

const stamp = Date.now().toString(36).slice(-4);
const watcher = await signUp(`lag_watcher_${stamp}`);
await signUp(`lag_newcomer_${stamp}`);
const t0 = Date.now();
for (;;) {
  await watcher.reload();
  await watcher.getByRole('heading', { name: 'People' }).waitFor({ timeout: 60_000 });
  if (await watcher.getByText(`lag_newcomer_${stamp}`).count()) break;
  if (Date.now() - t0 > 15 * 60_000) {
    console.log('newcomer not visible after 15 min');
    process.exit(1);
  }
  await watcher.waitForTimeout(5000);
}
console.log(`newcomer visible to the watcher after ${((Date.now() - t0) / 1000).toFixed(0)}s`);
await browser.close();
