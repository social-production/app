// Drives the activity wizard (Activity creation mockup v2, owner 2026-11-25): what and when,
// where (and the plan stage), who's needed, then check and create.
const pad = (n) => String(n).padStart(2, '0');
export const hhmm = (d) => `${pad(d.getHours())}:${pad(d.getMinutes())}`;

/** The next five-minute mark at least `minutes` from now. */
export function soon(minutes) {
  const d = new Date(Date.now() + minutes * 60_000);
  d.setMinutes(Math.ceil(d.getMinutes() / 5) * 5, 0, 0);
  return d;
}

/**
 * Fill and create an activity on an open wizard. `title` is typed unless it's prefilled
 * (`undefined`); `start`/`end` are `HH:MM` on the wheels (the day stays the default);
 * `place` picks "Somewhere else" and types it; `stage` taps the first plan stage; `role`
 * names the first role; `maximum` sets its most; `note` fills the optional note.
 */
export async function createActivity(page, o = {}) {
  const next = () => page.getByRole('button', { name: 'Next', exact: true }).click();
  if (o.title !== undefined) await page.getByPlaceholder('Activity title').fill(o.title);
  if (o.start) await page.locator(`[id="tw-Starts-${o.start}"]`).click();
  if (o.end) await page.locator(`[id="tw-Ends-${o.end}"]`).click();
  await next(); // → where
  if (o.place) {
    await page.getByRole('radio', { name: /^(Somewhere else|In person)/ }).click();
    await page.getByPlaceholder('Search a place').fill(o.place);
  }
  if (o.stage) await page.locator('.activity-chips button').first().click();
  await next(); // → who
  if (o.role) await page.getByPlaceholder('For example: Intake desk').first().fill(o.role);
  if (o.maximum) {
    // From "Any", raise gives needed + 1; lower steps down to `maximum`.
    const raise = page.getByRole('button', { name: 'Raise the most' }).first();
    const lower = page.getByRole('button', { name: 'Lower the most' }).first();
    await raise.click();
    const value = page.locator('.role-card').first().locator('.role-step').nth(1).locator('b');
    for (let i = 0; i < 20 && Number(await value.innerText()) < o.maximum; i++) await raise.click();
    for (let i = 0; i < 20 && Number(await value.innerText()) > o.maximum; i++) await lower.click();
  }
  await next(); // → check and create
  if (o.note) await page.getByPlaceholder('What should happen in this activity?').fill(o.note);
  if (o.beforeCreate) await o.beforeCreate();
  await page.getByRole('button', { name: /^Create (activity|session)$/ }).click();
}
