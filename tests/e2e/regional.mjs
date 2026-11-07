// M6 phase 3c: web's Regional settings (ADR 0081). A new person searches a default location
// and finds it again after a reload (it is kept on this device); turning on device location
// takes one GPS fix (a fake position in Fitzroy here) and names it by the gateway's reverse
// lookup; "Set from network" either sets the IP hint or explains why it can't.
//   node tests/e2e/regional.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';

import { BASE, browser, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
const stamp = Date.now().toString(36).slice(-4);
const context = await browser.newContext({
  viewport: { width: 390, height: 844 },
  geolocation: { latitude: -37.8, longitude: 144.98 },
  permissions: ['geolocation'],
});
const page = await context.newPage();
page.on('pageerror', (e) => errors.push(String(e)));
await page.goto(BASE);
await signUp(page, `regional${stamp}`);

const section = page.locator('#settings-regional');
const field = section.getByRole('textbox', { name: 'Place' });
await page.goto(`${BASE}/settings`);
await section.getByRole('heading', { name: 'Regional' }).waitFor({ timeout: 60_000 });
await field.fill('Brunswick Melbourne');
const suggestion = section.locator('.suggestions button').first();
await suggestion.waitFor({ timeout: 30_000 });
const label = (await suggestion.innerText()).trim();
await suggestion.click();
await page.reload();
await section.getByRole('heading', { name: 'Regional' }).waitFor({ timeout: 60_000 });
assert.equal(await field.inputValue(), label);
log('the default location stays after a reload:', label);

const gps = section.getByRole('switch', { name: 'Device location (GPS)' });
await gps.click();
await page.waitForFunction(
  () => document.querySelector('#settings-regional input[aria-label="Place"]')?.value.includes('Fitzroy'),
  null,
  { timeout: 30_000 },
);
assert.equal(await gps.getAttribute('aria-checked'), 'true');
await section.screenshot({ path: `${out}/regional-390.png` });
log('device location named the fix:', await field.inputValue());

await section.getByRole('button', { name: 'Set from network' }).click();
const said = section.getByRole('alert');
await Promise.race([
  said.waitFor({ timeout: 30_000 }),
  page.waitForFunction(
    (before) => document.querySelector('#settings-regional input[aria-label="Place"]')?.value !== before,
    await field.inputValue(),
    { timeout: 30_000 },
  ),
]);
log('set from network:', (await said.count()) ? await said.innerText() : await field.inputValue());

assert.deepEqual(errors, []);
await browser.close();
log('regional ok');
