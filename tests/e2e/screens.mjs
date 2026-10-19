// Screenshots of the main screens at phone and desktop widths, for side-by-side review
// against web/ (the UI is ported, ADR 0004/0032). Creates one account and one project.
//   node tests/e2e/screens.mjs http://localhost:4174 <out dir>
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const sizes = { phone360: { width: 360, height: 780 }, phone390: { width: 390, height: 844 }, desk: { width: 1280, height: 860 } };

const shot = async (page, name) => {
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${out}/${name}.png` });
};

for (const [tag, viewport] of Object.entries(sizes)) {
  const page = await device({ viewport });
  await shot(page, `${tag}-welcome`);
  await signUp(page, `Screens ${tag}`);
  await page.goto(`${BASE}/create/project`);
  await shot(page, `${tag}-create-type`);
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Title').fill(`Tool library (${tag})`);
  await page.getByLabel('Proposal description').fill('Lend and borrow tools across the street.');
  await shot(page, `${tag}-create-basics`);
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Continue' }).click();
  await shot(page, `${tag}-create-overview`);
  await page.getByRole('button', { name: 'Publish project' }).click();
  await page.locator('.project-detail h1').waitFor({ timeout: 120_000 });
  await shot(page, `${tag}-detail`);
  await page.goto(BASE);
  await page.locator('.project-card').first().waitFor({ timeout: 120_000 });
  await shot(page, `${tag}-feed`);
  log(`${tag} done`);
}
log(errors.length ? `page errors: ${errors.join('\n')}` : 'no page errors');
await browser.close();
