// Desktop smoke test (SEEDLESS step 1, ADR 0090): drives the real Tauri app through
// tauri-driver (W3C WebDriver on :4444, WebKitWebDriver underneath on Linux). It signs up
// with a new phrase on the device's own node, checks the profile, signs out, and logs back
// in with the words (the chain comes back from the network).
//
//   tauri-driver &   # then
//   node tests/desktop/smoke.mjs target/release/social-production
//
// No npm dependencies: plain fetch against the WebDriver API.
import { resolve } from 'node:path';

const APP = resolve(process.argv[2] ?? 'target/release/social-production');
const DRIVER = process.env.WEBDRIVER_URL ?? 'http://127.0.0.1:4444';
// The gateway the app was built with (VITE_SP_GATEWAYS' first entry).
const SEED = (process.env.VITE_SP_GATEWAYS ?? 'https://seed-node-production.up.railway.app').split(',')[0];
const log = (...a) => console.log(new Date().toISOString().slice(11, 19), ...a);

async function wd(method, path, body) {
  const response = await fetch(`${DRIVER}${path}`, {
    method,
    headers: { 'content-type': 'application/json' },
    ...(body === undefined ? {} : { body: JSON.stringify(body) }),
  });
  const json = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(`${method} ${path}: ${JSON.stringify(json.value ?? json)}`);
  return json.value;
}

const session = await wd('POST', '/session', {
  capabilities: { alwaysMatch: { 'tauri:options': { application: APP } } },
});
const S = `/session/${session.sessionId}`;

/** Run `body` (a function's source taking `args`) in the page; returns its value. */
const js = (body, ...args) => wd('POST', `${S}/execute/sync`, { script: body, args });

/** Wait until `check` (page JS returning a truthy value) holds. */
async function until(what, check, timeout = 60_000, ...args) {
  const end = Date.now() + timeout;
  for (;;) {
    const value = await js(check, ...args).catch(() => undefined);
    if (value) return value;
    if (Date.now() > end) {
      const text = await js('return document.body?.innerText.slice(0, 2000)').catch(() => '?');
      throw new Error(`timed out waiting for ${what}. Page:\n${text}`);
    }
    await new Promise((r) => setTimeout(r, 500));
  }
}

/** Click the first visible, enabled button (or tab/link) whose text is `label`; true once
 * clicked. Clicked in the page: element handles differ between WebDriver servers. */
const CLICK = `const [label, selector] = arguments;
  const b = [...document.querySelectorAll(selector)].find(
    (b) => b.innerText.trim() === label && b.offsetParent !== null && !b.disabled);
  if (b) b.click();
  return !!b;`;

const click = (label, selector = 'button, a, [role=tab]') =>
  until(`the "${label}" button`, CLICK, 60_000, label, selector);

/** Type into the input labelled `label` (a <label> wrapping a span with that text). */
async function fill(label, text) {
  // Element handles differ between WebDriver servers, so set the value in the page and fire
  // `input` (what Svelte's bind:value listens to).
  await until(
    `the "${label}" field`,
    `const [label, text] = arguments;
     const l = [...document.querySelectorAll('label')].find(
       (l) => l.querySelector('.field-label')?.innerText.trim() === label);
     const input = l?.querySelector('input, textarea');
     if (!input) return false;
     input.value = text;
     input.dispatchEvent(new Event('input', { bubbles: true }));
     return true;`,
    60_000,
    label,
    text,
  );
}

const signedIn = (state) =>
  until(`signed-in=${state}`, `return document.querySelector('.shell[data-signed-in="${state}"]')`, 180_000);

async function goto(path) {
  const origin = await js('return location.origin');
  await wd('POST', `${S}/url`, { url: `${origin}${path}` });
}

/** Until the seed node's people index (GET /index/people) lists `name`. */
async function waitForSeed(name, timeout = 480_000) {
  const end = Date.now() + timeout;
  for (;;) {
    const people = await fetch(`${SEED}/index/people?q=${name}`)
      .then((r) => r.json())
      .catch(() => []);
    if (people.some((p) => p.name === name)) return;
    if (Date.now() > end) throw new Error(`the seed node never listed ${name}`);
    await new Promise((r) => setTimeout(r, 5000));
  }
}

try {
  log('app started; waiting for the shell');
  await signedIn('false');

  log('sign up');
  await goto('/onboarding');
  await click('Sign up');
  const name = `desk${Date.now().toString(36).slice(-6)}`;
  await fill('Username', name);
  await click('Next');
  const words = await until(
    'the phrase',
    `const w = [...document.querySelectorAll('ol.grid li')].map((li) => li.innerText.split(/\\s+/).pop());
     return w.length === 24 ? w : null;`,
  );
  await click("I've written them down");
  const asked = await js(`return [...document.querySelectorAll('.field-label')]
    .map((l) => Number(l.innerText.match(/Word #(\\d+)/)?.[1])).filter(Boolean)`);
  for (const n of asked) await fill(`Word #${n}`, words[n - 1]);
  await click('Create account');
  await signedIn('true');
  log(`signed up as ${name} on the device's own node`);

  log('profile shows the name');
  await goto('/settings');
  const key = await until('the agent key', `return document.querySelector('#settings-account .key')?.innerText`);
  await goto('/profile');
  await until('the profile name', `return document.body.innerText.includes(arguments[0])`, 60_000, name);

  // Signing out removes the only copy until another node holds the chain (ADR 0090), so
  // wait until the seed node's index has the profile: this device reached the network.
  log('waiting for the network (the seed node) to hold the profile');
  await waitForSeed(name);

  log('sign out, then log in with the words');
  await goto('/settings');
  await click('Sign out');
  await signedIn('false');
  await goto('/onboarding');
  await click('Log in');
  await fill('Recovery phrase', words.join(' '));
  await click('Log in', 'button[type=submit]');
  await signedIn('true');
  await goto('/settings');
  const again = await until('the agent key after log-in', `return document.querySelector('#settings-account .key')?.innerText`);
  if (again.trim() !== key.trim()) throw new Error(`log-in gave another key: ${again} (was ${key})`);
  await goto('/profile');
  await until('the profile after log-in', `return document.body.innerText.includes(arguments[0])`, 180_000, name);
  log('PASS');
} catch (error) {
  console.error(error);
  process.exitCode = 1;
} finally {
  await wd('DELETE', S).catch(() => undefined);
}
