// Threads, comments, help requests and direct messages for tests/e2e/demo-content.mjs.
import { BASE, TAG_SEARCH, pickScopeTag } from '../lib.mjs';

/** A thread in `tag`'s channel; returns its URL. */
export async function thread(page, { title, text, tag }) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Thread/ }).click();
  await page.getByLabel(/^Thread title/).fill(title);
  await page.getByLabel(/^Opening post/).fill(text);
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel(TAG_SEARCH, { exact: true }).waitFor();
  await pickScopeTag(page, TAG_SEARCH, tag);
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
  await pickScopeTag(page, TAG_SEARCH, tag);
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
  await page.getByRole('link', { name: /^Message / }).click({ timeout: 120_000 });
  await page.waitForURL(/\/messages\?to=/, { timeout: 60_000 });
  await page.getByRole('textbox', { name: 'Write a message...' }).fill(text);
  await page.getByRole('button', { name: 'Send', exact: true }).click();
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

/** A post, with an optional photo (a PNG buffer). */
export async function post(page, { text, photo }) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Post/ }).click();
  await page.getByLabel(/What's on your mind/).fill(text);
  if (photo) {
    await page
      .getByLabel('Photos for this post')
      .setInputFiles({ name: 'garden.png', mimeType: 'image/png', buffer: photo });
    await page.locator('.post-photo-pick img').waitFor({ timeout: 30_000 });
  }
  await page.getByRole('button', { name: 'Publish post' }).click();
  await page.locator('.linked-body.detail').first().waitFor({ timeout: 120_000 });
}

/** A made-up photo of raised garden beds, drawn as SVG and rendered to a PNG by the browser. */
export async function gardenPhoto(page) {
  const beds = [0, 1, 2]
    .map((i) => {
      const x = 60 + i * 370;
      const plants = Array.from({ length: 7 }, (_, j) => {
        const cx = x + 30 + j * 40;
        return `<circle cx="${cx}" cy="${560 - (j % 2) * 14}" r="${22 + (j % 3) * 5}" fill="${['#3f8f3a', '#5aa83f', '#2f7a34'][j % 3]}"/>`;
      }).join('');
      return `<rect x="${x}" y="570" width="320" height="120" rx="10" fill="#8a5a33"/><rect x="${x}" y="570" width="320" height="22" fill="#a06b3e"/>${plants}`;
    })
    .join('');
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="900">
    <defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8ec5f0"/><stop offset="1" stop-color="#dff1fb"/></linearGradient></defs>
    <rect width="1200" height="900" fill="url(#sky)"/><circle cx="1000" cy="150" r="70" fill="#ffd75e"/>
    <path d="M0 480 Q300 400 600 470 T1200 450 V900 H0 Z" fill="#9bcf6b"/><rect y="700" width="1200" height="200" fill="#7ab55c"/>
    ${beds}<rect x="1110" y="420" width="16" height="200" fill="#6b4a2b"/><circle cx="1118" cy="400" r="70" fill="#4e9a45"/></svg>`;
  const shot = await page.context().newPage();
  await shot.setViewportSize({ width: 1200, height: 900 });
  await shot.setContent(`<body style="margin:0">${svg}</body>`);
  const png = await shot.screenshot({ type: 'png' });
  await shot.close();
  return png;
}
