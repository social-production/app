<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import ChatRow from '#lib/components/messages/ChatRow.svelte';
  import { relativeCompact } from '#lib/domain/time.ts';

  import type { PublicState } from './publicState.svelte.ts';

  /** Messages' Public tab: the chats of the spaces you're in, each with its type's tile. */
  interface Props {
    publicChats: PublicState;
    /** A linked chat is opening: rows load under it. */
    waiting: boolean;
    onOpen: (id: string) => void;
    onMenu: (id: string, x: number, y: number) => void;
  }

  let { publicChats, waiting, onOpen, onMenu }: Props = $props();
</script>

{#if waiting && !publicChats.loaded}
  <SkeletonRows kind="row" count={5} />
{:else if publicChats.error && publicChats.chats.length === 0}
  <div class="empty-state">{publicChats.error}</div>
{:else if !publicChats.loaded}
  <SkeletonRows count={4} kind="row" />
{:else if publicChats.chats.length === 0}
  <EmptyState
    icon="message"
    title={t('No public chats yet')}
    text={t(
      'Join a project, event or request and its chat shows here, next to everyone taking part.',
    )}
  />
{:else}
  {#each publicChats.chats as chat (chat.id)}
    <ChatRow
      title={chat.title}
      preview={chat.last?.preview ?? ''}
      time={relativeCompact(chat.activeAt)}
      unread={chat.unread}
      pinned={chat.pinned}
      muted={chat.muted}
      kind="Project"
      mode={chat.mode}
      onOpen={() => onOpen(chat.id)}
      onMenu={(x, y) => onMenu(chat.id, x, y)}
    />
  {/each}
{/if}
