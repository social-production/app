// Demo content with real places around Melbourne, so the map and the Region feed have pins to
// look at (owner, 2026-11-17). demo-content.mjs types its places, so they carry no
// coordinates; here every place is picked from the geocoder's suggestions. Proposals, help
// requests, an event and a project, in several suburbs, by the two dev founders (new members
// can only start 2 things a week). Not a test: each part logs and moves on if it fails.
//   node tests/e2e/demo-melbourne.mjs <ui url>
import { log } from './lib.mjs';
import { advanceTo, createEvent, daysFromNow, signInFounders } from './event-flow.mjs';
import * as s from './demo/steps.mjs';
import { placedEventPlan, placedHelpRequest, placedProposal } from './demo/places.mjs';

const failed = [];
// DEMO_SKIP: comma-separated words; parts whose names contain one are skipped (what a first
// run already made).
const skip = (process.env.DEMO_SKIP ?? '').split(',').filter(Boolean);
async function part(name, run) {
  if (skip.some((word) => name.includes(word))) return log(`- ${name} (skipped)`);
  const start = Date.now();
  try {
    await run();
    log(`✓ ${name} (${Math.round((Date.now() - start) / 1000)} s)`);
  } catch (e) {
    failed.push(name);
    log(`✗ ${name}: ${String(e?.message ?? e).split('\n')[0]}`);
  }
}

const { ann, bob } = await signInFounders({ width: 390, height: 844 });

const proposals = [
  ['Fitzroy repair café', 'neighbourhood', 'Fitzroy Town Hall', 'Bring a broken toaster, lamp or jacket; volunteers fix it with you.'],
  ['Brunswick seed library', 'gardening', 'Brunswick Library', 'Swap saved seeds from your garden; take some, leave some.'],
  ['Carlton laneway mural', 'neighbourhood', 'Argyle Square, Carlton', 'Paint the grey wall behind the square with a mural local kids design.'],
  ['Collingwood tool shed', 'neighbourhood', 'Collingwood Childrens Farm', 'A shared shed of ladders, drills and wheelbarrows to borrow.'],
  ['Richmond bike lanes count', 'bikes', 'Richmond Station', 'Count riders on Swan Street for a week to show the council the need.'],
  ['St Kilda beach clean-up crew', 'neighbourhood', 'St Kilda Pier', 'A monthly Sunday morning clean-up of the foreshore, gloves provided.'],
  ['Footscray community kitchen', 'neighbourhood', 'Footscray Market', 'Cook big batches together once a week and share them with anyone.'],
  ['Northcote pollinator strip', 'gardening', 'Ruckers Hill, Northcote', 'Plant native flowers along the nature strip for bees and butterflies.'],
  ['Coburg kids bike library', 'bikes', 'Coburg Lake Reserve', 'Outgrown kids bikes, fixed up and lent to the next family.'],
];

for (const [title, tag, place, text] of proposals) {
  await part(`proposal: ${title}`, () => placedProposal(ann, { title, tag, place, text }));
}

const requests = [
  ['Help moving a piano in Thornbury', 'neighbourhood', 'Thornbury Station', [['Lifter', 'Four strong people for an hour on Saturday.']]],
  ['Lift to the Royal Melbourne Hospital', 'neighbourhood', 'Royal Melbourne Hospital', [['Driver', 'Tuesday 9 am from Parkville, back by noon.']]],
  ['Garden help in Preston', 'gardening', 'Preston Market', [['Weeder', 'Two hours of weeding and a cup of tea.']]],
];
for (const [title, tag, place, roles] of requests) {
  await part(`help request: ${title}`, () =>
    placedHelpRequest(bob, { title, tag, place, roles, text: 'Anyone nearby who can help would be wonderful.' }),
  );
}

await part('event: Edinburgh Gardens picnic', async () => {
  const title = 'Edinburgh Gardens spring picnic';
  const url = await createEvent(ann, bob, title);
  await advanceTo(ann, bob, 'Plan');
  await ann.goto(url);
  await placedEventPlan(ann, {
    title: 'Picnic on the lawn',
    date: daysFromNow(10),
    start: '11:00',
    finish: '15:00',
    place: 'Edinburgh Gardens, Fitzroy North',
  });
  await s.approve(ann, bob, 'Picnic on the lawn');
});

await part('project: Abbotsford compost hub', async () => {
  const title = 'Abbotsford compost hub';
  const url = await placedProposal(ann, {
    title,
    tag: 'gardening',
    place: 'Abbotsford Convent',
    text: 'Three big compost bays for the street’s food scraps, turned every Sunday.',
  });
  await s.joinAndSupport(bob, url, title);
  const now = await s.advancePast(ann, bob, 'Proposal');
  await s.productionPlan(ann, {
    title: 'Build three bays',
    about: 'Three timber bays built in one weekend, then a turning roster.',
    place: 'Abbotsford Convent',
    why: 'Forty households asked for somewhere to compost.',
    step: 'Build the bays',
    detail: 'Recycled pallets from the market, delivered Friday.',
  });
  await s.approve(ann, bob, 'Build three bays');
  await s.advancePast(ann, bob, now);
});

log(failed.length ? `done, ${failed.length} failed: ${failed.join('; ')}` : 'done');
await ann.context().browser()?.close();
process.exit(0);

