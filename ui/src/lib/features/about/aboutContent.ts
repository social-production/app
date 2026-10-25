// Ported from web/src/lib/features/about/aboutContent.ts unchanged (web's About copy).
export type AboutTabId =
  'purpose' | 'strategy' | 'roadmap' | 'costs' | 'governance' | 'moderation' | 'join';

export type AboutTab = {
  id: AboutTabId;
  label: string;
};

export const ABOUT_SECTIONS: AboutTab[] = [
  { id: 'purpose', label: 'Purpose' },
  { id: 'strategy', label: 'Strategy' },
  { id: 'roadmap', label: 'Roadmap' },
  { id: 'costs', label: 'Costs' },
  { id: 'governance', label: 'Governance' },
  { id: 'moderation', label: 'Moderation' },
  { id: 'join', label: 'Join' },
];

export const ABOUT_TABS: AboutTab[] = ABOUT_SECTIONS;

export const aboutRoadmap = [
  {
    label: 'Now',
    title: 'Web prototype',
    current: true,
    points: [
      'FastAPI backend and a SvelteKit app people can use today.',
      'Facilitation only: projects, events, help, and shared decisions. No asset holding.',
      'Public use as the prototype becomes stable.',
    ],
  },
  {
    label: 'Next',
    title: 'Legal entity and funding',
    current: false,
    points: [
      'A community-controlled non-profit holds donated funds.',
      'Stewardship stays elected and recallable, with no discretionary slush fund.',
      'Physical means of production can be coordinated once that structure exists.',
    ],
  },
  {
    label: 'Later',
    title: 'Peer-to-peer infrastructure',
    current: false,
    points: [
      'Move hosting toward infrastructure the users run together.',
      'Reduce dependence on a single conventional server.',
    ],
  },
  {
    label: 'Aim',
    title: 'Shutdown-resistant coordination',
    current: false,
    points: [
      'Keep useful activity organised for need rather than sale.',
      'Make the network hard to capture or switch off.',
    ],
  },
];

export type ReadingLink = {
  label: string;
  href: string;
  blurb: string;
};

/** Condensed About explainer copy derived from Why Social Production.md. */
export const aboutWhy = {
  lead: 'Social Production exists to organise useful activity outside markets, wages, and private profit.',
  framing: [
    {
      title: 'Use-value',
      body: 'A thing’s use-value is its concrete usefulness: food when you are hungry, a hammer that drives nails, software that helps people coordinate. It satisfies a want or need directly.',
    },
    {
      title: 'Exchange-value',
      body: 'Exchange-value is not a physical property of the thing. It is a social relation in which value appears only through exchange. Price is a symbol of that real abstraction — value mediated by the market rather than by need.',
    },
  ],
  points: [
    'Under capitalism, production is organised mainly for exchange, not because something is needed.',
    'We already organise outside exchange every day: meals, mutual aid, open-source software, protests, community work.',
    'Corporate social networks enclose that coordination and turn it into ads, data extraction, and attention markets.',
    'Social Production is the reverse enclosure: tools for collective coordination, owned and governed by the people using them.',
    'No ads. No data sales. No production for exchange. The only reason to organise here is because something needs doing.',
  ],
  reading: [
    {
      label: 'Mode of production',
      href: 'https://en.wikipedia.org/wiki/Mode_of_production',
      blurb:
        'How a society organises production, ownership, and the relations through which people make and share things.',
    },
    {
      label: 'Value-form theory',
      href: 'https://en.wikipedia.org/wiki/Value-form',
      blurb:
        'Why value under capitalism is a social form expressed in exchange, not a natural property of useful things.',
    },
    {
      label: 'Communization',
      href: 'https://en.wikipedia.org/wiki/Communization',
      blurb:
        'Currents that emphasise abolishing exchange and building communist relations rather than managing capitalism under new owners.',
    },
  ] as ReadingLink[],
};

export const aboutProduce = {
  lead: 'The strategy is to build new relations of production first — not to wait for political power, and not to seize factories only to reproduce the same market logic under new managers.',
  steps: [
    {
      title: 'Coordinate for use',
      body: 'Start with facilitation: projects, events, help requests, discussion, and shared decision-making oriented to need.',
    },
    {
      title: 'Prove the relations',
      body: 'Show that collective activity can run for use rather than sale before trying to hold large means of production.',
    },
    {
      title: 'Grow the material base',
      body: 'As participation grows, the network can fund and steward tools, land, and infrastructure under community control.',
    },
  ],
  note: 'Historically, “seize the means first” often reproduced exchange under new owners. Social Production takes the opposite path: new non-market relations first, then a growing material base.',
};

export const aboutCosts = {
  lead: 'Servers cost money. That does not mean the network has to become a product, an equity play, or a market.',
  cards: [
    {
      title: 'Phase 1',
      body: 'Hosting is covered by people who believe in the use-value of the network. There is no promised return except the network itself.',
    },
    {
      title: 'Phase 2',
      body: 'A community-controlled non-profit can hold donated funds for specific means of production. Transparent purchases, elected and recallable stewardship, no discretionary slush fund.',
    },
    {
      title: 'Phase 3',
      body: 'Move toward peer-to-peer infrastructure so users collectively host the network and reduce reliance on centralised commercial hosting.',
    },
  ],
};

export const aboutBoard = {
  lead: 'Platform moderators / board members execute community decisions. They do not rule the platform.',
  points: [
    'They stay in place only while they keep 66% approval and meet the platform quorum derived from weekly actives.',
    'Any signed-in user can cast or change a standing yes/no vote on a board member at any time.',
    'If approval or quorum falls below threshold, they can be removed automatically.',
    'Their job is to execute software merges and community-directed purchases, not invent policy on their own.',
  ],
};

export const aboutJoin = {
  lead: 'Social Production is a prototype, an experiment, and a call to action. Use it, shape it, and help build production for use.',
  recap: [
    'Capitalism is not only private property. It is production for exchange.',
    'The strategy is to build non-market relations of production first.',
    'Phase 1 builds the facilitation tools and governance mechanics.',
    'Later phases add community-controlled assets and decentralised infrastructure.',
    'Votes and moderation both use derived quorums from weekly actives, not unanimous turnout.',
  ],
  links: [
    { label: 'GitHub', href: 'https://github.com/social-production' },
    { label: 'Discord', href: 'https://discord.gg/VvbJ3hhEPb' },
    { label: 'Reddit', href: 'https://www.reddit.com/r/SocialProduction/' },
  ],
};
