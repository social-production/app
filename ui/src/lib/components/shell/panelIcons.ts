/** SVG bodies of the top bar's panel toggles and small glyphs (verbatim from web's AppShell). */
export const panelIcons = {
  map: '<path d="M5 6.5 9 4.5l5 2.5 5-2.5v13l-5 2.5-5-2.5-4 2V6.5Z" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" /><path d="M9 4.5v13M14 7v13" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" fill="none" />',
  activity:
    '<rect x="4" y="5" width="16" height="15" rx="2" fill="none" stroke="currentColor" stroke-width="1.8" /><path d="M4 9h16M8 3v4M16 3v4" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" fill="none" /><circle cx="17.5" cy="17.5" r="4.5" fill="var(--panel)" stroke="currentColor" stroke-width="1.6" /><path d="M15.8 17.5 16.9 18.6 19.3 16.2" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none" />',
  search:
    '<path d="M10.5 18a7.5 7.5 0 1 1 0-15 7.5 7.5 0 0 1 0 15Zm6.8 2.2-4.2-4.2" fill="none" stroke="currentColor" stroke-linecap="round" stroke-width="2" />',
  close:
    '<path d="M15.5 8.5 8.5 15.5M8.5 8.5l7 7" stroke="currentColor" stroke-width="2" stroke-linecap="round" />',
  gear: '<path d="M10.3 2h3.4l.5 2.4c.5.2 1 .4 1.5.6l2.1-1.2 2.4 2.4-1.2 2.1c.2.5.4 1 .6 1.5L22 10.3v3.4l-2.4.5c-.2.5-.4 1-.6 1.5l1.2 2.1-2.4 2.4-2.1-1.2c-.5.2-1 .4-1.5.6L13.7 22h-3.4l-.5-2.4c-.5-.2-1-.4-1.5-.6l-2.1 1.2-2.4-2.4 1.2-2.1c-.2-.5-.4-1-.6-1.5L2 13.7v-3.4l2.4-.5c.2-.5.4-1 .6-1.5L3.8 6.2l2.4-2.4 2.1 1.2c.5-.2 1-.4 1.5-.6L10.3 2Zm1.7 6.2A3.8 3.8 0 1 0 12 15.8 3.8 3.8 0 0 0 12 8.2Z" fill="currentColor"></path>',
} as const;
