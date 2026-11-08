// M6 phase 5a: web's map panel (ADR 0085). A new person creates a project at a searched place
// in Fitzroy, sets Fitzroy as their default location, and opens the map from the top bar: the
// panel lists the project with its distance (from the gateway's /index/map-markers, ADR 0082),
// a saved filter survives a reload, `/map` shows the same panel, and a result opens the
// project and closes the panel. Map tiles come from Carto; when the sandbox can't reach it,
// the panel says so and the list still works.
//   node tests/e2e/map.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';

import { BASE, device, browser, errors, log, signUp, tagTestChannel } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
const stamp = Date.now().toString(36).slice(-4);
const title = `Map shed ${stamp}`;

const page = await device({ viewport: { width: 1280, height: 860 } });
await signUp(page, `map${stamp}`);
const next = () => page.getByRole('button', { name: 'Continue' }).click();

await page.getByRole('button', { name: 'Create', exact: true }).click();
await page.getByRole('menuitem', { name: /Project/ }).click();
await page.getByRole('radio', { name: /Productive project/ }).click();
await next();
await page.getByLabel('Title').fill(title);
await page.getByLabel('Proposal description').fill('Tools near the map.');
await next(); // → Location
await page.getByRole('radio', { name: /Physical/ }).check();
await page.getByRole('textbox', { name: 'Place' }).fill('Fitzroy Melbourne');
const suggestion = page.locator('.location-picker .suggestions button').first();
await suggestion.waitFor({ timeout: 30_000 });
await suggestion.click();
await next(); // → Scope
await tagTestChannel(page);
await next(); // → Overview
await page.getByRole('button', { name: 'Publish project' }).click();
await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
log('created', title);

// Fitzroy as the default location, so the map starts there (web's Regional settings).
await page.goto(`${BASE}/settings`);
const regional = page.locator('#settings-regional');
await regional.getByRole('heading', { name: 'Regional' }).waitFor({ timeout: 60_000 });
await regional.getByRole('textbox', { name: 'Place' }).fill('Fitzroy Melbourne');
await regional.locator('.suggestions button').first().click({ timeout: 30_000 });

// The top bar's map button opens the panel; the index lists the project once it has read it.
const mapButton = page.getByRole('button', { name: 'Open map' });
const card = page.locator('.map-overlay .map-discovery-card', { hasText: title });
let found = false;
for (let i = 0; i < 45 && !found; i++) {
  await mapButton.click();
  await page.locator('.map-overlay .map-panel').waitFor({ timeout: 30_000 });
  found = await card.waitFor({ timeout: 8_000 }).then(() => true, () => false);
  if (!found) await mapButton.click(); // close, and look again after the next index round
}
assert.ok(found, 'the map lists the new project');
assert.match(await card.innerText(), /\d+\.\d km/);
await page.screenshot({ path: `${out}/map-panel.png` });
log('the map lists it:', (await card.innerText()).replace(/\s+/g, ' '));

// A filter is kept on this device (web's map preferences).
await page.getByRole('button', { name: 'Map entity filter' }).first().click();
await page.getByRole('menuitemradio', { name: 'Projects' }).click();
await card.waitFor({ timeout: 30_000 });
await page.goto(`${BASE}/map`);
await page.locator('.map-page .map-panel').waitFor({ timeout: 30_000 });
const prefs = await page.evaluate(() =>
  Object.entries(localStorage).find(([k]) => k.startsWith('sp_map_preferences_'))?.[1],
);
assert.equal(JSON.parse(prefs ?? '{}').entityFilter, 'projects');
await page.locator('.map-page .map-discovery-card', { hasText: title }).waitFor({ timeout: 30_000 });
log('/map shows it, with the saved filter');

// Phone width: the map sits over the list, filters over the map.
await page.setViewportSize({ width: 390, height: 844 });
await page.locator('.map-page .filter-panel-overlay').waitFor({ timeout: 10_000 });
await page.screenshot({ path: `${out}/map-390.png` });

// A result opens its page.
await page.locator('.map-page .map-discovery-card a.title', { hasText: title }).click();
await page.getByRole('heading', { name: title }).waitFor({ timeout: 60_000 });
log('a result opens the project');

const tileTrouble = (e) => /cartocdn|maplibre|tiles|Failed to fetch|NetworkError/i.test(e);
assert.deepEqual(errors.filter((e) => !tileTrouble(e)), []);
await browser.close();
log('map ok');
