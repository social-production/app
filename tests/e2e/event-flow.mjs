// Shared steps for browser tests that need an event moved through its phases by the two dev
// founders (scripts/dev-seed.sh): create it, Advance to … with a vote, file and approve an
// event plan. Used by events.mjs and activities.mjs.
import { readFileSync } from 'node:fs';

import { BASE, addPlan, continueSteps, device, log, phraseSignIn } from './lib.mjs';

export const founders = () =>
  JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));

/** Both founders signed in on their own devices (`viewport` for founder one's). */
export async function signInFounders(viewport) {
  const [one, two] = founders();
  const ann = await device(viewport ? { viewport } : {});
  await phraseSignIn(ann, one.phrase.split(' '));
  const bob = await device();
  await phraseSignIn(bob, two.phrase.split(' '));
  return { ann, bob };
}

/** Founder one creates a public event and supports it; founder two joins and supports it.
 * Returns the event's URL. */
export async function createEvent(ann, bob, title) {
  await ann.goto(BASE);
  await ann.getByRole('button', { name: 'Create', exact: true }).click();
  await ann.getByRole('menuitem', { name: /Event/ }).click();
  await ann.getByLabel('Event title').fill(title);
  await ann.getByLabel('Description').fill('Weed, mulch and share lunch.');
  await continueSteps(ann, 4);
  await ann.getByText("Set when members approve the event's plan").waitFor();
  await ann.getByRole('button', { name: 'Publish event' }).click();
  await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  if (!/\/events\//.test(ann.url())) throw new Error(`not an event URL: ${ann.url()}`);
  const eventUrl = ann.url().split('?')[0];
  await ann.getByRole('button', { name: /^Support/ }).click();
  await bob.goto(eventUrl);
  await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  await bob.getByRole('button', { name: 'Join event' }).click();
  await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
  await bob.getByRole('button', { name: /^Support/ }).click();
  await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'the second member');
  return eventUrl;
}

/** Reload until the loaded event page shows `locator` (gossip between the two agents). */
export async function until(page, locator, what) {
  for (let i = 0; i < 40; i++) {
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    if (await locator().count()) return;
    await page.waitForTimeout(3000);
    await page.reload();
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Vote Yes on the open phase move in the Take part tab's dock. */
export async function voteYes(page) {
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
export async function advance(page) {
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

/** Founder one advances, founder two votes yes; waits for the event to show `stage`. */
export async function advanceTo(ann, bob, stage) {
  await advance(ann);
  await voteYes(bob);
  await until(bob, () => bob.locator('.feed-card-stage', { hasText: stage }), stage);
}

const isoDay = (d) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
export const daysFromNow = (n) => isoDay(new Date(Date.now() + n * 24 * 60 * 60 * 1000));

/** web's event plan wizard: overview, schedule (a single `date`, or `range: [from, to]`),
 * times, place, support, one stage, review; submits it. */
export async function filePlan(page, { title, date, range, start, finish, place }) {
  await page.reload();
  await page.getByRole('tab', { name: 'Participation' }).click();
  await addPlan(page, 'Event plan');
  const wizard = page.getByRole('dialog', { name: /^Create event plan$/i });
  const next = () => wizard.getByRole('button', { name: 'Next' }).click();
  await wizard.getByPlaceholder('Plan title').fill(title);
  await wizard.getByPlaceholder('Describe the plan').fill('Tools out at nine, lunch at noon.');
  await next(); // → schedule mode
  if (range) await wizard.getByLabel('Schedule').selectOption('range');
  await next(); // → date(s)
  if (range) {
    await wizard.getByLabel('Start date').fill(range[0]);
    await wizard.getByLabel('End date').fill(range[1]);
  } else {
    await wizard.getByLabel('Date').fill(date);
  }
  await next(); // → times
  await wizard.getByLabel('Start time').fill(start);
  await wizard.getByLabel('Finish time').fill(finish);
  await next(); // → place
  await wizard.getByRole('textbox', { name: 'Place' }).fill(place);
  await next();
  await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Everyone who signalled can come.');
  await next();
  await wizard.getByPlaceholder('Stage title').fill('Morning');
  await wizard.getByPlaceholder('What happens in this stage?').fill('Weed the beds.');
  await next(); // → review
  await wizard.getByText(`${start} – ${finish}`).waitFor();
  await wizard.getByRole('button', { name: /^Submit event plan$/i }).click();
  await page.locator('.plan-card', { hasText: title }).waitFor({ timeout: 60_000 });
}

/** Rate every criterion "Support" and vote Yes; waits for `yes` votes on the card. */
export async function assess(page, planTitle, yes) {
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

/** Both founders approve the plan until it leads. */
export async function approvePlan(ann, bob, planTitle) {
  await assess(ann, planTitle, 1);
  await until(bob, () => bob.locator('.plan-card', { hasText: planTitle }), 'the plan');
  await assess(bob, planTitle, 2);
  await until(bob, () => bob.locator('.plan-card.leading', { hasText: planTitle }), 'the plan to lead');
}
