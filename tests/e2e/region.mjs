// M6 phase 5b: web's Region feed scope and "show on map" (ADR 0084, 0085). A new person makes a
// project at a searched place in Fitzroy and sets Fitzroy as their default location: the
// feed's Region scope lists the project (the gateway's /index/region); the map's "Open as
// list" opens Region on the map's place; the project's Venue link opens the map there.
//   node tests/e2e/region.mjs http://localhost:4174 [screenshot dir]
import assert from "node:assert/strict";

import {
  BASE,
  browser,
  device,
  errors,
  feedScope,
  log,
  setDefaultPlace,
  signUp,
  tagTestChannel,
} from "./lib.mjs";

const out = process.argv[3] ?? "screens";
const stamp = Date.now().toString(36).slice(-4);
const title = `Region shed ${stamp}`;

const page = await device({ viewport: { width: 390, height: 844 } });
await signUp(page, `region${stamp}`);
const next = () => page.getByRole("button", { name: "Continue" }).click();

await page.getByRole("button", { name: "Create", exact: true }).click();
await page.getByRole("menuitem", { name: /Proposal/ }).click();
await page.getByLabel("Title").fill(title);
await page.getByLabel("What do you propose?").fill("Tools for the street.");
await next(); // → Tags
await tagTestChannel(page);
await next(); // → Location
await page.getByRole("radio", { name: "In person" }).check();
await page.getByRole("textbox", { name: "Place" }).fill("Fitzroy Melbourne");
await page
  .locator(".location-picker .suggestions button")
  .first()
  .click({ timeout: 30_000 });
await next(); // → Overview
await page.getByRole("button", { name: "Publish proposal" }).click();
await page.getByRole("heading", { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = page.url();
log("created", title);

await page.goto(`${BASE}/settings`);
await setDefaultPlace(page, "Fitzroy Melbourne");

// The feed's Region scope, from the default place; the index lists the project once read.
await page.goto(BASE);
await feedScope(page, "Region");
// Region's place line ("Near Fitzroy … · 25 km") opens the sheet with the place field.
await page
  .locator(".feed-page .region-near", { hasText: "Fitzroy" })
  .waitFor({ timeout: 30_000 });
const card = page.locator(".feed-page .stack .surface", { hasText: title });
for (let i = 0; i < 45 && !(await card.count()); i++) {
  await page.waitForTimeout(8_000);
  await page.reload();
  await page.locator(".feed-page .region-near").waitFor({ timeout: 60_000 });
}
await card.first().waitFor({ timeout: 30_000 });
await page.screenshot({ path: `${out}/region-feed-390.png` });
log("Region lists it, from the default place");

// The map's "Open as list" opens Region on the map's place.
await page.goto(`${BASE}/map`);
const list = page
  .locator(".map-page")
  .getByRole("button", { name: "Open as list" })
  .first();
await page
  .locator(".map-page .map-discovery-card", { hasText: title })
  .waitFor({ timeout: 60_000 });
await list.click();
await page.waitForURL(/scope=region/, { timeout: 30_000 });
assert.match(page.url(), /lat=-37\.\d+&lon=144\.\d+/);
await card.first().waitFor({ timeout: 60_000 });
log("Open as list shows Region on the map’s place");

// The project's Venue link opens the app's map there (not OpenStreetMap).
await page.goto(projectUrl);
await page.getByRole("tab", { name: "Links" }).click();
await page.locator(".links-tab .detail-place").click();
await page.locator(".map-overlay .map-panel").waitFor({ timeout: 30_000 });
await page
  .locator(".map-overlay .map-discovery-card", { hasText: title })
  .waitFor({ timeout: 60_000 });
assert.ok(page.url().startsWith(projectUrl.split("?")[0]), "the page stays");
await page.screenshot({ path: `${out}/region-venue-map-390.png` });
log("the Venue link opens the map on the place");

const tileTrouble = (e) =>
  /cartocdn|maplibre|tiles|Failed to fetch|NetworkError/i.test(e);
assert.deepEqual(
  errors.filter((e) => !tileTrouble(e)),
  [],
);
await browser.close();
log("region ok");
