import { shareNotice } from './pushNotices.ts';
import type { Contact } from '#lib/domain/messages.ts';
import { shareMessage } from '#lib/domain/share.ts';

import { sendNewChat } from './messages.ts';

/**
 * shares.py `send_share_message` for a project or an event (`share_project_with_user`,
 * `share_event_with_user`): the link goes into your direct chat with them (started if you have
 * none), and their device shows it as web's `prj-share` / `evt-share` notification.
 */
export async function shareSpace(
  to: Contact,
  label: 'project' | 'event',
  title: string,
  url: string,
): Promise<void> {
  await sendNewChat({
    kind: 'direct',
    members: [to],
    title: '',
    body: shareMessage(label, title, url),
  });
  void shareNotice(to.agent, label, title, url);
}

/** Detail Pages Cleanup §5, "Share profile": a link to someone's profile instead of their key,
 * sent into your direct chat with the recipient. */
export async function shareProfile(to: Contact, name: string, url: string): Promise<void> {
  await sendNewChat({
    kind: 'direct',
    members: [to],
    title: '',
    body: `${name}'s profile: ${url}`,
  });
}

/** Any shared thing into your direct chat with them: spaces as shares.py words them (their
 * notice comes with them), a comment as its words and link. */
export async function shareTo(
  to: Contact,
  kind: 'project' | 'event' | 'help request' | 'thread' | 'post' | 'comment' | 'profile',
  title: string,
  url: string,
): Promise<void> {
  if (kind === 'profile') return shareProfile(to, title, url);
  const body = kind === 'comment' ? `${title}\n${url}` : shareMessage(kind, title, url);
  await sendNewChat({ kind: 'direct', members: [to], title: '', body });
  if (kind !== 'comment') void shareNotice(to.agent, kind, title, url);
}
