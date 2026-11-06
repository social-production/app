// M5c events end to end (ADR 0057) against a local seed node started by scripts/dev-seed.sh
// (its dev founders vote). Founder one creates a public event; both move it to Event Plan,
// approve a dated plan, see its time and place on the event, and move it to Activity.
//   node tests/e2e/events.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, errors, log } from './lib.mjs';
import { advanceTo, approvePlan, createEvent, daysFromNow, filePlan, signInFounders } from './event-flow.mjs';

const stamp = Date.now().toString(36).slice(-4);
const title = `Working bee ${stamp}`;
const shots = process.env.E2E_OUT ?? 'screens';
mkdirSync(shots, { recursive: true });

const { ann, bob } = await signInFounders({ width: 390, height: 844 });
const eventUrl = await createEvent(ann, bob, title);
log('founder one created the event; founder two joined; both support it');

await advanceTo(ann, bob, 'Event Plan');
log('the event moved to Event Plan');

const iso = daysFromNow(14);
const day = new Date(`${iso}T12:00`);
const planTitle = `Saturday bee ${stamp}`;
await filePlan(ann, { title: planTitle, date: iso, start: '09:00', finish: '13:00', place: 'Allotment 4' });
log('founder one filed an event plan');
await approvePlan(ann, bob, planTitle);
log('both approved the event plan; it leads');

// web's EventOverviewHeader: the leading plan's time and place.
await bob.reload();
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 60_000 });
const facts = await bob.locator('.live-fact').allInnerTexts();
assert.ok(facts.some((f) => /9:00|09:00/.test(f) && /1:00|13:00/.test(f)), `time line: ${facts}`);
assert.ok(facts.includes('Allotment 4'), `place: ${facts}`);
await bob.screenshot({ path: `${shots}/event-details-390.png` });
log(`the event shows its plan's time and place: ${facts.join(' | ')}`);

await advanceTo(ann, bob, 'Activity');
log('the approved plan let the event move to Activity');

// web's EventCard in the feed, linking to /events/<id>.
const card = bob.locator('.event-card', { hasText: title });
/** The public feed, newest first, until `ready` holds (the index catches up within minutes). */
async function feedUntil(ready, tries) {
  for (let i = 0; i < tries && !(await ready()); i++) {
    await bob.goto(BASE);
    await bob.getByRole('button', { name: 'Sort public feed by' }).click();
    await bob.getByRole('menuitemradio', { name: 'Most recent' }).click();
    await bob.locator('.surface').first().waitFor({ timeout: 60_000 });
    if (await ready()) break;
    await bob.waitForTimeout(5000);
  }
}
await feedUntil(async () => (await card.count()) > 0, 20);
await card.waitFor();
assert.match(await card.locator('a.title').getAttribute('href'), /\/events\//);
log('the feed lists it as an event card');
// web's EventCard place line: the leading plan's start, then the place (the index re-reads
// a written space about once a minute).
const line = card.locator('.location');
await feedUntil(async () => /9:00/.test((await line.allInnerTexts()).join('')), 30);
const placeLine = await line.innerText();
assert.match(placeLine, /9:00.* · Allotment 4$/, `card time and place: ${placeLine}`);
log(`the event card shows its plan's start and place: ${placeLine}`);

// web's EventActivityPhase: the plan's day is marked, and the wizard's times stay inside it.
await bob.goto(eventUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('tab', { name: 'Participation' }).click();
const planned = bob.locator('.calendar-cell.planned-day');
await planned.first().waitFor({ timeout: 60_000 });
assert.equal(await planned.count(), 1, 'one plan day');
assert.equal(await planned.locator('.calendar-day-number').innerText(), String(day.getDate()));
await bob.getByText('Click a marked plan day to add the first activity.').waitFor();
await bob.getByRole('button', { name: 'Add activity' }).click();
await bob.getByPlaceholder('Activity title').fill('Weeding');
await bob.getByRole('button', { name: 'Next', exact: true }).click();
const startField = bob.getByLabel('Start time');
assert.equal(await startField.getAttribute('min'), `${iso}T09:00`);
assert.equal(await startField.getAttribute('max'), `${iso}T13:00`);
assert.equal(await startField.inputValue(), `${iso}T09:00`, 'opens inside the plan day');
await bob.screenshot({ path: `${shots}/event-activity-wizard-390.png` });
log('the plan day is marked, and new activities stay inside it');

assert.deepEqual(errors, [], 'no page errors');
log('all M5c event checks passed');
await browser.close();
