// M5c events end to end (ADR 0057) against a local seed node started by scripts/dev-seed.sh
// (its dev founders vote). Founder one creates a public event; both move it to Event Plan,
// approve a dated plan, see its time and place on the event, and move it to Activity.
//   node tests/e2e/events.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, continueSteps } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Working bee ${stamp}`;
const shots = process.env.E2E_OUT ?? 'screens';
mkdirSync(shots, { recursive: true });

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(BASE);
await ann.getByRole('button', { name: 'Create', exact: true }).click();
await ann.getByRole('menuitem', { name: /Event/ }).click();
await ann.getByLabel('Event title').fill(title);
await ann.getByLabel('Description').fill('Weed, mulch and share lunch.');
await continueSteps(ann, 4);
await ann.getByText("Set when members approve the event's plan").waitFor();
await ann.getByRole('button', { name: 'Publish event' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.match(ann.url(), /\/events\//);
const eventUrl = ann.url();
await ann.getByRole('button', { name: /^Support/ }).click();
log('founder one created the event and supports it');

const bob = await device();
await phraseSignIn(bob, two.phrase.split(' '));
await bob.goto(eventUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join event' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
log('founder two joined and supports it');

/** Reload until the loaded event page shows `locator` (gossip between the two agents). */
async function until(page, locator, what) {
  for (let i = 0; i < 40; i++) {
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    if (await locator().count()) return;
    await page.waitForTimeout(3000);
    await page.reload();
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Vote Yes on the open phase move in the Take part tab's dock. */
async function voteYes(page) {
  const vote = () => page.getByRole('button', { name: 'Vote', exact: true });
  await page.keyboard.press('Escape');
  for (let i = 0; ; i++) {
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    await page.getByRole('tab', { name: 'Participation' }).click();
    if (await vote().count()) break;
    if (i >= 40) throw new Error('timed out waiting for an open proposal');
    await page.waitForTimeout(3000);
    await page.reload();
  }
  await vote().click();
  await page.locator('.choice-bar .choice.yes').click();
}

/** web's Phase ▾ menu: "Advance to …" with a reason. */
async function advance(page) {
  await page.reload();
  await page.getByRole('tab', { name: 'Participation' }).click();
  await page.getByRole('button', { name: 'Phase', exact: true }).click();
  await page.getByRole('dialog', { name: 'Change phase' }).getByRole('button', { name: /^Advance to / }).click();
  const sheet = page.getByRole('dialog', { name: /^Advance to / });
  await sheet.getByPlaceholder('State why this phase change should happen now.').fill('We are ready.');
  await sheet.getByRole('button', { name: /^Advance to / }).click();
  try {
    await sheet.waitFor({ state: 'hidden', timeout: 60_000 });
  } catch (e) {
    log('advance sheet:', await sheet.innerText());
    throw e;
  }
}

await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'the second member');
await advance(ann);
await voteYes(bob);
await until(bob, () => bob.locator('.feed-card-stage', { hasText: 'Event Plan' }), 'Event Plan');
log('the event moved to Event Plan');

// web's event plan wizard: overview, schedule (single date), times, place, support, stages.
const day = new Date(Date.now() + 14 * 24 * 60 * 60 * 1000);
const iso = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;
const planTitle = `Saturday bee ${stamp}`;
await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: 'Add plan' }).click();
const wizard = ann.getByRole('dialog', { name: /^Create event plan$/i });
const next = () => wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Plan title').fill(planTitle);
await wizard.getByPlaceholder('Describe the plan').fill('Tools out at nine, lunch at noon.');
await next(); // → schedule mode (single date)
await next(); // → date
await wizard.getByLabel('Date').fill(iso);
await next(); // → times
await wizard.getByLabel('Start time').fill('09:00');
await wizard.getByLabel('Finish time').fill('13:00');
await next(); // → place
await wizard.getByLabel('Location').fill('Allotment 4');
await next();
await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Everyone who signalled can come.');
await next();
await wizard.getByPlaceholder('Stage title').fill('Morning');
await wizard.getByPlaceholder('What happens in this stage?').fill('Weed the beds.');
await next(); // → review
await wizard.getByText('09:00 – 13:00').waitFor();
await wizard.getByRole('button', { name: /^Submit event plan$/i }).click();
await ann.locator('.plan-card', { hasText: planTitle }).waitFor({ timeout: 60_000 });
log('founder one filed an event plan');

/** Rate every criterion "Support" and vote Yes; waits for `yes` votes on the card. */
async function assess(page, yes) {
  await page.getByRole('tab', { name: 'Participation' }).click();
  await page.locator('.plan-card', { hasText: planTitle }).click();
  const sheet = page.locator('.plan-sheet');
  await sheet.getByText('Is the timing and schedule realistic?').waitFor();
  for (const row of await sheet.locator('.criterion-row').all()) {
    await row.getByRole('button', { name: /^4(\s*Support)?$/ }).click();
    await row.getByText('Your rating: Support').waitFor({ timeout: 30_000 });
  }
  await sheet.getByRole('button', { name: 'Yes', exact: true }).click();
  await sheet.getByText('Your vote is final once cast.').waitFor({ timeout: 30_000 });
  await page.locator('.plan-card', { hasText: `${yes} yes` }).waitFor({ timeout: 120_000 });
  await page.keyboard.press('Escape');
}
await assess(ann, 1);
await until(bob, () => bob.locator('.plan-card', { hasText: planTitle }), 'the plan');
await assess(bob, 2);
await until(bob, () => bob.locator('.plan-card.leading', { hasText: planTitle }), 'the plan to lead');
log('both approved the event plan; it leads');

// web's EventOverviewHeader: the leading plan's time and place.
await bob.reload();
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 60_000 });
const facts = await bob.locator('.live-fact').allInnerTexts();
assert.ok(facts.some((f) => /9:00|09:00/.test(f) && /1:00|13:00/.test(f)), `time line: ${facts}`);
assert.ok(facts.includes('Allotment 4'), `place: ${facts}`);
await bob.screenshot({ path: `${shots}/event-details-390.png` });
log(`the event shows its plan's time and place: ${facts.join(' | ')}`);

await advance(ann);
await voteYes(bob);
await until(bob, () => bob.locator('.feed-card-stage', { hasText: 'Activity' }), 'Activity');
log('the approved plan let the event move to Activity');

// web's EventCard in the feed, linking to /events/<id>.
const card = bob.locator('.event-card', { hasText: title });
for (let i = 0; i < 20 && !(await card.count()); i++) {
  await bob.goto(BASE);
  await bob.getByRole('button', { name: 'Sort public feed by' }).click();
  await bob.getByRole('menuitemradio', { name: 'Most recent' }).click();
  await bob.locator('.surface').first().waitFor({ timeout: 60_000 });
  if (await card.count()) break;
  await bob.waitForTimeout(3000);
}
await card.waitFor();
assert.match(await card.locator('a.title').getAttribute('href'), /\/events\//);
log('the feed lists it as an event card');

assert.deepEqual(errors, [], 'no page errors');
log('all M5c event checks passed');
await browser.close();
