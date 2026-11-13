// Desktop smoke test (SEEDLESS step 1, ADR 0090): drives the real Tauri app through
// tauri-driver (W3C WebDriver on :4444, WebKitWebDriver underneath on Linux). It signs up
// with a new phrase on the device's own node, checks the profile, signs out, and logs back
// in with the words (the chain comes back from the network).
//
//   tauri-driver &   # then
//   node tests/desktop/smoke.mjs target/release/social-production
//
// No npm dependencies: plain fetch against the WebDriver API.
import { existsSync, readFileSync } from 'node:fs';
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
      // An open dialog sits at the end of the page: show it first.
      const text = await js(`const d = document.querySelector('[role=alertdialog], [role=dialog]');
        return (d ? 'DIALOG: ' + d.innerText + '\\n---\\n' : '') + document.body?.innerText.slice(0, 2000)`).catch(() => '?');
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

const DONE = Symbol('done');

/** Whether this build's directory DNA differs from the live seed's (`/readyz`). */
async function resetsDirectory() {
  const file = resolve('happ/directory-dna-hash.txt');
  if (!existsSync(file)) return false;
  const mine = readFileSync(file, 'utf8').trim();
  const live = await fetch(`${SEED}/readyz`)
    .then((r) => r.json())
    .then((j) => j?.cells?.dnas?.directory)
    .catch(() => undefined);
  if (live) log(`directory DNA: this build ${mine}, live ${live}`);
  return !!live && !!mine && live !== mine;
}

/** Whether the seed node's people index (GET /index/people) lists `name` within `timeout`. */
async function waitForSeed(name, timeout) {
  const end = Date.now() + timeout;
  for (;;) {
    const people = await fetch(`${SEED}/index/people?q=${name}`)
      .then((r) => r.json())
      .catch(() => []);
    if (people.some((p) => p.name === name)) return true;
    if (Date.now() > end) return false;
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
  // ADR 0091: the app picked Railway's bootstrap from its list (the first, and it is up).
  const server = await until('the network section', `return document.querySelector('#settings-network .server-url')?.innerText`);
  if (!server.includes('railway.app')) throw new Error(`booted with ${server}, not Railway's bootstrap`);
  log(`network: ${server}`);
  await goto('/profile');
  await until('the profile name', `return document.body.innerText.includes(arguments[0])`, 60_000, name);

  // A PR that changes the directory's integrity is a new network the live seed doesn't run
  // until it is deployed, so nothing can hold this chain yet: the network half of the test
  // runs on main after the deploy (desktop.yml's bundle writes the build's hash).
  if (await resetsDirectory()) {
    log('this build resets the directory network: the live seed runs the old one, so the');
    log('sign-out / log-in-from-the-network half runs on main after the deploy');
    log('PASS (device half)');
    throw DONE;
  }

  // Signing out removes the only copy until another node holds the chain (ADR 0090), so the
  // seed node must have it first: this also proves the desktop shares the live network.
  // (2026-11-11: listed after 59 s. If this fails, run scripts/check-bootstrap.sh: the
  // seed's bootstrap entries were once missing for hours, see STATUS.md.)
  if (!(await waitForSeed(name, 300_000))) throw new Error(`the seed node never listed ${name}`);
  log('the seed node holds the profile');

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
  if (error !== DONE) {
    console.error(error);
    process.exitCode = 1;
  }
} finally {
  await wd('DELETE', S).catch(() => undefined);
}
