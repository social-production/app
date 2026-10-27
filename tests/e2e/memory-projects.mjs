// Seed-node memory per project (ADR 0060 follow-up): founder one creates N projects one by one
// and the seed node's resident size is read from /readyz after each, once the archivist has
// joined it. Not part of CI; run against a local release seed node (RELEASE=1).
//   node tests/e2e/memory-projects.mjs http://localhost:4174 [N] [gateway URL]
import { readFileSync } from 'node:fs';

import { BASE, browser, device, log, phraseSignIn } from './lib.mjs';

const count = Number(process.argv[3] ?? 6);
const gateway = process.argv[4] ?? 'http://localhost:8080';
const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const rss = async () => (await (await fetch(`${gateway}/readyz`)).json()).rssMb;
const indexed = async () => (await (await fetch(`${gateway}/index/projects`)).json()).total ?? 0;

const ann = await device();
await phraseSignIn(ann, one.phrase.split(' '));
log(`signed in; rss ${await rss()} MB`);
const stamp = Date.now().toString(36).slice(-4);
for (let i = 0; i < count; i++) {
  const before = await indexed();
  await ann.goto(BASE);
  await ann.getByRole('button', { name: 'Create', exact: true }).click();
  await ann.getByRole('menuitem', { name: /Project/ }).click();
  await ann.getByRole('radio', { name: /Productive project/ }).click();
  await ann.getByRole('button', { name: 'Continue' }).click();
  await ann.getByLabel('Title').fill(`Memory ${stamp} ${i}`);
  await ann.getByLabel('Proposal description').fill('Measuring the seed node.');
  for (let s = 0; s < 3; s++) await ann.getByRole('button', { name: 'Continue' }).click();
  await ann.getByRole('button', { name: 'Publish project' }).click();
  await ann.getByRole('heading', { name: `Memory ${stamp} ${i}` }).waitFor({ timeout: 180_000 });
  // The archivist joins on its next round (each minute).
  for (let t = 0; t < 60 && (await indexed()) <= before; t++) await ann.waitForTimeout(5000);
  await ann.waitForTimeout(20_000);
  log(`project ${i + 1}: rss ${await rss()} MB`);
}
await browser.close();
