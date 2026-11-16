// Demo people and content of every kind on a test network, for looking at the screens with
// something real-looking in them (owner, 2026-11-16). Not a test: each part logs and moves on
// if it fails. The two dev founders (Full standing) carry proposals through their votes, so
// one becomes a project, one a service that takes requests and one an event; two stay
// proposals. Three new people join, support, comment, post (one with a photo), ask for help and message each other.
// A new member can start 2 things (proposals, threads, help requests) a week, so the founders
// start the rest. Their recovery phrases are written to `accounts file` so you can log in as them.
//   node tests/e2e/demo-content.mjs <ui url> <accounts file>
import { writeFileSync } from 'node:fs';

import { BASE, browser, device, log, signUp } from './lib.mjs';
import { approvePlan, daysFromNow, filePlan, signInFounders } from './event-flow.mjs';
import * as s from './demo/steps.mjs';
import * as c from './demo/social.mjs';

const accountsFile = process.argv[3] ?? 'demo-accounts.json';
const phone = { width: 390, height: 844 };
const failed = [];
async function part(name, run) {
  const start = Date.now();
  try {
    await run();
    log(`✓ ${name} (${Math.round((Date.now() - start) / 1000)} s)`);
  } catch (e) {
    failed.push(name);
    const where = String(e?.stack ?? '')
      .split('\n')
      .filter((l) => l.includes('/e2e/demo'))
      .map((l) => l.trim().replace(/^at .*\/e2e\//, ''))
      .slice(0, 3)
      .join(' < ');
    log(`✗ ${name}: ${String(e?.message ?? e).split('\n')[0]} [${where}]`);
    if (process.env.DEMO_SHOTS)
      await ann.screenshot({ path: `${process.env.DEMO_SHOTS}/${name.replace(/\W+/g, '-')}.png` }).catch(() => {});
  }
}

const { ann, bob } = await signInFounders(phone);
// The seed allows 50 sign-ups or restores per network a day (5 before 2026-11-16). The
// founders' first log-ins after a fresh seed are restores; three new people join.
const names = ['maya_k', 'tomas_b', 'priya_r'];
const people = {};
const accounts = [];
for (const name of names) {
  await part(`sign up ${name}`, async () => {
    const page = await device({ viewport: phone });
    const words = await signUp(page, name);
    people[name] = page;
    accounts.push({ name, phrase: words.join(' ') });
    writeFileSync(accountsFile, JSON.stringify(accounts, null, 2));
  });
}
const [maya, tomas, priya] = names.map((n) => people[n]);
const [leo, grace] = [tomas, maya];
const crowd = [maya, tomas, priya].filter(Boolean);

await part('channels and a community', async () => {
  await s.channel(ann, 'gardening', 'Growing food and flowers together: plots, seeds, harvests.');
  await s.channel(ann, 'bikes', 'Riding, fixing and sharing bikes around the neighbourhood.');
  await s.channel(ann, 'neighbourhood', 'Everything local: streets, halls, parks and the people in them.');
  await s.community(ann, 'elm-street', 'Neighbours on and around Elm Street. Say hello!');
});

const garden = { title: 'Elm Street community garden' };
await part('a proposal that becomes a project', async () => {
  garden.url = await s.propose(ann, {
    ...garden,
    tag: 'gardening',
    place: 'Vacant lot, 14 Elm Street',
    text: 'Turn the empty lot on Elm Street into shared vegetable beds, with a compost bay and a tap.',
  });
  await s.joinAndSupport(bob, garden.url, garden.title);
  const plan = 'Raised beds and a roster';
  const now = await s.advancePast(ann, bob, 'Proposal');
  await s.productionPlan(ann, {
    title: plan,
    about: 'Twelve raised beds built over two weekends, then a weekly watering roster.',
    place: '14 Elm Street',
    why: 'Thirty neighbours signed up; most want a bed of their own.',
    step: 'Build the beds',
    detail: 'Timber from the hardware co-op, soil delivered Saturday morning.',
  });
  await s.approve(ann, bob, plan);
  await s.advancePast(ann, bob, now);
  for (const p of crowd.slice(0, 3)) await s.joinAndSupport(p, garden.url, garden.title);
  await s.projectChat(ann, garden.url, 'Soil arrives Saturday at 8. Gloves are in the shed.');
  if (maya) await s.projectChat(maya, garden.url, 'I can bring a wheelbarrow!');
});

const bikes = { title: 'Saturday bike repair co-op' };
await part('a proposal that becomes a service with requests', async () => {
  bikes.url = await s.propose(ann, {
    ...bikes,
    tag: 'bikes',
    place: 'Scout hall, Hope Lane',
    text: 'Volunteers fix neighbours’ bikes for free every Saturday, and teach you to do it yourself.',
  });
  await s.joinAndSupport(bob, bikes.url, bikes.title);
  const plan = 'Open workshop on Saturdays';
  const now = await s.advancePast(ann, bob, 'Proposal');
  await s.operationsPlan(ann, {
    title: plan,
    about: 'Doors open 10 to 2 each Saturday; book a slot or just drop in.',
    place: 'Scout hall, Hope Lane',
    why: 'Plenty of people have bikes that only need a small fix.',
    step: 'Set up the stands',
    detail: 'Two repair stands, a parts box and a sign-in sheet.',
  });
  await s.approve(ann, bob, plan);
  await s.advancePast(ann, bob, now);
  for (const p of [tomas, leo].filter(Boolean)) await s.joinAndSupport(p, bikes.url, bikes.title);
  if (!grace) return;
  const ask = grace.getByRole('button', {
    name: 'Request service',
    exact: true,
  });
  for (let i = 0; i < 20; i++) {
    await grace.goto(bikes.url);
    await grace.getByRole('tab', { name: 'Participation' }).click();
    if (
      await ask.waitFor({ timeout: 10_000 }).then(
        () => true,
        () => false,
      )
    )
      break;
  }
  await ask.click();
  const composer = grace.getByRole('dialog');
  await composer.getByPlaceholder('Request title').fill('Squeaky brakes on my commuter');
  await composer.locator('textarea').first().fill('The back brake squeals and barely stops me on hills.');
  await composer.getByRole('button', { name: 'Request', exact: true }).click();
  await grace.getByRole('heading', { name: 'Your requests' }).waitFor({ timeout: 120_000 });
});

const picnic = { title: 'Spring harvest picnic' };
await part('a proposal that becomes an event', async () => {
  picnic.url = await s.propose(ann, {
    ...picnic,
    tag: 'gardening',
    place: 'Elm Street park',
    text: 'Bring a plate and something you grew. Music, seed swap and games for the kids.',
  });
  await s.joinAndSupport(bob, picnic.url, picnic.title);
  const plan = 'Picnic on the oval';
  const now = await s.advancePast(ann, bob, 'Proposal');
  await filePlan(ann, {
    title: plan,
    date: daysFromNow(10),
    start: '11:00',
    finish: '15:00',
    place: 'Elm Street park oval',
  });
  await approvePlan(ann, bob, plan);
  await s.advancePast(ann, bob, now);
  for (const p of [priya, grace].filter(Boolean)) await s.joinAndSupport(p, picnic.url, picnic.title);
});

await part('two proposals still gathering support', async () => {
  const tools = { title: 'Tool library in the old scout hall' };
  tools.url = await s.propose(priya ?? ann, {
    ...tools,
    tag: 'neighbourhood',
    text: 'Borrow a drill, a ladder or a pressure washer instead of buying one. Donations welcome.',
  });
  for (const p of [maya, leo, ann].filter((p) => p && p !== priya)) await s.joinAndSupport(p, tools.url, tools.title);
  const mural = { title: 'Street mural on Hope Lane' };
  mural.url = await s.propose(priya ?? bob, {
    ...mural,
    tag: 'neighbourhood',
    text: 'Local kids and artists paint the long grey wall by the station: a river of native birds.',
  });
  for (const p of [grace, tomas].filter(Boolean)) await s.joinAndSupport(p, mural.url, mural.title);
});

await part('threads with comments and replies', async () => {
  const t1 = await c.thread(maya ?? ann, {
    title: 'Best tomatoes for a shady yard?',
    tag: 'gardening',
    text: 'My yard gets about four hours of sun. Which tomatoes have done well for you?',
  });
  await c.comment(tomas ?? bob, t1, 'Cherry tomatoes cope best with shade. Try Tommy Toe.');
  await c.comment(priya ?? ann, t1, 'Grow them in pots so you can chase the sun around.');
  await c.reply(maya ?? ann, t1, 'Tommy Toe', 'Thanks! Where do you buy seedlings?');
  const t2 = await c.thread(leo ?? bob, {
    title: 'Where can I recycle old bike tyres?',
    tag: 'bikes',
    text: 'I have six worn tyres and tubes. Anyone know a place that takes them?',
  });
  await c.comment(grace ?? ann, t2, 'The tip on Station Road takes them for free on weekends.');
  await c.comment(bob, t2, 'Bring the tubes to the repair co-op: we patch the good ones.');
});

await part('a post and a post with a photo', async () => {
  await c.post(maya ?? bob, {
    text: 'First tomato of the season! Small, but it’s mine. 🍅 Thanks everyone for the tips.',
  });
  await c.post(ann, {
    text: 'The beds are in at Elm Street. Twelve of them, soil and all. Watering roster starts Monday.',
    photo: await c.gardenPhoto(ann),
  });
});

await part('help requests', async () => {
  await c.helpRequest(bob, {
    title: 'Lift to the station on Friday',
    text: 'I have a big suitcase and my train leaves at 7:40 am.',
    place: 'Corner of Elm and Hope Lane',
    tag: 'neighbourhood',
    roles: [['Driver', 'Someone with a car and a big boot.']],
  });
  await c.helpRequest(tomas ?? bob, {
    title: 'Help moving a couch upstairs',
    text: 'A three-seater to a second-floor flat, no lift. Pizza afterwards!',
    place: '22 Hope Lane',
    tag: 'neighbourhood',
    roles: [
      ['Lifter', 'Two strong pairs of hands.'],
      ['Spotter', 'Someone to guide us round the stairwell.'],
    ],
  });
});

await part('direct messages', async () => {
  if (!maya || !tomas) return;
  await c.message(maya, await c.keyOf(tomas), 'Hi Tomás! Are you coming to the garden build on Saturday?');
  await c.answer(tomas, 'maya_k', 'Yes! I’ll bring my drill and some snacks.');
  if (priya) await c.message(priya, await c.keyOf(maya), 'Loved your tomato thread. Want some of my seeds?');
});

writeFileSync(accountsFile, JSON.stringify(accounts, null, 2));
log(failed.length ? `done; failed: ${failed.join(', ')}` : 'done; everything made');
log(`accounts (with their recovery phrases): ${accountsFile}`);
await browser.close();
