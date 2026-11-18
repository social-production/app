<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import ChatMenu, { type ChatMenuItem } from '#lib/components/messages/ChatMenu.svelte';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import type { ListPrefs } from '#lib/domain/messages.ts';

  import type { InboxState } from './inboxState.svelte.ts';
  import type { PublicState } from './publicState.svelte.ts';

  /** Where web's chat menu was opened: a row of either list, or an open chat's header. */
  export interface MenuAt {
    list: 'personal' | 'public';
    id: string;
    x: number;
    y: number;
    header: boolean;
  }

  /**
   * web's chat menu (MessagesPage `chatMenu`): View profile (direct), Pin, Mute, Chat info
   * (groups), Delete, and Leave group after a divider; on a project chat's header, Mute only.
   * Leaving asks first, as on web.
   */
  interface Props {
    menu: MenuAt | null;
    inbox: InboxState;
    publicChats: PublicState;
    onClose: () => void;
    onInfo: (id: string) => void;
    /** A chat that was open was deleted or left. */
    onGone: (id: string) => void;
    onError: (message: string) => void;
  }

  let { menu, inbox, publicChats, onClose, onInfo, onGone, onError }: Props = $props();

  let leaving = $state<{ id: string; started: boolean } | null>(null);

  function apply(at: MenuAt, prefs: ListPrefs): void {
    const write =
      at.list === 'personal' ? inbox.setPrefs(at.id, prefs) : publicChats.setPrefs(at.id, prefs);
    void write.then(
      () => prefs.hidden && onGone(at.id),
      (e: unknown) => onError(e instanceof Error ? e.message : t('Could not update the chat')),
    );
  }

  function items(at: MenuAt): ChatMenuItem[] {
    if (at.list === 'public') {
      const chat = publicChats.chats.find((c) => c.id === at.id);
      const mute = {
        label: chat?.muted ? t('Unmute') : t('Mute'),
        icon: chat?.muted ? 'bell' : 'mute',
        onSelect: () => apply(at, { muted: !chat?.muted }),
      };
      if (at.header) return [mute];
      return [
        {
          label: chat?.pinned ? t('Unpin') : t('Pin'),
          icon: 'pin',
          onSelect: () => apply(at, { pinned: !chat?.pinned }),
        },
        mute,
        {
          label: t('Delete'),
          icon: 'trash',
          danger: true,
          onSelect: () => apply(at, { hidden: true }),
        },
      ];
    }
    const chat = inbox.all.find((c) => c.id === at.id);
    if (!chat) return [];
    const partner = chat.kind === 'direct' ? chat.members.find((m) => m !== inbox.me) : undefined;
    const group = chat.kind === 'group';
    const list: ChatMenuItem[] = [];
    if (partner)
      list.push({
        label: t('View profile'),
        icon: 'user',
        href: resolve('/profile/[agent]', { agent: partner }),
      });
    list.push(
      {
        label: chat.pinned ? t('Unpin') : t('Pin'),
        icon: 'pin',
        onSelect: () => apply(at, { pinned: !chat.pinned }),
      },
      {
        label: chat.muted ? t('Unmute') : t('Mute'),
        icon: chat.muted ? 'bell' : 'mute',
        onSelect: () => apply(at, { muted: !chat.muted }),
      },
    );
    if (group) list.push({ label: t('Chat info'), icon: 'info', onSelect: () => onInfo(at.id) });
    // web: deleting a group from your list is leaving it.
    const del = group ? () => leave(at.id) : () => apply(at, { hidden: true });
    list.push({ label: t('Delete chat'), icon: 'trash', danger: true, onSelect: del });
    if (group) {
      list.push({
        label: t('Leave group'),
        icon: 'leave',
        danger: true,
        onSelect: () => (leaving = { id: at.id, started: chat.manager === inbox.me }),
      });
    }
    return list;
  }

  async function leave(id: string): Promise<void> {
    try {
      await inbox.leave(id);
      onGone(id);
    } catch (e) {
      onError(e instanceof Error ? e.message : t('Could not leave the group.'));
    }
  }

  function settle(ok: boolean): void {
    const target = leaving;
    leaving = null;
    if (ok && target) void leave(target.id);
  }
</script>

{#if menu}
  <ChatMenu x={menu.x} y={menu.y} items={items(menu)} {onClose} />
{/if}

<ConfirmDialog
  open={leaving !== null}
  title={t('Leave this group?')}
  body={leaving?.started
    ? t(
        "You'll stop getting its messages. You started it, so the longest-standing member takes over managing it. Someone in the group can add you back.",
      )
    : t("You'll stop getting its messages. Someone in the group can add you back.")}
  confirmLabel={t('Leave')}
  danger
  onSettle={settle}
/>
