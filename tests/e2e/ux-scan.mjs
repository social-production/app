// A scan of the controls at 390px, in light and in dark: the feed and its filter sheet, the
// map and its filters, every step of every create flow, and each page's bottom controls
// (post, thread, request, private event, proposal, profile, channel). As dev founder one.
//   node tests/e2e/ux-scan.mjs http://localhost:4174 <out dir>      (ONLY=feed|create|pages)
import { mkdirSync, readFileSync } from "node:fs";

import {
  BASE,
  browser,
  device,
  ensureTestChannel,
  errors,
  feedScope,
  log,
  phraseSignIn,
  tagTestChannel,
  TEST_CHANNEL,
} from "./lib.mjs";

const out = process.env.E2E_OUT ?? process.argv[3] ?? "screens";
mkdirSync(out, { recursive: true });
const stamp = Date.now().toString(36).slice(-4);
const only = process.env.ONLY;
const page = await device({ viewport: { width: 390, height: 844 } });

/** The screen as it is now, in both themes. */
async function shot(name) {
  for (const scheme of ["light", "dark"]) {
    await page.emulateMedia({ colorScheme: scheme });
    await page.waitForTimeout(700);
    await page.screenshot({ path: `${out}/${name}-${scheme}.png` });
  }
  await page.emulateMedia({ colorScheme: "light" });
}

/** Run one part; a part that fails is logged and shot, so it does not lose the rest. */
async function part(name, run) {
  try {
    await run();
    log(`scanned ${name}`);
  } catch (error) {
    log(`MISSED ${name}: ${String(error).split("\n")[0]}`);
    await page
      .screenshot({ path: `${out}/MISSED-${name}.png` })
      .catch(() => {});
    await page.keyboard.press("Escape").catch(() => {});
  }
}

const next = () => page.getByRole("button", { name: "Continue" }).click();
const create = async (item) => {
  await page.goto(BASE);
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await page.getByRole("menuitem", { name: item }).click();
};

// A founder: a new member may start only 2 projects a week, fewer than the scan creates.
const [founder] = JSON.parse(
  readFileSync(new URL("./fixtures/dev-founders.json", import.meta.url)),
);
await phraseSignIn(page, founder.phrase.split(" "));
await ensureTestChannel();

if (!only || only === "feed") {
  await part("feed", async () => {
    await page.goto(BASE);
    await feedScope(page, "Global");
    await page.locator(".surface").first().waitFor({ timeout: 60_000 });
    await shot("feed-global");
    await page.getByRole("button", { name: "Feed filters" }).click();
    await page.getByRole("dialog", { name: /^Show in / }).waitFor();
    await shot("feed-filter-sheet");
    await page.keyboard.press("Escape");
    await feedScope(page, "Region");
    await page.waitForTimeout(1500);
    await shot("feed-region");
    await page.getByRole("button", { name: "Feed filters" }).click();
    await shot("feed-region-filter-sheet");
    await page.keyboard.press("Escape");
  });
  await part("map", async () => {
    await page.goto(BASE);
    await page.getByRole("button", { name: "Open map" }).click();
    await page.locator(".map-panel").waitFor({ timeout: 30_000 });
    await page.waitForTimeout(2500);
    await shot("map");
    await page
      .locator(".map-panel")
      .getByRole("button", { name: /Filters/ })
      .click();
    await page.waitForTimeout(800);
    await shot("map-filter-sheet");
    await page.keyboard.press("Escape");
  });
}

if (!only || only === "create") {
  await part("create-post", async () => {
    await create(/Post/);
    await page
      .getByLabel(/What's on your mind/)
      .fill(`Planting garlic this weekend ${stamp}`);
    await shot("create-post");
    await page.getByRole("button", { name: "Publish post" }).click();
    await page.locator(".linked-body.detail").waitFor({ timeout: 120_000 });
    const composer = page.getByLabel("Add a comment");
    await composer.fill("Cloves pointy end up.");
    await composer.press("Enter");
    await page.locator(".comment-card").first().waitFor({ timeout: 60_000 });
    await shot("page-post");
  });
  await part("create-thread", async () => {
    await create(/Thread/);
    await page.getByLabel(/^Thread title/).fill(`Seed swap ideas ${stamp}`);
    await page
      .getByLabel(/^Opening post/)
      .fill("Share what grew well last year.");
    await shot("create-thread-1");
    await next();
    await tagTestChannel(page);
    await shot("create-thread-2");
    await page.getByRole("button", { name: "Publish thread" }).click();
    await page
      .locator('.comment-composer, [aria-label="Add a comment"]')
      .first()
      .waitFor({ timeout: 120_000 });
    await shot("page-thread");
  });
  await part("create-request", async () => {
    await create(/^Request/);
    await page.getByLabel(/^Title/).fill(`Lift to the station ${stamp}`);
    await page
      .getByLabel("What do you need?")
      .fill("Friday morning, with a suitcase.");
    await shot("create-request-1");
    await next();
    await page.getByLabel("Place").fill("Allotment gate");
    await shot("create-request-2");
    await next();
    await shot("create-request-3");
    await next();
    await tagTestChannel(page);
    await shot("create-request-4");
    await next();
    await page.getByLabel("Role title").fill("Driver");
    await page.getByLabel("Role details").fill("A car with a big boot.");
    await shot("create-request-5");
    await next();
    await shot("create-request-6");
    await page.getByRole("button", { name: "Publish help request" }).click();
    await page.locator(".role-bar").first().waitFor({ timeout: 120_000 });
    await shot("page-request");
  });
  await part("create-event", async () => {
    await create(/Private event/);
    await page.getByLabel("Event title").fill(`Birthday lunch ${stamp}`);
    await page.getByLabel("Description").fill("Lunch, then cake.");
    await shot("create-event-1");
    for (let step = 2; step <= 5; step++) {
      await next();
      await page.waitForTimeout(400);
      await shot(`create-event-${step}`);
    }
    await next(); // → Plan
    const day = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000);
    const iso = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, "0")}-${String(day.getDate()).padStart(2, "0")}`;
    await page.getByLabel("Plan title").fill("Lunch at the hall");
    await page.getByLabel("Plan description").fill("Everyone brings a plate.");
    await page.getByLabel(/^Date/).fill(iso);
    await page.getByLabel("Start time").fill("12:00");
    await page.getByLabel("Finish time").fill("15:00");
    await page.locator(".stage-card").getByLabel("Title").fill("Lunch");
    await page
      .locator(".stage-card")
      .getByLabel("Details")
      .fill("Plates on the long table.");
    await page.screenshot({
      path: `${out}/create-event-6-light.png`,
      fullPage: true,
    });
    await shot("create-event-6");
    await next();
    await shot("create-event-7");
    await page.getByRole("button", { name: "Publish event" }).click();
    await page.locator(".project-detail h1").waitFor({ timeout: 120_000 });
    await shot("page-private-event");
  });
  await part("create-proposal", async () => {
    await create(/Proposal/);
    await page.getByLabel("Title").fill(`Scan shed ${stamp}`);
    await page
      .getByLabel("What do you propose?")
      .fill("A shed to look at the controls of.");
    await shot("create-proposal-1");
    await next();
    await tagTestChannel(page);
    await shot("create-proposal-2");
    await next();
    await shot("create-proposal-3");
    await next();
    await shot("create-proposal-4");
    await page.getByRole("button", { name: "Publish proposal" }).click();
    await page.locator(".project-detail h1").waitFor({ timeout: 120_000 });
    await shot("page-proposal");
    for (const tab of ["Participation", "Chat", "Links", "History"]) {
      await page.getByRole("tab", { name: tab }).click();
      await page.waitForTimeout(2000);
      await shot(`page-proposal-${tab.toLowerCase()}`);
    }
  });
  for (const kind of ["channel", "community"]) {
    await part(`create-${kind}`, async () => {
      await page.goto(`${BASE}/create/${kind}`);
      await page.getByRole("dialog").first().waitFor({ timeout: 45_000 });
      await shot(`create-${kind}`);
    });
  }
}

if (!only || only === "pages") {
  const pages = [
    [
      "profile",
      "/profile",
      () => page.locator(".profile-page, .profile-hero, h1").first(),
    ],
    ["channel", `/channels/${TEST_CHANNEL}`, () => page.locator(".scope-hero")],
    ["platform", "/platform", () => page.getByRole("heading").first()],
    ["messages", "/messages", () => page.getByRole("heading").first()],
    ["people", "/people", () => page.locator(".people-row").first()],
    [
      "feedback",
      "/feedback",
      () => page.getByRole("heading", { name: "Feedback" }),
    ],
    [
      "settings",
      "/settings",
      () => page.locator("#settings-account .set-row").first(),
    ],
    ["search", `/search?q=${TEST_CHANNEL}`, () => page.locator(".type-chips")],
  ];
  for (const [name, path, ready] of pages) {
    await part(`page-${name}`, async () => {
      await page.goto(`${BASE}${path}`);
      await ready().waitFor({ timeout: 45_000 });
      await page.waitForTimeout(1500);
      await shot(`page-${name}`);
    });
  }
}

log(errors.length ? `page errors: ${errors.join(" | ")}` : "no page errors");
await browser.close();
