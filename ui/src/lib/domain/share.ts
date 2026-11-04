import type { ChatMessage } from './chat.ts';

/** web's utils/share.ts and web-backend's content/shares.py, without the browser parts. */

export interface ShareInput {
  url: string;
  title?: string;
}

/** web's shareTargets: the common places people share to, for browsers without a share sheet. */
export function shareTargets(input: ShareInput): { id: string; label: string; href: string }[] {
  const url = encodeURIComponent(input.url);
  const text = encodeURIComponent(input.title ?? '');
  const both = encodeURIComponent(`${input.title ?? ''} ${input.url}`.trim());
  return [
    { id: 'whatsapp', label: 'WhatsApp', href: `https://wa.me/?text=${both}` },
    { id: 'telegram', label: 'Telegram', href: `https://t.me/share/url?url=${url}&text=${text}` },
    { id: 'email', label: 'Email', href: `mailto:?subject=${text}&body=${both}` },
    { id: 'x', label: 'X', href: `https://x.com/intent/post?url=${url}&text=${text}` },
    {
      id: 'facebook',
      label: 'Facebook',
      href: `https://www.facebook.com/sharer/sharer.php?u=${url}`,
    },
  ];
}

/** shares.py `send_share_message`: what lands in the direct chat. */
export function shareMessage(kindLabel: string, title: string, href: string): string {
  return `Shared a ${kindLabel}: ${title}\nOpen ${kindLabel}: ${href}`;
}

/** What web's shares.py sends for each kind it shares: the message's label, the page's path,
 * and the notification it raises (`prj-share`, `evt-share`, `hr-share`, `thread-share`,
 * `post-share`). */
const SHARED = {
  project: { path: 'projects', kind: 'prj-share', mode: 'productive' },
  event: { path: 'events', kind: 'evt-share', mode: 'event' },
  'help request': { path: 'help-requests', kind: 'hr-share', mode: 'help-request' },
  thread: { path: 'threads', kind: 'thread-share', mode: 'thread' },
  post: { path: 'posts', kind: 'post-share', mode: 'post' },
} as const;

type SharedLabel = keyof typeof SHARED;

const SHARE =
  /^Shared a (project|event|help request|thread|post): (.+)\nOpen \1: \S*\/([a-z-]+)\/([^\s/?#]+)$/;

/** A space shared in a message, as `shareMessage` wrote it. */
export function sharedSpace(
  body: string,
): { label: SharedLabel; title: string; id: string } | null {
  const [, label, title, path, id] = SHARE.exec(body.trim()) ?? [];
  if (!label || !title || !id) return null;
  const known = SHARED[label as SharedLabel];
  return known.path === path
    ? { label: label as SharedLabel, title, id: decodeURIComponent(id) }
    : null;
}

/** shares.py's notification bodies: projects and events name themselves. */
function shareBody(label: SharedLabel, title: string): string {
  return label === 'project' || label === 'event'
    ? `${title} was shared with you.`
    : `A ${label} was shared with you.`;
}

/**
 * shares.py's share notifications, derived from the share messages someone else sent you in a
 * direct chat (ADR 0067): web sends the message and the notification together.
 */
export function shareNotifications(seed: string, messages: ChatMessage[], me: string) {
  return messages.flatMap((m) => {
    const shared = m.author === me || (m.kind ?? 'text') !== 'text' ? null : sharedSpace(m.body);
    if (!shared) return [];
    const { kind, mode } = SHARED[shared.label];
    return [
      {
        id: `${kind}:${m.id}`,
        kind,
        subject: 'project' as const,
        seed,
        projectMode: mode,
        actor: m.author,
        title: shared.title,
        body: shareBody(shared.label, shared.title),
        lead: null,
        target: { space: shared.id },
        createdAt: m.createdAt,
      },
    ];
  });
}
