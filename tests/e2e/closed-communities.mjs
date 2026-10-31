// web's private communities (ADR 0068 part d), at 390px. Founder one creates an
// organizer-run private community: the lock shows. One invites a new member by name; they
// accept under Invites and land in it. One removes them from Settings' "Who decides"; they
// then join again with the invite link. One makes an event for the community's members
// (web's "Private community" audience, ADR 0069); two finds it on the community's page and
// joins it without an invite.
//   node tests/e2e/closed-communities.mjs http://localhost:4174 [screens dir]
// It uses dev founder one (a founder on the live test network too), makes one community and
// one event, and signs up one account.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const handle = `hidden${stamp}`;
const two = { name: `Member${stamp}` };
const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const retry = async (what, attempt, tries = 20) => {
  for (let i = 0; ; i++) {
    if (await attempt().then(Boolean, () => false)) return;
    if (i >= tries) throw new Error(what);
  }
};

const b = await device({ viewport: phone });
await signUp(b, two.name);
log(`${two.name} signed up`);

const a = await device({ viewport: phone });
await phraseSignIn(a, one.phrase.split(' '));
await a.goto(`${BASE}/create/community`);
await a.getByLabel(/Community handle/).fill(handle);
await a.getByLabel('Openness').selectOption('invite_only');
await a.getByLabel('Who decides').selectOption('organizer');
await a.getByLabel(/Description/).fill('Made by the closed-communities test.');
await a.getByRole('button', { name: 'Create community' }).click();
await a.getByRole('heading', { name: handle }).waitFor({ timeout: 120_000 });
await a.getByLabel('Private', { exact: true }).waitFor();
const url = a.url();
log('one made the private community');

// The invite panel: the link carries the seed in its fragment; invite two by name.
await a.getByRole('button', { name: `Invite people to ${handle}` }).click();
const link = await a.getByLabel(`${handle} invite link`).inputValue();
assert.match(link, new RegExp(`/communities/${handle}#invite=`), 'the seed rides in the fragment');
await a.screenshot({ path: `${out}/phone390-community-invite.png` });
const picker = a.getByLabel('Invite by username');
// The directory may take a moment to list someone who just signed up.
await retry(`${two.name} never appeared in the people search`, async () => {
  await picker.fill('');
  await picker.fill(two.name);
  const row = a.getByRole('option', { name: two.name }).first();
  await row.waitFor({ timeout: 5_000 });
  await row.click();
  return true;
}, 30);
await a.getByText(`Invite sent to ${two.name}`).waitFor({ timeout: 60_000 });
log('one invited two');

// Two accepts under Invites and lands in the community.
const card = b.locator('.invites li', { hasText: handle });
await retry('the invite never reached two', async () => {
  await b.goto(`${BASE}/invites`);
  await card.waitFor({ timeout: 15_000 });
  return true;
}, 30);
await card.getByText(/Private community/).waitFor();
await card.getByRole('button', { name: 'Accept' }).click();
await b.getByRole('heading', { name: handle }).waitFor({ timeout: 120_000 });
await b.getByRole('button', { name: `Leave ${handle}` }).waitFor({ timeout: 120_000 });
log('two accepted');

// One's left rail lists it; one removes two from "Who decides" (organizers act at once).
await retry('two never showed as a member', async () => {
  await a.goto(url);
  await a.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  await a.getByRole('button', { name: '2 members', exact: true }).waitFor({ timeout: 10_000 });
  return true;
});
await a.getByRole('button', { name: 'Settings', exact: true }).click();
const settings = a.locator('.scope-settings');
await settings.getByText("You're an organizer, so changes apply straight away.").waitFor();
await settings.getByPlaceholder('Username to remove').fill(two.name);
await settings.getByRole('option', { name: two.name }).first().click();
await a.screenshot({ path: `${out}/phone390-community-who-decides.png` });
await settings.getByRole('button', { name: 'Remove', exact: true }).click();
await a.getByRole('alertdialog').getByRole('button', { name: 'Remove' }).click();
await settings.getByText('Member removed').waitFor({ timeout: 60_000 });
await settings.locator('.sheet-close').click();
log('one removed two');

// Two is out (the page offers the invite panel again), and the link lets them back in.
await retry('two was never removed', async () => {
  await b.goto('about:blank');
  await b.goto(link);
  await b.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  await b.getByText('Join with invite').waitFor({ timeout: 10_000 });
  return true;
});
await b.screenshot({ path: `${out}/phone390-community-join-invite.png` });
await b.getByRole('button', { name: 'Join', exact: true }).click();
await b.getByRole('button', { name: `Leave ${handle}` }).waitFor({ timeout: 120_000 });
log('two joined again with the link');

// One makes a collaborative event for the community (no up-front plan).
const title = `Picnic ${stamp}`;
await a.goto(`${BASE}/create/event`);
const next = () => a.getByRole('button', { name: 'Continue' }).click();
await a.getByLabel('Event title').fill(title);
await a.getByLabel('Description').fill('Made by the closed-communities test.');
await next(); // → Audience
await a.getByText('Private community', { exact: true }).click();
await next(); // → Who
await a.getByLabel('Private community').fill(handle);
await a.getByRole('option', { name: `${handle} (Private)` }).click({ timeout: 60_000 });
await a.screenshot({ path: `${out}/phone390-community-event-who.png` });
await next(); // → Control
await a.getByText(`Private community: ${handle}.`).waitFor();
await a.getByText('Collaborative', { exact: true }).click();
await next(); // → Location (decide later)
await next(); // → Overview
await a.locator('.overview-row', { hasText: 'Private community' }).waitFor();
await a.getByRole('button', { name: 'Publish event' }).click();
await a.getByRole('heading', { name: title }).waitFor({ timeout: 180_000 });
assert.match(a.url(), /\/events\/[^/]+\.private\./);
log('one made an event for the community');

// Two finds it on the community's page and joins it.
const eventCard = b.locator('.event-card', { hasText: title });
await retry('the event never reached the community page', async () => {
  await b.goto('about:blank');
  await b.goto(url);
  await b.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  await eventCard.waitFor({ timeout: 30_000 });
  return true;
});
await b.screenshot({ path: `${out}/phone390-community-event-card.png` });
await eventCard.getByRole('link', { name: title }).first().click();
await b.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await b.getByRole('button', { name: 'Join event' }).click();
await b.getByRole('button', { name: 'Leave event' }).waitFor({ timeout: 120_000 });
log('two joined the community event');

assert.deepEqual(errors, []);
await browser.close();
log('closed-communities: ok');
