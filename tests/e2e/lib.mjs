// Shared browser-test helpers: devices (fresh browser profiles), sign-up and sign-in,
// against a local seed node (see accounts.mjs for how to run).
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

import { chromium } from 'playwright';

export const BASE = process.argv[2] ?? 'http://localhost:4174';
// CHROMIUM_PATH: a preinstalled Chromium (cloud sessions) instead of Playwright's download.
// A fake microphone, allowed without a prompt, for the call tests (calls.mjs).
export const browser = await chromium.launch({
  args: ['--use-fake-ui-for-media-stream', '--use-fake-device-for-media-stream'],
  ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
});
export const log = (...a) => console.log(...a);
/** Uncaught page errors from every device; tests assert it stays empty. */
export const errors = [];

/** A fresh browser profile; `passkeys` adds a device passkey store (virtual authenticator). */
export async function device({ passkeys = false, viewport } = {}) {
  // UX_SCHEME=dark: every device in the dark theme, for reviewing screenshots.
  const context = await browser.newContext({
    ...(viewport ? { viewport } : {}),
    ...(process.env.UX_SCHEME ? { colorScheme: process.env.UX_SCHEME } : {}),
  });
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  if (passkeys) {
    await cdp.send('WebAuthn.addVirtualAuthenticator', {
      options: {
        protocol: 'ctap2', transport: 'internal', hasResidentKey: true,
        hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
      },
    });
  }
  page.on('pageerror', (e) => errors.push(String(e)));
  if (process.env.UX_SHOTS) watch(page);
  await page.goto(BASE);
  return page;
}

// UX_SHOTS=<dir>: a screenshot of every device every four seconds, for reviewing a flow's
// screens afterwards (identical frames are skipped).
let devices = 0;
function watch(page) {
  const id = ++devices;
  let n = 0;
  let last = '';
  const timer = setInterval(async () => {
    const shot = await page.screenshot().catch(() => null);
    if (!shot) return page.isClosed() && clearInterval(timer);
    const key = createHash('sha1').update(shot).digest('hex');
    if (key === last) return;
    last = key;
    mkdirSync(process.env.UX_SHOTS, { recursive: true });
    writeFileSync(`${process.env.UX_SHOTS}/d${id}-${String(++n).padStart(3, '0')}.png`, shot);
  }, 4000);
  timer.unref();
}

// The shell marks itself once the account is connected (any screen width).
export const signedIn = (page) => page.locator('.shell[data-signed-in="true"]').waitFor({ timeout: 120_000 });
export const open = (page, name) => page.getByRole('link', { name, exact: true }).first().click();

/** Signed out, the home page is the public feed; sign-up and log-in live at /onboarding. */
export async function onboarding(page) {
  await page.goto(`${BASE}/onboarding`);
}

export async function signUp(page, name) {
  await onboarding(page);
  await page.getByRole('tab', { name: 'Sign up' }).click();
  await page.getByLabel('Username').fill(name);
  await page.getByRole('button', { name: 'Next' }).click();
  const words = (await page.locator('ol.grid li').allInnerTexts()).map((t) => t.split(/\s+/).pop());
  await page.getByRole('button', { name: "I've written them down" }).click();
  for (const label of await page.locator('label').allInnerTexts()) {
    const n = Number(label.match(/Word #(\d+)/)?.[1]);
    if (n) await page.getByLabel(`Word #${n}`, { exact: true }).fill(words[n - 1]);
  }
  await page.getByRole('button', { name: 'Create account' }).click();
  await signedIn(page);
  // The app then leaves onboarding for the feed (which has the full shell again).
  await page.waitForURL((url) => !url.pathname.startsWith('/onboarding'));
  return words;
}

export async function phraseSignIn(page, words) {
  await onboarding(page);
  await page.getByRole('tab', { name: 'Log in' }).click();
  await page.getByLabel('Recovery phrase').fill(words.join(' '));
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await signedIn(page);
  await page.waitForURL((url) => !url.pathname.startsWith('/onboarding'));
}

export async function signOut(page) {
  await open(page, 'Settings');
  await page.getByRole('button', { name: 'Sign out' }).click();
  await page.getByRole('dialog', { name: 'Sign out' }).getByRole('button', { name: 'Sign out' }).click();
  // Signing out reloads the page; wait until it has, before navigating anywhere.
  await page.locator('.shell[data-signed-in="false"]').waitFor({ timeout: 60_000 });
  await page.waitForLoadState('load');
}

export async function keyOf(page) {
  await open(page, 'Settings');
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}


/** web's right rail (the top bar's activity button). */
export const rail = (page) => page.locator('.right-rail .rail-panel');
export const railTabs = (page) => page.locator('.right-rail [role="tab"]');
export const railTab = (page, name) => railTabs(page).filter({ hasText: name }).click();
export const railButton = (page) => page.getByRole('button', { name: 'Open activity' });

/** Open the rail (it reloads on open) until `locator` shows in it; `wait` per opening (a
 * founder in many projects waits longer for the rail to load). */
export async function railShows(page, locator, what, wait = 10_000) {
  for (let i = 0; i < 20; i++) {
    if (!(await rail(page).count())) await railButton(page).click();
    // Your activity has tabs (Coming up, Votes, Requests): look in each until it shows.
    const until = Date.now() + wait;
    while (Date.now() < until) {
      for (const tab of await railTabs(page).all()) {
        await tab.click();
        if (await locator().isVisible()) return;
      }
      await page.waitForTimeout(500);
    }
    await railButton(page).click(); // close, then reopen to reload it
  }
  throw new Error(`timed out waiting for ${what} in the rail`);
}

/** The channel browser tests tag their projects, events and help requests with (web requires
 * one, ADR 0068). Founder one makes it once per run; a taken handle means it already exists. */
export const TEST_CHANNEL = 'testing';
let testChannel;
export function ensureTestChannel() {
  testChannel ??= (async () => {
    const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
    const page = await device();
    await phraseSignIn(page, one.phrase.split(' '));
    await page.goto(`${BASE}/create/channel`);
    await page.getByLabel(/Channel handle/).fill(TEST_CHANNEL);
    await page.getByLabel(/Description/).fill('Made by the browser tests.');
    // On a fresh node the week's first write can land beside this one ("the source chain
    // head has moved"): press again.
    const moved = page.getByText('head has moved');
    for (let i = 0; i < 4; i++) {
      await page.getByRole('button', { name: 'Create channel' }).click();
      await page
        .getByRole('heading', { name: TEST_CHANNEL })
        .or(page.getByText('already used'))
        .or(moved)
        .first()
        .waitFor({ timeout: 120_000 });
      if (!(await moved.isVisible())) break;
      await page.waitForTimeout(2000);
    }
    await page.context().close();
  })();
  return testChannel;
}

/** On a create flow's scope step: types `name` into the tag field labelled `label` and picks it.
 * The index lists a new channel within a few minutes, so keep searching until it does. */
export async function pickScopeTag(page, label, name) {
  const field = page.getByLabel(label, { exact: true });
  // A channel and a community can share a name; channels are listed first.
  const option = page.getByRole('option', { name, exact: true }).first();
  for (let i = 0; ; i++) {
    await field.fill(name);
    // The search answers asynchronously: wait for it, not just count what is there now.
    if (await option.waitFor({ timeout: 5000 }).then(() => true, () => false)) break;
    if (i >= 60) throw new Error(`${name} never appeared in the "${label}" search`);
    await field.fill('');
    await page.waitForTimeout(2000);
  }
  await option.click();
}

/** The create flows' one search for channels and communities (Polish Pass mockups). */
export const TAG_SEARCH = 'Search channels and communities';

/** On a create flow's scope step: ticks the test channel (the first one ticked is the main tag). */
export async function tagTestChannel(page, label = TAG_SEARCH) {
  await ensureTestChannel();
  await pickScopeTag(page, label, TEST_CHANNEL);
}

/** Clicks Continue `n` times, tagging the test channel first whenever it is on a scope step
 * (a project's or help request's primary channel, or an event's channel tags). */
export async function continueSteps(page, n = 1) {
  for (let i = 0; i < n; i++) {
    if (await page.getByLabel(TAG_SEARCH, { exact: true }).count()) await tagTestChannel(page);
    await page.getByRole('button', { name: 'Continue' }).click();
  }
}

/** Opens the plan wizard from "Add plan". A proposal has no type yet (ADR 0059), so it asks
 * which kind of plan first ('Production plan', 'Operations plan' or 'Event plan'); a project
 * that already has its type goes straight to the wizard. */
export async function addPlan(page, kind = 'Production plan') {
  await page.getByRole('button', { name: 'Add plan' }).click();
  const choice = page.locator('.plan-kind-choice');
  await choice.or(page.getByRole('dialog')).first().waitFor({ timeout: 30_000 });
  if (!(await choice.isVisible())) return;
  await choice.getByRole('radio', { name: new RegExp(kind) }).click();
  await choice.getByRole('button', { name: 'Continue' }).click();
}

/** The feed's scope tabs (ADR 0102): 'Home', 'Global' or 'Region'. */
export const feedScope = (page, name) =>
  page.getByRole('tablist', { name: 'Choose public feed scope' }).getByRole('tab', { name, exact: true }).click();

/** One choice in the feed's filter sheet (ADR 0102): `group` is the menu's old name
 * ('Sort public feed by', 'Choose personal audience', 'Filter public feed', 'Public feed time
 * window'), `option` the choice's label. */
export async function feedChoice(page, group, option) {
  await page.getByRole('button', { name: 'Feed filters' }).click();
  const sheet = page.getByRole('dialog', { name: /^Show in / });
  await sheet.getByRole('radiogroup', { name: group }).getByRole('radio', { name: option, exact: true }).click();
  await sheet.getByRole('button', { name: 'Show results' }).click();
  await sheet.waitFor({ state: 'hidden' });
}

/** Settings → Regional → Default location: search for `query` in the row's sheet, pick the
 * first match and save it with the sheet's "Use …" button. Returns the row's new value. */
export async function setDefaultPlace(page, query) {
  const regional = page.locator('#settings-regional');
  await regional.getByRole('heading', { name: 'Regional' }).waitFor({ timeout: 60_000 });
  const row = regional.getByRole('button', { name: /^Default location/ });
  await row.click();
  const sheet = page.getByRole('dialog', { name: 'Default location' });
  await sheet.getByRole('textbox', { name: 'Search a place' }).fill(query);
  // The first row is "Use my location"; the matches follow it.
  const match = sheet.locator('.place-search-list li:nth-child(2) button');
  await match.waitFor({ timeout: 30_000 });
  await match.click();
  await sheet.locator('.sheet-submit').click();
  await sheet.waitFor({ state: 'hidden' });
  return (await row.locator('.set-row-value').innerText()).trim();
}

/** A profile's Message button opens the chat with that person (empty until the first
 * message); send `body` from the normal message bar and wait for the conversation. */
export async function messageFromProfile(page, name, body, shot = async () => {}) {
  await page.getByRole('link', { name: `Message ${name}` }).click({ timeout: 120_000 });
  await page.waitForURL(/\/messages\?to=/, { timeout: 60_000 });
  await page.locator('.identity-title', { hasText: name }).waitFor({ timeout: 120_000 });
  await page.getByText('No messages here yet.').waitFor({ timeout: 60_000 });
  await shot();
  await page.getByRole('textbox', { name: 'Write a message...' }).fill(body);
  await page.getByRole('button', { name: 'Send', exact: true }).click();
  await page.waitForURL(/\/messages\?conversation=/, { timeout: 180_000 });
}

/** New message → New group as Telegram does it: tick the members, the round arrow, name
 * the group, create it, then send `body` as its first message (if given). Starts from the
 * chat list. `shot` is called on the member list and on the name step. */
export async function startGroup(page, title, members, body, shot = async () => {}) {
  await page.getByRole('button', { name: 'New message' }).click();
  await page.getByRole('button', { name: 'New group' }).click();
  for (const member of members) {
    await page.getByRole('searchbox', { name: 'Search people' }).fill(member);
    await page.getByRole('button', { name: member, pressed: false }).click({ timeout: 120_000 });
  }
  await page.getByText(`${members.length} selected`).waitFor();
  await shot('members');
  await page.getByRole('button', { name: 'Next' }).click();
  await page.getByPlaceholder('Group name').fill(title);
  await shot('name');
  await page.getByRole('button', { name: 'Create group' }).click();
  await page.waitForURL(/\/messages\?conversation=/, { timeout: 300_000 });
  await page.locator('.identity-title', { hasText: title }).waitFor({ timeout: 120_000 });
  if (!body) return;
  await page.getByRole('textbox', { name: 'Write a message...' }).fill(body);
  await page.getByRole('button', { name: 'Send', exact: true }).click();
}
