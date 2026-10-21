import { chatSummary, myChats } from './messages.ts';
import { myProjectChats } from './publicChats.ts';

/**
 * web's get_total_unread_message_count: unread messages in every chat you're in and haven't
 * deleted, plus your project chats (the Public tab). A chat that doesn't answer counts 0.
 */
export async function unreadMessageCount(): Promise<number> {
  const [personal, projects] = await Promise.all([personalUnread(), myProjectChats()]);
  return personal + projects.reduce((sum, c) => sum + c.unread, 0);
}

async function personalUnread(): Promise<number> {
  const invites = await myChats();
  const settled = await Promise.allSettled(invites.map((i) => chatSummary(i.seed, i.at)));
  return settled.reduce(
    (sum, r) =>
      r.status === 'fulfilled' && r.value.member && !r.value.hidden ? sum + r.value.unread : sum,
    0,
  );
}
