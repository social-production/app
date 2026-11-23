import { t } from '#lib/i18n/index.ts';
/**
 * The About page's text. It deliberately differs from web's (owner, 2026-11-10: fewer
 * sections, commons-based peer production, stigmergic signals in place of prices) and
 * describes this app's own rules (ADR 0033, 0071, 0074) rather than web's.
 */
export type AboutTabId = 'why' | 'how' | 'decisions' | 'network';

/** `short` is the phone label (web's `tab-label-compact`). */
export type AboutTab = { id: AboutTabId; label: string; short: string };

export const ABOUT_SECTIONS: AboutTab[] = [
  { id: 'why', label: t('Why'), short: 'Why' },
  { id: 'how', label: t('How it works'), short: 'How' },
  { id: 'decisions', label: t('Decisions'), short: 'Decisions' },
  { id: 'network', label: t('The network'), short: 'Network' },
];

export type Block = { title?: string; paragraphs: string[] };

export type ReadingLink = { label: string; href: string; blurb: string };

export const aboutWhy: Block[] = [
  {
    paragraphs: [
      'Almost everything people need reaches them as a commodity: something made to be sold. Social Production is a tool for making things the other way, because they are useful, and for organising that on purpose and at scale.',
    ],
  },
  {
    title: t('Use-value, exchange-value and value'),
    paragraphs: [
      'Everything people make has a use-value: what it is good for. Bread feeds, a bike carries, a clinic heals. Made as a commodity, it also has an exchange-value: what it trades for on a market.',
      'Behind exchange-value Marx found value: the share of society’s labour time a commodity counts for once it meets others in exchange. Value is not in the bread. It is a social relation between producers who are connected only by buying and selling.',
    ],
  },
  {
    title: t('Production for profit'),
    paragraphs: [
      'Capital is value set in motion to make more value. Owners of the means of production buy labour-power for a wage and sell what it makes for more than they paid; the difference is surplus value, and making it is the point of the whole exercise.',
      'So production follows what sells at a profit, not what is needed. A useful thing nobody can profit from goes unmade, a harmful one that sells gets made, and work is organised for the owner rather than the people doing it or using it.',
    ],
  },
  {
    title: t('A different mode of production'),
    paragraphs: [
      'A mode of production is how a society organises its work: who holds the means of production, who decides, and how the results are shared. Ours rests on private ownership, wage labour and market exchange.',
      'Mutual aid, volunteering and open-source software exist in its gaps, mostly unpaid and squeezed into spare time. They show that other relations are possible; they are not yet how things are made.',
      'Social Production builds those relations directly rather than managing the market more fairly or handing it to new owners, an idea communization theory names: people propose what is needed, decide openly, share the work and share the result. Nothing made here is bought or sold. There are no prices, wages, ads or data sales, and no one owns the platform.',
    ],
  },
  {
    title: t('Commons-based peer production'),
    paragraphs: [
      'Yochai Benkler named the way Wikipedia, Linux and OpenStreetMap get made: commons-based peer production. Many people contribute, each choosing what to work on; the result belongs to everyone; and coordination comes from the shared work itself rather than from managers or prices.',
      'That has worked best for information, which costs almost nothing to copy. Social Production carries the pattern into the physical world of workshops, food, repairs, care, events and services, adding what those need: plans, roles, schedules and places, and signals people leave for each other in place of prices (stigmergy, under How it works).',
    ],
  },
];

export const aboutReading: ReadingLink[] = [
  {
    label: t('Commons-based peer production'),
    href: 'https://en.wikipedia.org/wiki/Commons-based_peer_production',
    blurb: 'Benkler’s account of production by many contributors, outside firms and markets.',
  },
  {
    label: t('Stigmergy'),
    href: 'https://en.wikipedia.org/wiki/Stigmergy',
    blurb: 'Coordination through traces left in a shared environment, from termites to wikis.',
  },
  {
    label: t('Use-value and exchange-value'),
    href: 'https://en.wikipedia.org/wiki/Use_value',
    blurb: 'The distinction this project starts from.',
  },
  {
    label: t('Mode of production'),
    href: 'https://en.wikipedia.org/wiki/Mode_of_production',
    blurb:
      'How a society organises production, ownership, and the relations through which people make and share things.',
  },
  {
    label: t('Value-form theory'),
    href: 'https://en.wikipedia.org/wiki/Value-form',
    blurb:
      'Why value under capitalism is a social form expressed in exchange, not a natural property of useful things.',
  },
  {
    label: t('Communization'),
    href: 'https://en.wikipedia.org/wiki/Communization',
    blurb:
      'Currents that emphasise abolishing exchange and building new relations directly, rather than managing the old ones under new owners.',
  },
];

export const aboutHow: Block[] = [
  {
    title: t('Signals instead of prices'),
    paragraphs: [
      'A market coordinates through prices: a high price tells producers that something is wanted and that they can profit by making it. It is a powerful signal, but it only counts wants backed by money, and it tells you nothing about need.',
      'Social Production coordinates through stigmergy, the way ants and termites build without a plan or a boss: each one leaves a trace in the shared environment, and the traces guide what others do next. Here the traces are open and readable by everyone. You can mark a proposed project with Demand or Oppose, join it, take a role in an activity, or ask for help. Each mark changes what others see: projects with strong demand rise in the feed, a role with open places shows on the calendar and in the right rail, a help request stays visible until its places are filled or its time has passed.',
      'Nobody allocates the work. People see what is wanted, where help is short and what is already under way, and choose for themselves.',
    ],
  },
  {
    title: t('From idea to activity'),
    paragraphs: [
      'Every project starts as a proposal. While it collects signals, members add and rate the values it should keep to. Once at least 66% of the signals are Demand, members can vote to move on to planning: a production plan and a distribution plan for a project, an operations plan and an access plan for a service. Members can write competing plans, assess them against the agreed values and vote; the plan that leads is the one carried out.',
      'In the Activity phase the plan becomes dated activities with roles. People sign up for roles, the activity goes ahead once its roles are filled, and afterwards the people who took part check in and rate how it went.',
      'Alongside projects there are events, help requests (for one-off help with a time and roles), services that people can request, and channels and communities for discussion.',
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
    title: t('Votes'),
    paragraphs: [
      'A decision passes with at least 66% yes among the votes cast, once enough members have voted. How many is enough grows with the group but much more slowly than its size: it is the sample a pollster would need for a confident answer (Cochran’s formula, with a margin of error that tightens as the group grows), and never more than three quarters of the members. Small groups need most of their members; large ones need a representative share.',
      'Votes are final once cast. A vote counts only if the voter was a full member before the proposal was made, and anyone can challenge a count that left eligible members out.',
    ],
  },
  {
    title: t('Standing'),
    paragraphs: [
      'New accounts can join projects, talk and help straight away. To vote and to vouch, an account needs full standing: two full members vouch for it, or it stays active for 30 days. Full members can mark an account they believe is a bot; three marks open a public review, and if 66% of the voting full members agree, the account is paused and its votes stop counting.',
    ],
  },
  {
    title: t('Moderation'),
    paragraphs: [
      'Anyone can report a post, project, comment or message. Full members vote on the report, with a quorum sized from the people involved (its members, commenters and those who signalled on it) and a yes share of at least 66%, higher for older or popular items. Serious harm can be hidden quickly while the vote runs, behind a warning you can choose to look past.',
    ],
  },
];

export const aboutNetwork: Block[] = [
  {
    title: t('Peer to peer'),
    paragraphs: [
      'Social Production runs on Holochain. There is no central database: each person has their own signed record of what they did, and shared data is held by the participants themselves. Your account is a key that only your recovery phrase can restore, and the same phrase works in any copy of the app.',
      'The desktop app is a full node: it holds a copy of the network and keeps it available for others. A small seed node helps new people find the network and lets browsers take part; the aim is a network that keeps running if that node is ever switched off.',
    ],
  },
  {
    title: t('Costs'),
    paragraphs: [
      'For now the seed node is paid for by people who believe in the project, with no return expected except the network itself. As more people run the desktop app, the network carries itself and that cost falls away. There is nothing to buy and nothing for sale.',
    ],
  },
  {
    title: t('Join in'),
    paragraphs: [
      'This is an early test network: expect rough edges and the occasional reset while it grows. Use it, start something useful, and tell us what breaks.',
    ],
  },
];

export const aboutLinks = [
  { label: t('GitHub'), href: 'https://github.com/social-production' },
  { label: t('Discord'), href: 'https://discord.gg/VvbJ3hhEPb' },
  { label: t('Reddit'), href: 'https://www.reddit.com/r/SocialProduction/' },
];
