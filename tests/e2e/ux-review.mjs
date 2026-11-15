// Screenshots of the UX pass's screens at 390px, in light and in dark, for looking over
// against the approved mockups (STATUS.md "Approved designs"). One new account; it makes one
// proposal so the detail tabs have something to show.
//   node tests/e2e/ux-review.mjs http://localhost:4174 <out dir>
import { mkdirSync } from "node:fs";

import {
  BASE,
  browser,
  continueSteps,
  device,
  ensureTestChannel,
  errors,
  log,
  signUp,
  TEST_CHANNEL,
} from "./lib.mjs";

const out = process.argv[3] ?? "screens";
mkdirSync(out, { recursive: true });
const stamp = Date.now().toString(36).slice(-4);

const page = await device({ viewport: { width: 390, height: 844 } });

/** The screen as it is now, in both themes (the app follows the device's scheme). */
async function shot(name, { full = true } = {}) {
  for (const scheme of ["light", "dark"]) {
    await page.emulateMedia({ colorScheme: scheme });
    await page.waitForTimeout(700);
    await page.screenshot({
      path: `${out}/${name}-${scheme}.png`,
      fullPage: full,
    });
  }
  await page.emulateMedia({ colorScheme: "light" });
}

/** Go to a path, wait for something on it, and shoot it. A screen that fails is logged, so
 * one slow page does not lose the rest. */
async function visit(name, path, ready, options) {
  try {
    await page.goto(`${BASE}${path}`);
    await ready().waitFor({ timeout: 90_000 });
    await shot(name, options);
    log(`shot ${name}`);
  } catch (error) {
    log(`MISSED ${name}: ${String(error).split("\n")[0]}`);
  }
}

await signUp(page, `review${stamp}`);
await ensureTestChannel();

await visit("settings", "/settings", () =>
  page.locator("#settings-account .set-row").first(),
);
await visit("people", "/people", () => page.locator(".people-row").first());
await visit("search", `/search?q=${TEST_CHANNEL}`, () =>
  page.locator(".type-chips"),
);
await visit("search-empty", "/search", () => page.locator(".type-chips"));
await visit("channel", `/channels/${TEST_CHANNEL}`, () =>
  page.locator(".scope-hero"),
);
await visit("feedback", "/feedback", () =>
  page.getByRole("heading", { name: "Feedback" }),
);
await visit("new-message", "/messages?new=message", () =>
  page.getByRole("heading", { name: "New message" }),
);
await visit("new-group", "/messages?new=group", () =>
  page.getByRole("heading", { name: "New group" }),
);
for (const kind of [
  "post",
  "thread",
  "help-request",
  "event",
  "channel",
  "community",
]) {
  await visit(
    `create-${kind}`,
    `/create/${kind}`,
    () => page.locator("main form, main .page").first(),
    {
      full: false,
    },
  );
}

// A proposal, for its Links and History tabs and the create menu.
try {
  await page.goto(`${BASE}/create/project`);
  await page.getByLabel("Title").fill(`Review shed ${stamp}`);
  await page
    .getByLabel("What do you propose?")
    .fill("A shed to look at the tabs of.");
  await shot("create-proposal", { full: false });
  await continueSteps(page, 3);
  await page.getByRole("button", { name: "Publish proposal" }).click();
  await page.locator(".project-detail h1").waitFor({ timeout: 120_000 });
  await shot("detail");
  for (const tab of ["Take part", "Links", "History"]) {
    await page.getByRole("tab", { name: tab }).click();
    await page.waitForTimeout(2500);
    await shot(`detail-${tab.toLowerCase().replace(" ", "-")}`);
  }
  await page.goto(BASE);
  await page.getByRole("button", { name: "Create" }).first().click();
  await shot("create-menu", { full: false });
} catch (error) {
  log(`MISSED the proposal screens: ${String(error).split("\n")[0]}`);
}

log(errors.length ? `page errors: ${errors.join("\n")}` : "no page errors");
await browser.close();
