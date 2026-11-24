import type { ChatSummary, Contact } from '#lib/domain/messages.ts';

import { chatSummary, contacts, myChats } from './messages.ts';
import { myAgent } from './people.ts';

/** How long the list is reused before it is read again. */
const FRESH_MS = 5 * 60 * 1000;
let cached: { at: number; list: Promise<Contact[]> } | null = null;

const lastAt = (s: ChatSummary): number => s.last?.createdAt ?? s.invitedAt;

async function load(): Promise<Contact[]> {
  const [all, me, chats] = await Promise.all([contacts(), myAgent(), myChats().catch(() => [])]);
  const recent = [...chats].sort((a, b) => b.at - a.at).slice(0, 12);
  const summaries = await Promise.all(
    recent.map((c) => chatSummary(c.seed, c.at).catch(() => null)),
  );
  const talked = summaries
    .filter((s): s is ChatSummary => !!s && s.kind === 'direct')
    .sort((a, b) => lastAt(b) - lastAt(a))
    .flatMap((s) => s.members.filter((m) => m !== me));
  const rank = (agent: string): number => {
    const i = talked.indexOf(agent);
    return i < 0 ? Number.MAX_SAFE_INTEGER : i;
  };
  return all.filter((c) => c.agent !== me).sort((a, b) => rank(a.agent) - rank(b.agent));
}

/** Everyone but you, the people you talk to most first (Owner Notes Mockups §6: the share
 * sheet's faces). */
export function shareContacts(): Promise<Contact[]> {
  if (!cached || Date.now() - cached.at > FRESH_MS) {
    const list = load();
    cached = { at: Date.now(), list };
    list.catch(() => (cached = null));
  }
  return cached.list;
}

/** The share sheet's search: with nothing typed, the people you talk to most. */
export async function searchShareContacts(
  query: string,
  exclude: string[] = [],
): Promise<Contact[]> {
  const needle = query.trim().toLowerCase();
  return (await shareContacts())
    .filter((c) => !exclude.includes(c.agent) && c.name.toLowerCase().includes(needle))
    .slice(0, needle ? 8 : 12);
}
