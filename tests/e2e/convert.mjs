// Convert (ADR 0106) end to end against a local seed node started by scripts/dev-seed.sh: a
// founder alone moves a proposal to its Plan phase, votes to convert it, and starts the new
// proposal from the closed page; each links the other.
//   node tests/e2e/convert.mjs http://localhost:4174
import assert from "node:assert/strict";
import { mkdirSync, readFileSync } from "node:fs";

import {
  BASE,
  browser,
  continueSteps,
  device,
  errors,
  log,
  phraseSignIn,
} from "./lib.mjs";

const [one] = JSON.parse(
  readFileSync(new URL("./fixtures/dev-founders.json", import.meta.url)),
);
const stamp = Date.now().toString(36).slice(-4);
const title = `Seed swap ${stamp}`;
const shots = process.env.E2E_OUT ?? "screens";
mkdirSync(shots, { recursive: true });

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(" "));
await ann.goto(`${BASE}/create/project`);
await ann.getByLabel("Title").fill(title);
await ann.getByLabel("What do you propose?").fill("Swap seeds every spring.");
await continueSteps(ann, 3);
await ann.getByRole("button", { name: "Publish proposal" }).click();
await ann.getByRole("heading", { name: title }).waitFor({ timeout: 120_000 });
const oldUrl = ann.url();
// A proposal needs support before it can move on.
await ann.getByRole("button", { name: /^Support/ }).click();
await ann.locator(".dock-capsule .dock-chip").waitFor({ timeout: 60_000 });
log("founder one created the proposal and supports it");

/** Reload the Take part tab until `locator` shows (the fold runs after the vote lands). */
async function until(locator, what) {
  for (let i = 0; i < 40; i++) {
    await ann
      .locator(".project-detail h1")
      .waitFor({ state: "attached", timeout: 60_000 });
    await ann.getByRole("tab", { name: "Participation" }).click();
    if (await locator().count()) return;
    await ann.waitForTimeout(3000);
    await ann.reload();
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Pick `move` on the phase sheet, give a reason and start the vote (proposing is a Yes). */
async function move(name, reason, shot) {
  const sheet = ann.getByRole("dialog", { name: "Where should this go next?" });
  // The choice unlocks once the Support signal reaches the project.
  for (let i = 0; ; i++) {
    await ann.getByRole("tab", { name: "Participation" }).click();
    await ann.getByRole("button", { name: "Phase", exact: true }).click();
    if (await sheet.getByRole("radio", { name }).isEnabled({ timeout: 10_000 }))
      break;
    if (i >= 30) throw new Error(`${name} never unlocked`);
    await ann.waitForTimeout(3000);
    await ann.reload();
    await ann
      .locator(".project-detail h1")
      .waitFor({ state: "attached", timeout: 60_000 });
  }
  await sheet.getByRole("radio", { name }).click();
  await sheet
    .getByPlaceholder("State why this phase change should happen now.")
    .fill(reason);
  if (shot) await ann.screenshot({ path: `${shots}/${shot}` });
  await sheet.getByRole("button", { name: "Start the vote" }).click();
  await sheet.waitFor({ state: "hidden", timeout: 60_000 });
}

await move(/^Advance to /, "We are ready.");
await until(
  () => ann.locator(".phase-context h2", { hasText: /^Plan$/ }),
  "the Plan phase",
);
log("a lone founder moved it to its Plan phase");

await move(
  /^Convert to a new proposal/,
  "Better as a library than a swap.",
  "convert-sheet-390.png",
);
const start = () => ann.getByRole("link", { name: /Start the new proposal/ });
await until(start, "the closed page");
assert.equal(await ann.locator(".phase-context h2").innerText(), "Closed");
await ann.screenshot({ path: `${shots}/converted-closed-390.png` });
await ann.emulateMedia({ colorScheme: "dark" });
await ann.waitForTimeout(800); // let the colour transitions finish
await ann.screenshot({ path: `${shots}/converted-closed-390-dark.png` });
await ann.emulateMedia({ colorScheme: "light" });
log("the convert passed and the closed page offers the new proposal");

await start().click();
await ann.waitForURL(/\/create\/project\?continues=/);
// The old project's title, description and tags fill in once it loads.
for (let i = 0; (await ann.getByLabel("Title").inputValue()) !== title; i++) {
  if (i >= 60) throw new Error("the title never carried over");
  await ann.waitForTimeout(500);
}
await ann
  .getByLabel("What do you propose?")
  .fill("A seed library open every Saturday.");
// Its channel came over with the tags, so the scope step is already done.
for (let i = 0; i < 3; i++)
  await ann.getByRole("button", { name: "Continue" }).click();
await ann.getByRole("button", { name: "Publish proposal" }).click();
await ann.waitForURL(
  (url) => url.href !== oldUrl && /\/projects\//.test(url.pathname),
  { timeout: 120_000 },
);
await ann
  .getByRole("link", { name: /Continues the work of/ })
  .waitFor({ timeout: 120_000 });
await ann
  .locator(".context-panel")
  .screenshot({ path: `${shots}/continues-390.png` });
log("the new proposal says whose work it continues");

await ann.goto(oldUrl);
await until(
  () => ann.getByRole("link", { name: /Where the work continues/ }),
  "the link forward",
);
log("the closed project links to the new proposal");

assert.deepEqual(errors, [], "no page errors");
log("all convert checks passed");
await browser.close();
