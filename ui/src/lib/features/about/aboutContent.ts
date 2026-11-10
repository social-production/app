/**
 * The About page's text. It deliberately differs from web's (owner, 2026-11-10: fewer
 * sections, commons-based peer production, stigmergic signals in place of prices) and
 * describes this app's own rules (ADR 0033, 0071, 0074) rather than web's.
 */
export type AboutTabId = 'why' | 'how' | 'decisions' | 'network';

/** `short` is the phone label (web's `tab-label-compact`). */
export type AboutTab = { id: AboutTabId; label: string; short: string };

export const ABOUT_SECTIONS: AboutTab[] = [
  { id: 'why', label: 'Why', short: 'Why' },
  { id: 'how', label: 'How it works', short: 'How' },
  { id: 'decisions', label: 'Decisions', short: 'Decisions' },
  { id: 'network', label: 'The network', short: 'Network' },
];

export type Block = { title?: string; paragraphs: string[] };

export type ReadingLink = { label: string; href: string; blurb: string };

export const aboutWhy: Block[] = [
  {
    paragraphs: [
      'Most of what keeps people alive and well is already made without anyone selling it: meals cooked for a household, a neighbour driving someone to hospital, a mutual-aid kitchen, a Wikipedia article, the open-source software running most of the internet. Social Production is a tool for organising more of life that way, on purpose and at scale.',
    ],
  },
  {
    title: 'Use, not exchange',
    paragraphs: [
      'A thing’s use-value is what it is good for: bread feeds, a bike carries, a clinic heals. Its exchange-value is what it fetches on a market. Under capitalism things get made mainly because they can be sold, so a useful thing nobody can profit from goes unmade, and a harmful one that sells well gets made anyway.',
      'Here the only reason to start anything is that it is useful to someone. There are no prices, wages, ads or data sales, and no one owns the platform.',
    ],
  },
  {
    title: 'Commons-based peer production',
    paragraphs: [
      'Yochai Benkler gave a name to the way Wikipedia, Linux and OpenStreetMap get made: commons-based peer production. Many people contribute, each choosing what to work on; the result belongs to everyone; and coordination comes from the shared work itself rather than from managers or markets.',
      'That has worked best for information, which costs almost nothing to copy. Social Production carries the same pattern into the physical world: workshops, food, repairs, care, events and services. Those need people in the same place at the same time, with tools and materials, so the tool adds what pure information projects never needed: plans, roles, schedules and places.',
    ],
  },
];

export const aboutReading: ReadingLink[] = [
  {
    label: 'Commons-based peer production',
    href: 'https://en.wikipedia.org/wiki/Commons-based_peer_production',
    blurb: 'Benkler’s account of production by many contributors, outside firms and markets.',
  },
  {
    label: 'Stigmergy',
    href: 'https://en.wikipedia.org/wiki/Stigmergy',
    blurb: 'Coordination through traces left in a shared environment, from termites to wikis.',
  },
  {
    label: 'Use-value and exchange-value',
    href: 'https://en.wikipedia.org/wiki/Use_value',
    blurb: 'The distinction this project starts from.',
  },
];

export const aboutHow: Block[] = [
  {
    title: 'Signals instead of prices',
    paragraphs: [
      'A market coordinates through prices: a high price tells producers that something is wanted and that they can profit by making it. It is a powerful signal, but it only counts wants backed by money, and it tells you nothing about need.',
      'Social Production coordinates through stigmergy, the way ants and termites build without a plan or a boss: each one leaves a trace in the shared environment, and the traces guide what others do next. Here the traces are open and readable by everyone. You can mark a proposed project with Demand or Oppose, join it, take a role in an activity, or ask for help. Each mark changes what others see: projects with strong demand rise in the feed, a role with open places shows on the calendar and in the right rail, a help request stays visible until its places are filled or its time has passed.',
      'Nobody allocates the work. People see what is wanted, where help is short and what is already under way, and choose for themselves.',
    ],
  },
  {
    title: 'From idea to activity',
    paragraphs: [
      'Every project starts as a proposal. While it collects signals, members add and rate the values it should keep to. Once at least 66% of the signals are Demand, members can vote to move on to planning: a production plan and a distribution plan for a productive project, an operations plan and an access plan for a collective service. Members can write competing plans, assess them against the agreed values and vote; the plan that leads is the one carried out.',
      'In the Activity phase the plan becomes dated activities with roles. People sign up for roles, the activity goes ahead once its roles are filled, and afterwards the people who took part check in and rate how it went.',
      'Alongside projects there are events, help requests (for one-off help with a time and roles), personal and collective services that people can request, and channels and communities for discussion.',
    ],
  },
];

export const aboutDecisions: Block[] = [
  {
    paragraphs: [
      'There are no owners and no administrators. Every change to a project, its phase, plans, updates, links and settings, is decided by its members, and the rules are enforced by every participant’s own copy of the software rather than by a server.',
    ],
  },
  {
    title: 'Votes',
    paragraphs: [
      'A decision passes with at least 66% yes among the votes cast, once enough members have voted. How many is enough grows with the group but much more slowly than its size: it is the sample a pollster would need for a confident answer (Cochran’s formula, with a margin of error that tightens as the group grows), and never more than three quarters of the members. Small groups need most of their members; large ones need a representative share.',
      'Votes are final once cast. A vote counts only if the voter was a full member before the proposal was made, and anyone can challenge a count that left eligible members out.',
    ],
  },
  {
    title: 'Standing',
    paragraphs: [
      'New accounts can join projects, talk and help straight away. To vote and to vouch, an account needs full standing: two full members vouch for it, or it stays active for 30 days. Full members can mark an account they believe is a bot; three marks open a public review, and if 66% of the voting full members agree, the account is paused and its votes stop counting.',
    ],
  },
  {
    title: 'Moderation',
    paragraphs: [
      'Anyone can report a post, project, comment or message. Full members vote on the report, with a quorum sized from the people involved (its members, commenters and those who signalled on it) and a yes share of at least 66%, higher for older or popular items. Serious harm can be hidden quickly while the vote runs, behind a warning you can choose to look past.',
    ],
  },
];

export const aboutNetwork: Block[] = [
  {
    title: 'Peer to peer',
    paragraphs: [
      'Social Production runs on Holochain. There is no central database: each person has their own signed record of what they did, and shared data is held by the participants themselves. Your account is a key that only your recovery phrase can restore, and the same phrase works in any copy of the app.',
      'The desktop app is a full node: it holds a copy of the network and keeps it available for others. A small seed node helps new people find the network and lets browsers take part; the aim is a network that keeps running if that node is ever switched off.',
    ],
  },
  {
    title: 'Costs',
    paragraphs: [
      'For now the seed node is paid for by people who believe in the project, with no return expected except the network itself. As more people run the desktop app, the network carries itself and that cost falls away. There is nothing to buy and nothing for sale.',
    ],
  },
  {
    title: 'Join in',
    paragraphs: [
      'This is an early test network: expect rough edges and the occasional reset while it grows. Use it, start something useful, and tell us what breaks.',
    ],
  },
];

export const aboutLinks = [
  { label: 'GitHub', href: 'https://github.com/social-production' },
  { label: 'Discord', href: 'https://discord.gg/VvbJ3hhEPb' },
  { label: 'Reddit', href: 'https://www.reddit.com/r/SocialProduction/' },
];
