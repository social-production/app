// M6 phase 3: web's LocationPicker in the create flow (ADR 0080). A new person creates a
// project at a searched place (the gateway's /locations/search, ADR 0079, backed by Nominatim):
// the suggestion list opens, picking one fills the field and the Overview, and the charter the
// UI sends carries the place (its geocoder id and coordinates). A typed name without a pick
// stays the label (as on web, typing already sets it, so web's "Use “…”" button stays hidden).
//   node tests/e2e/places.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';

import { BASE, browser, device, errors, log, signUp, tagTestChannel } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
const stamp = Date.now().toString(36).slice(-4);
const title = `Tool library ${stamp}`;

const page = await device({ viewport: { width: 390, height: 844 } });
// Zome calls go through the gateway's /ws proxy; their payloads ride as raw msgpack bytes.
const sent = [];
page.on('websocket', (ws) => ws.on('framesent', (f) => sent.push(Buffer.from(f.payload))));
await signUp(page, `places${stamp}`);
const next = () => page.getByRole('button', { name: 'Continue' }).click();

await page.getByRole('button', { name: 'Create', exact: true }).click();
await page.getByRole('menuitem', { name: /Project/ }).click();
await page.getByRole('radio', { name: /Productive project/ }).click();
await next();
await page.getByLabel('Title').fill(title);
await page.getByLabel('Proposal description').fill('Lend drills and ladders.');
await next(); // → Location
await page.getByRole('radio', { name: /Physical/ }).check();

const field = page.getByRole('textbox', { name: 'Place' });
await field.fill('Fitzroy Melbourne');
const suggestion = page.locator('.location-picker .suggestions button').first();
await suggestion.waitFor({ timeout: 30_000 });
await page.screenshot({ path: `${out}/places-suggestions.png` });
const label = (await suggestion.innerText()).trim();
await suggestion.click();
assert.equal(await field.inputValue(), label.slice(0, 160).trim());
log('picked', label);

await next(); // → Scope
await tagTestChannel(page);
await next(); // → Overview
await page.locator('.overview-row', { hasText: 'Location' }).getByText(label.slice(0, 40)).waitFor();
await page.getByRole('button', { name: 'Publish project' }).click();
await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });

const charter = sent.find((b) => b.includes(title) && b.includes('provider_id'));
assert.ok(charter, 'the create call carries a place with its geocoder id');
assert.ok(charter.includes('approximate'), 'the place is approximate, as web defaults');
log('the project was saved with its place');

// A typed-only name has no place, as on web.
await page.goto(BASE);
await page.getByRole('button', { name: 'Create', exact: true }).click();
await page.getByRole('menuitem', { name: /Project/ }).click();
await page.getByRole('radio', { name: /Productive project/ }).click();
await next();
await page.getByLabel('Title').fill(`${title} two`);
await page.getByLabel('Proposal description').fill('A second shed.');
await next();
await page.getByRole('radio', { name: /Physical/ }).check();
await field.fill('Behind the hall');
await next();
await tagTestChannel(page);
await next();
await page.locator('.overview-row', { hasText: 'Behind the hall' }).waitFor();
log('a typed place name is kept as typed');

assert.deepEqual(errors, []);
await browser.close();
log('places ok');
