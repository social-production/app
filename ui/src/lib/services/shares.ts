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
}

/** web's `buildSharePrefill`: the body "Create post" starts from. */
export function sharePrefill(title: string, url: string): string {
  return `Check this out: ${title}\n${url}`;
}
