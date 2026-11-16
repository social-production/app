/** The full-screen search (Chats, Search and Reports mockups §3): open, and what it starts with. */
export const searchOverlay = $state({ open: false, query: '' });

export function openSearch(query = ''): void {
  searchOverlay.query = query;
  searchOverlay.open = true;
}
