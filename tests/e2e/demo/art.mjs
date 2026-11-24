// A picture and cover for demo projects and events (ADR 0117): an emoji on a tinted tile and a
// soft gradient banner, drawn in a scratch tab so the demo needs no image files.
const draw = async (page, html, width, height) => {
  const tab = await page.context().newPage();
  try {
    await tab.setViewportSize({ width, height });
    await tab.setContent(`<body style="margin:0">${html}</body>`);
    return await tab.locator('div').first().screenshot({ type: 'png' });
  } finally {
    await tab.close();
  }
};

/** Sets the create form's Cover and Picture from `emoji` on colour `hue` (0–360). */
export async function addArt(page, emoji, hue) {
  const tile = `<div style="width:400px;height:400px;display:flex;align-items:center;justify-content:center;
    background:hsl(${hue} 55% 88%);font-size:220px">${emoji}</div>`;
  const band = `<div style="width:1200px;height:400px;position:relative;overflow:hidden;
    background:linear-gradient(120deg,hsl(${hue} 50% 42%),hsl(${(hue + 40) % 360} 55% 62%))">
    <span style="position:absolute;right:80px;top:40px;font-size:300px;opacity:.28">${emoji}</span></div>`;
  const cover = await draw(page, band, 1200, 400);
  const picture = await draw(page, tile, 400, 400);
  await page.getByLabel('Cover image').setInputFiles({ name: 'cover.png', mimeType: 'image/png', buffer: cover });
  await page.getByLabel('Picture image').setInputFiles({ name: 'picture.png', mimeType: 'image/png', buffer: picture });
  await page.waitForTimeout(1500);
}
