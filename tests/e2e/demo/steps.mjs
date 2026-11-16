// Steps for tests/e2e/demo-content.mjs: proposals carried through their votes, plans of each
// kind, channels and a community, written with realistic words for looking at the screens.
import { BASE, TAG_SEARCH, addPlan, pickScopeTag } from '../lib.mjs';
import { advance, until, voteYes } from '../event-flow.mjs';

/** Founder one makes a channel (a taken handle means an earlier run made it). */
export async function channel(page, handle, about) {
  await page.goto(`${BASE}/create/channel`);
  await page.getByLabel(/Channel handle/).fill(handle);
  await page.getByLabel(/Description/).fill(about);
  for (let i = 0; i < 4; i++) {
    await page.getByRole('button', { name: 'Create channel' }).click();
    const moved = page.getByText('head has moved');
    await page
      .getByRole('heading', { name: handle })
      .or(page.getByText('already used'))
      .or(moved)
      .first()
      .waitFor({ timeout: 120_000 });
    if (!(await moved.isVisible())) return;
    await page.waitForTimeout(2000);
  }
}

/** An open community (anyone can join). */
export async function community(page, handle, about) {
  await page.goto(`${BASE}/create/community`);
  await page.getByLabel(/Community handle/).fill(handle);
  await page.getByLabel(/Description/).fill(about);
  await page.getByRole('button', { name: 'Create community' }).click();
  await page
    .getByRole('heading', { name: handle })
    .or(page.getByText('already used'))
    .first()
    .waitFor({ timeout: 120_000 });
}

/** A new proposal in `tag`'s channel; returns its URL. */
export async function propose(page, { title, text, tag, place }) {
  await page.goto(`${BASE}/create/project`);
  await page.getByLabel('Title').fill(title);
  await page.getByLabel('What do you propose?').fill(text);
  await page.getByRole('button', { name: 'Continue' }).click(); // → Tags
  await pickScopeTag(page, TAG_SEARCH, tag);
  await page.getByRole('button', { name: 'Continue' }).click(); // → Location
  if (place) {
    const field = page.getByRole('textbox', { name: 'Place' });
    if (await field.count()) await field.fill(place);
  }
  await page.getByRole('button', { name: 'Continue' }).click(); // → Overview
  await page.getByRole('button', { name: 'Publish proposal' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  const url = page.url().split('?')[0];
  await page.getByRole('button', { name: /^Support/ }).click();
  return url;
}

/** `page` joins the proposal at `url` and supports it. */
export async function joinAndSupport(page, url, title) {
  await page.goto(url);
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  const join = page.getByRole('button', { name: /^Join / });
  if (await join.count()) await join.click();
  await page.waitForTimeout(1500);
  const support = page.getByRole('button', { name: /^Support/ });
  if (await support.count()) await support.click();
  await page.waitForTimeout(1500);
}

const stage = (page) => page.locator('.feed-card-stage').first();

/** Founder one asks to move on, founder two votes yes; waits until the stage is not `from`. */
export async function advancePast(ann, bob, from) {
  await advance(ann);
  await voteYes(bob);
  const moved = () => bob.locator('.feed-card-stage').filter({ hasNotText: new RegExp(`^\\s*${from}\\s*$`) });
  await until(bob, moved, `a stage after ${from}`);
  return (await stage(bob).innerText()).trim();
}

/** Plan wizard steps every kind shares after its own: support, one stage, review, submit. */
async function finishPlan(page, wizard, kind, { why, step, detail }) {
  const next = () => wizard.getByRole('button', { name: 'Next' }).click();
  await wizard.getByPlaceholder('Explain how this plan responds to support').fill(why);
  await next();
  await wizard.getByPlaceholder('Stage title').fill(step);
  await wizard.getByPlaceholder('What happens in this stage?').fill(detail);
  await next(); // → review
  await wizard.getByRole('button', { name: new RegExp(`^Submit ${kind}$`, 'i') }).click();
}

/** A production plan (makes the proposal a project). */
export async function productionPlan(page, plan) {
  await page.reload();
  await page.getByRole('tab', { name: 'Participation' }).click();
  await addPlan(page, 'Production plan');
  const wizard = page.getByRole('dialog', { name: 'Create Production Plan' });
  const next = () => wizard.getByRole('button', { name: 'Next' }).click();
  await wizard.getByPlaceholder('Plan title').fill(plan.title);
  await wizard.getByPlaceholder('Describe the plan').fill(plan.about);
  await next(); // → plan type
  await next(); // → location
  await wizard.getByRole('textbox', { name: 'Place' }).fill(plan.place);
  await next();
  await finishPlan(page, wizard, 'Production Plan', plan);
  await page.locator('.plan-card', { hasText: plan.title }).waitFor({ timeout: 60_000 });
}

/** An operations plan that takes requests (makes the proposal a service). */
export async function operationsPlan(page, plan) {
  await page.reload();
  await page.getByRole('tab', { name: 'Participation' }).click();
  await addPlan(page, 'Operations plan');
  const wizard = page.getByRole('dialog', { name: 'Create Operations Plan' });
  const next = () => wizard.getByRole('button', { name: 'Next' }).click();
  await wizard.getByPlaceholder('Plan title').fill(plan.title);
  await wizard.getByPlaceholder('Describe the plan').fill(plan.about);
  await next(); // → plan type
  await next(); // → location
  await wizard.getByRole('textbox', { name: 'Place' }).fill(plan.place);
  await next(); // → requests
  await wizard.getByLabel('Requests').selectOption({ index: 1 });
  await next();
  await finishPlan(page, wizard, 'Operations Plan', plan);
  await page.locator('.plan-card', { hasText: plan.title }).waitFor({ timeout: 60_000 });
}

/** Rate every criterion "Support" and vote Yes (event-flow's `assess`); both founders approve
 * the plan until it leads. */
export { assess, approvePlan as approve } from '../event-flow.mjs';

/** A message in a project's Chat tab. */
export async function projectChat(page, url, text) {
  await page.goto(url);
  await page.locator('.project-detail h1').waitFor({ timeout: 120_000 });
  await page.getByRole('tab', { name: 'Chat' }).click();
  const box = page.getByRole('textbox', { name: 'Message the project...' });
  await box.fill(text);
  await box.press('Enter');
  await page.locator('.chat-log').getByText(text).first().waitFor({ timeout: 60_000 });
}
