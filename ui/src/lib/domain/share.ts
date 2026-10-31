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

const HELP_SHARE =
  /^Shared a help request: (.+)\nOpen help request: \S*\/help-requests\/([^\s/?#]+)$/;

/** A help request shared in a message, as `shareMessage` wrote it. */
export function sharedHelpRequest(body: string): { title: string; id: string } | null {
  const [, title, id] = HELP_SHARE.exec(body.trim()) ?? [];
  return title && id ? { title, id: decodeURIComponent(id) } : null;
}

/**
 * shares.py's `hr-share` notification ("A help request was shared with you."), derived from the
 * share message someone else sent you in a direct chat (ADR 0067).
 */
export function helpShareNotifications(seed: string, messages: ChatMessage[], me: string) {
  return messages.flatMap((m) => {
    const shared =
      m.author === me || (m.kind ?? 'text') !== 'text' ? null : sharedHelpRequest(m.body);
    if (!shared) return [];
    return [
      {
        id: `hr-share:${m.id}`,
        kind: 'hr-share' as const,
        subject: 'project' as const,
        seed,
        projectMode: 'help-request' as const,
        actor: m.author,
        title: shared.title,
        body: 'A help request was shared with you.',
        lead: null,
        target: { helpRequest: shared.id },
        createdAt: m.createdAt,
      },
    ];
  });
}
