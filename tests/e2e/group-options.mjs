// M4c group options, edits and pins at phone width (ADR 0045). Three new accounts: Ann starts
// a group with Bob, renames it and adds Cat from Chat info, edits and pins a message; Bob
// mutes the chat from its row menu; Ann leaves and Bob takes over managing it.
//   node tests/e2e/group-options.mjs http://localhost:4174 [screens dir]
// It signs up test accounts, so it may run against the live test network too.
import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";

import { BASE, browser, device, errors, log, signUp } from "./lib.mjs";

const out = process.argv[3] ?? "screens";
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const [annName, bobName, catName] = [
  `ann${stamp}`,
  `bob${stamp}`,
  `cat${stamp}`,
];
const group = `Seed swap ${stamp}`;
const renamed = `Seed library ${stamp}`;
const typo = `Meet at the hall at 6 ${stamp}`;
const fixed = `Meet at the barn at 6 ${stamp}`;

const bubble = (page, text) =>
  page.locator(".chat-log").getByText(text).first();
const row = (page, name) =>
  page.locator(".conversation-row", { hasText: name }).first();

/** Messages polls every 8 s; reload until a chat row for `name` shows. */
async function rowShows(page, name) {
  for (let i = 0; i < 20; i++) {
    await page.goto(`${BASE}/messages`);
    for (let j = 0; j < 4; j++) {
      if (await row(page, name).count()) return row(page, name);
      await page.waitForTimeout(3000);
    }
  }
  throw new Error(`timed out waiting for the chat ${name}`);
}

async function account(name) {
  const page = await device({ viewport: phone });
  await signUp(page, name);
  return page;
}

const ann = await account(annName);
const bob = await account(bobName);
const cat = await account(catName);
log("three accounts signed up");

await ann.goto(`${BASE}/messages`);
await ann.getByRole("button", { name: "New message" }).click();
const sheet = ann.getByRole("dialog");
await sheet.getByRole("tab", { name: "Group" }).click();
await sheet.getByPlaceholder("Group name").fill(group);
await sheet.getByPlaceholder("Add members").fill(bobName);
await sheet.getByRole("option", { name: bobName }).click({ timeout: 60_000 });
await sheet.getByPlaceholder("Write a message…").fill(typo);
await sheet.getByRole("button", { name: "Send" }).click();
await ann.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
await bubble(ann, typo).waitFor({ timeout: 60_000 });
log("ann started the group");

// Chat info from the header menu: rename, then add Cat.
await ann.getByRole("button", { name: "Chat options" }).click();
await ann.getByRole("menuitem", { name: "Chat info" }).click();
const info = ann.getByRole("dialog");
await info.getByPlaceholder("Rename group chat").fill(renamed);
await info.getByRole("button", { name: "Save" }).click();
await info.getByText("Group name updated.").waitFor({ timeout: 60_000 });
await info.getByRole("button", { name: "Add member" }).click();
await info.getByPlaceholder("Type a username").fill(catName);
await info.getByRole("button", { name: catName }).click({ timeout: 60_000 });
await info
  .getByText(`${catName} joined the group chat.`)
  .waitFor({ timeout: 60_000 });
await info.getByText(`${annName} (you) · manages the group`).waitFor();
await ann.screenshot({ path: `${out}/phone390-chat-info.png` });
await ann.keyboard.press("Escape");
await ann
  .locator(".identity-title", { hasText: renamed })
  .waitFor({ timeout: 60_000 });
log("ann renamed the group and added cat");

// Edit the typo, then pin the message.
await bubble(ann, typo).click();
await ann.getByRole("menuitem", { name: "Edit" }).click();
const field = ann.getByRole("textbox", { name: "Write a message..." });
assert.equal(await field.inputValue(), typo, "the field holds the message");
await field.fill(fixed);
await ann.getByRole("button", { name: "Save" }).click();
await bubble(ann, fixed).waitFor({ timeout: 60_000 });
await ann
  .locator(".chat-message", { hasText: fixed })
  .locator(".edited-mark")
  .waitFor();
await bubble(ann, fixed).click();
await ann.getByRole("menuitem", { name: "Pin" }).click();
await ann.locator(".pin-bar", { hasText: fixed }).waitFor({ timeout: 60_000 });
await ann.screenshot({ path: `${out}/phone390-pinned.png` });
log("ann edited and pinned her message");

// Bob mutes the group from its row menu, then opens it.
const bobsRow = await rowShows(bob, renamed);
await bobsRow.click({ button: "right" });
await bob.getByRole("menuitem", { name: "Mute" }).click();
await row(bob, renamed).getByLabel("Muted").waitFor({ timeout: 60_000 });
await bob.screenshot({ path: `${out}/phone390-muted-row.png` });
await row(bob, renamed).click();
await bob.locator(".pin-bar", { hasText: fixed }).waitFor({ timeout: 90_000 });
log("bob muted the group and sees the pin");

// Ann leaves; Bob takes over.
await ann.getByRole("button", { name: "Chat options" }).click();
await ann.getByRole("menuitem", { name: "Leave group" }).click();
await ann
  .getByRole("alertdialog")
  .getByRole("button", { name: "Leave" })
  .click();
await ann.waitForURL(`${BASE}/messages`, { timeout: 60_000 });
await ann.getByRole("button", { name: "New message" }).waitFor();
assert.equal(await row(ann, renamed).count(), 0, "the group left her list");
log("ann left the group");

await bob.getByText(`${annName} left the group.`).waitFor({ timeout: 120_000 });
await bob.getByRole("button", { name: "Chat options" }).click();
await bob.getByRole("menuitem", { name: "Chat info" }).click();
await bob
  .getByRole("dialog")
  .getByText(`${bobName} (you) · manages the group`)
  .waitFor({
    timeout: 60_000,
  });
await bob.screenshot({ path: `${out}/phone390-handover.png` });
log("bob manages the group now");

await (await rowShows(cat, renamed)).click();
await bubble(cat, fixed).waitFor({ timeout: 90_000 });
log("cat is in the group");

assert.deepEqual(errors, [], "no page errors");
log("all M4c group options checks passed");
await browser.close();
