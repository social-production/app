// Steps for demo-melbourne.mjs: the same flows as steps.mjs and social.mjs, but each place is
// picked from the geocoder's suggestions, so it has coordinates and shows on the map.
import { BASE, TAG_SEARCH, addPlan, pickScopeTag } from '../lib.mjs';
import { addArt } from './art.mjs';

/** Types `query` into the Place field `field` and picks the first suggestion (falls back to
 * the typed name when the geocoder offers none within 15 s). */
export async function pickPlace(page, field, query) {
  // The create flows start on "Not yet"; "In person" shows the Place field.
  if (!(await field.isVisible().catch(() => false))) {
    const inPerson = page.getByText(/^(In person|Physical)$/).first();
    if (await inPerson.count()) await inPerson.click();
  }
  await field.fill(query);
  const first = page.locator('.location-picker .suggestions button').first();
  try {
    await first.waitFor({ timeout: 15_000 });
    await first.click();
  } catch {
    // No suggestion: the place keeps its typed name (no pin).
  }
}

/** A proposal at a picked place, supported by its author; returns its URL. */
export async function placedProposal(page, { title, text, tag, place, art }) {
  await page.goto(`${BASE}/create/project`);
  if (art) await addArt(page, ...art);
  await page.getByLabel('Title').fill(title);
  await page.getByLabel('What do you propose?').fill(text);
  await page.getByRole('button', { name: 'Continue' }).click(); // → Tags
  await pickScopeTag(page, TAG_SEARCH, tag);
  await page.getByRole('button', { name: 'Continue' }).click(); // → Location
  await pickPlace(page, page.getByRole('textbox', { name: 'Place' }), place);
  await page.getByRole('button', { name: 'Continue' }).click(); // → Overview
  await page.getByRole('button', { name: 'Publish proposal' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  const url = page.url().split('?')[0];
  await page.getByRole('button', { name: /^Support/ }).click();
  return url;
}

/** A help request with roles at a picked place; returns its URL. */
export async function placedHelpRequest(page, { title, text, place, tag, roles }) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /^Request/ }).click();
  const next = () => page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel(/^Title/).fill(title);
  await page.getByLabel('What do you need?').fill(text);
  await next(); // → Location
  await pickPlace(page, page.getByLabel('Place'), place);
  await next(); // → Timing
  await next(); // → Scope
  await pickScopeTag(page, TAG_SEARCH, tag);
  await next(); // → Roles
  for (const [i, [role, details]] of roles.entries()) {
    if (i > 0) await page.getByRole('button', { name: 'Add a role' }).click();
    await page.getByLabel('Role title').nth(i).fill(role);
    await page.getByLabel('Role details').nth(i).fill(details);
  }
  await next(); // → Overview
  await page.getByRole('button', { name: 'Publish help request' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  return page.url().split('?')[0];
}

/** event-flow's `filePlan` (one day) with a picked place. */
export async function placedEventPlan(page, { title, date, start, finish, place }) {
  await page.reload();
  await page.getByRole('tab', { name: 'Participation' }).click();
  await addPlan(page, 'Event plan');
  const wizard = page.getByRole('dialog', { name: /^Create event plan$/i });
  const next = () => wizard.getByRole('button', { name: 'Next' }).click();
  await wizard.getByPlaceholder('Plan title').fill(title);
  await wizard.getByPlaceholder('Describe the plan').fill('Bring a rug and something to share.');
  await next(); // → schedule mode
  await next(); // → date
  await wizard.getByLabel('Date').fill(date);
  await next(); // → times
  await wizard.getByLabel('Start time').fill(start);
  await wizard.getByLabel('Finish time').fill(finish);
  await next(); // → place
  await pickPlace(page, wizard.getByRole('textbox', { name: 'Place' }), place);
  await next();
  await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Everyone is welcome.');
  await next();
  await wizard.getByPlaceholder('Stage title').fill('Lunch');
  await wizard.getByPlaceholder('What happens in this stage?').fill('Share food on the lawn.');
  await next(); // → review
  await wizard.getByRole('button', { name: /^Submit event plan$/i }).click();
  await page.locator('.plan-card', { hasText: title }).waitFor({ timeout: 60_000 });
}
