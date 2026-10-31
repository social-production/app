// M5c help requests end to end (ADR 0067). Founder one asks for help with two roles (one
// place for a driver, any number of helpers); a new person finds its feed card.
// The feed card's author link opens the asker's profile, with web's personal help-request card;
// they sign up as driver (which joins the request), see the role full for others, and leave again.
// Then founder one shares it with them from web's Share menu: they get the link in a direct
// chat and web's "A help request was shared with you." notification, which opens it.
//   node tests/e2e/help-requests.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp, tagTestChannel } from './lib.mjs';

const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Lift to the station ${stamp}`;

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(BASE);
await ann.getByRole('button', { name: 'Create', exact: true }).click();
await ann.getByRole('menuitem', { name: /Help request/ }).click();
const next = () => ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel(/^Title/).fill(title);
await ann.getByLabel('What do you need?').fill('Friday morning, with a suitcase.');
await next(); // → Location (physical)
await ann.getByLabel('Place').fill('Allotment gate');
await next(); // → Timing (two hours from now, for two hours)
await next(); // → Scope
await tagTestChannel(ann);
await next(); // → Roles
await ann.getByLabel('Role title').fill('Driver');
await ann.getByLabel('Role details').fill('A car with a big boot.');
await ann.getByRole('button', { name: 'Add role' }).click();
await ann.getByLabel('Role title').nth(1).fill('Company');
await ann.getByLabel('Slots needed').nth(1).fill('0');
await next(); // → Overview
await ann.getByText('2 roles').waitFor();
await ann.getByRole('button', { name: 'Publish help request' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.match(ann.url(), /\/help-requests\//);
const url = ann.url();
await ann.locator('.role-bar', { hasText: 'Driver' }).waitFor({ timeout: 120_000 });
await ann.getByText('Allotment gate').first().waitFor();
log('founder one asked for help');

const helper = await device();
await signUp(helper, `Helper${stamp}`);
let profile = '';
// The feed card: when, where and sign-ups (the index reads new spaces within a minute).
for (let i = 0; ; i++) {
  await helper.goto(BASE);
  await helper.getByRole('button', { name: 'Sort public feed by' }).click();
  await helper.getByRole('menuitemradio', { name: 'Most recent' }).click();
  await helper.locator('.surface').first().waitFor({ timeout: 60_000 }).catch(() => {});
  const card = helper.locator('.help-request-card', { hasText: title });
  if (await card.count()) {
    await card.getByText('0 signed up · 1 needed').waitFor();
    // web's personal feed: the author link opens the asker's profile, which lists the request
    // on web's PersonalHelpRequestCard.
    // (Read the href: the feed re-renders its cards while it refreshes.)
    profile = await card.locator('.content-meta-row a').first().getAttribute('href');
    break;
  }
  if (i >= 40) throw new Error('the help request never reached the feed');
  await helper.waitForTimeout(3000);
}
log('the feed shows its card');
assert.match(profile, /^\/profile\//);
await helper.goto(`${BASE}${profile}`);
const personal = helper.locator('.feed-stack .personal-help-request-card', { hasText: title });
await personal.waitFor({ timeout: 60_000 });
await personal.getByText('0 signed up · 1 needed').waitFor();
await personal.getByText('Allotment gate', { exact: false }).waitFor();
await personal.locator('.identity .header-name').waitFor();
if (process.argv[3]) await helper.screenshot({ path: `${process.argv[3]}/profile-help-request.png` });
log('the asker\'s profile shows it on the personal help-request card');
await helper.goto(url);
const driver = helper.locator('.role-bar', { hasText: 'Driver' });
await driver.waitFor({ timeout: 120_000 });
await driver.getByRole('button', { name: 'Sign up' }).click();
await driver.getByRole('button', { name: 'Leave' }).waitFor({ timeout: 60_000 });
await driver.getByText('1 signed up · 1 needed').waitFor();
log('a helper signed up as driver');

for (let i = 0; ; i++) {
  await ann.goto(url);
  const full = ann.locator('.role-bar', { hasText: 'Driver' }).getByRole('button', { name: 'Full' });
  await ann.locator('.role-bar', { hasText: 'Driver' }).waitFor({ timeout: 120_000 });
  if (await full.count()) break;
  if (i >= 40) throw new Error('the driver role never showed as full');
  await ann.waitForTimeout(3000);
}
log('the asker sees the driver role full');

await driver.getByRole('button', { name: 'Leave' }).click();
await driver.getByRole('button', { name: 'Sign up' }).waitFor({ timeout: 60_000 });
log('the helper left the role');

// web's Share menu: share outside the app (Copy link), then send to the helper by name.
await ann.goto(url);
await ann.getByRole('button', { name: 'Share help request' }).click();
await ann.getByRole('button', { name: 'Copy link' }).waitFor();
const search = ann.locator('#share-people-search');
// The directory may take a moment to list someone who just signed up.
for (let i = 0; ; i++) {
  await search.fill('');
  await search.fill(`Helper${stamp}`);
  const row = ann.locator('.contact-row', { hasText: `Helper${stamp}` }).first();
  if (await row.waitFor({ timeout: 5_000 }).then(() => true, () => false)) {
    await row.click();
    break;
  }
  if (i >= 30) throw new Error('the helper never appeared in the share search');
}
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await search.waitFor({ state: 'detached', timeout: 120_000 });
log('founder one shared it with the helper');

const shared = helper.locator('.notification-card', { hasText: 'A help request was shared with you.' });
for (let i = 0; ; i++) {
  await helper.goto(`${BASE}/notifications`);
  await helper.getByRole('heading', { name: 'Notifications' }).waitFor({ timeout: 60_000 });
  if (await shared.first().waitFor({ timeout: 9_000 }).then(() => true, () => false)) break;
  if (i >= 40) throw new Error('the share never reached the helper');
}
await shared.first().getByText(title).waitFor();
await shared.first().getByText('Help request', { exact: true }).waitFor();
await shared.first().click();
await helper.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.equal(new URL(helper.url()).pathname, new URL(url).pathname);
log('the helper opened it from the notification');

assert.deepEqual(errors, []);
await browser.close();
log('help-requests: ok');
