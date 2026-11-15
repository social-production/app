// Threads, comments, help requests and direct messages for tests/e2e/demo-content.mjs.
import { BASE, pickScopeTag } from '../lib.mjs';

/** A thread in `tag`'s channel; returns its URL. */
export async function thread(page, { title, text, tag }) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Thread/ }).click();
  await page.getByLabel(/^Thread title/).fill(title);
  await page.getByLabel(/^Opening post/).fill(text);
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Primary tag type').waitFor();
  await pickScopeTag(page, 'Primary channel tag', tag);
  await page.getByRole('button', { name: 'Publish thread' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  return page.url().split('?')[0];
}

/** A top-level comment on the thread at `url`, with an up-vote on the thread. */
export async function comment(page, url, text, { upvote = true } = {}) {
  await page.goto(url);
  await page.locator('.overview-shell').waitFor({ timeout: 120_000 });
  if (upvote) {
    const up = page.locator('.overview-shell').getByRole('button', { name: 'Vote up' });
    if (await up.count()) await up.click();
  }
  const box = page.getByLabel('Write a comment...');
  await box.fill(text);
  await box.press('Enter');
  await page.locator('.comment-card', { hasText: text }).first().waitFor({ timeout: 60_000 });
}

/** A reply to the comment holding `to` (it may take a while to arrive). */
export async function reply(page, url, to, text) {
  const card = page.locator('.comment-card', { hasText: to }).first();
  for (let i = 0; i < 20 && !(await card.count()); i++) {
    await page.goto(url);
    await page.waitForTimeout(3000);
  }
  await card.getByRole('button', { name: 'Reply' }).first().click();
  const box = page.getByLabel('Write a reply...');
  await box.fill(text);
  await box.press('Enter');
  await page.locator('.comment-card', { hasText: text }).first().waitFor({ timeout: 60_000 });
}

/** A help request with roles, in `tag`'s channel; returns its URL. */
export async function helpRequest(page, { title, text, place, tag, roles }) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /^Request/ }).click();
  const next = () => page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel(/^Title/).fill(title);
  await page.getByLabel('What do you need?').fill(text);
  await next(); // → Location
  await page.getByLabel('Place').fill(place);
  await next(); // → Timing
  await next(); // → Scope
  await pickScopeTag(page, 'Primary channel tag', tag);
  await next(); // → Roles
  for (const [i, [role, details]] of roles.entries()) {
    if (i > 0) await page.getByRole('button', { name: 'Add role' }).click();
    await page.getByLabel('Role title').nth(i).fill(role);
    await page.getByLabel('Role details').nth(i).fill(details);
  }
  await next(); // → Overview
  await page.getByRole('button', { name: 'Publish help request' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  return page.url().split('?')[0];
}

/** `page` sends a direct message to the person with key `key` from their profile. */
export async function message(page, key, text) {
  await page.goto(`${BASE}/profile/${key}`);
  await page.getByRole('button', { name: /^Message / }).click({ timeout: 120_000 });
  await page.getByPlaceholder('Write a message…').fill(text);
  await page.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
  await page.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
  await page.locator('.chat-log').getByText(text).first().waitFor({ timeout: 60_000 });
}

/** A reply in the open conversation with `name` (waits for it to arrive). */
export async function answer(page, name, text) {
  const row = page.locator('.conversation-row', { hasText: name }).first();
  for (let i = 0; i < 30 && !(await row.count()); i++) {
    await page.goto(`${BASE}/messages`);
    await page.waitForTimeout(4000);
  }
  await row.click();
  const box = page.getByRole('textbox', { name: 'Write a message...' });
  await box.fill(text);
  await box.press('Enter');
  await page.locator('.chat-log').getByText(text).first().waitFor({ timeout: 60_000 });
}

/** The signed-in person's public key, from Settings. */
export async function keyOf(page) {
  await page.goto(`${BASE}/settings`);
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}
