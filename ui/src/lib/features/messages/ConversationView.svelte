<script lang="ts">
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ChatMarks from '#lib/components/messages/ChatMarks.svelte';
  import GroupInfoSheet from '#lib/components/messages/GroupInfoSheet.svelte';
  import PinBar from '#lib/components/messages/PinBar.svelte';
  import type { ChatPin, ChatSummary } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    chatPins,
    deleteChatMessage,
    editChatMessage,
    listChatMessages,
    markChatRead,
    pinChatMessage,
    postChatMessage,
  } from '#lib/services/messages.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';

  import type { InboxState } from './inboxState.svelte.ts';
  import { inTurn } from './writes.ts';

  /** web's MessagesPage with a conversation open: its header (tap it for the chat menu), the
   * pin bar, the chat (polled every 5 s, web's THREAD_POLL_MS, marked read as messages
   * arrive) and a group's "Chat info" sheet. */
  interface Props {
    chat: ChatSummary;
    inbox: InboxState;
    infoOpen: boolean;
    onInfoClose: () => void;
    onMenu: (x: number, y: number) => void;
    onBack: () => void;
  }

  let { chat, inbox, infoOpen, onInfoClose, onMenu, onBack }: Props = $props();

  // The page keys this view by chat, so the first chat and names are the ones to keep.
  // svelte-ignore state_referenced_locally
  const seed = chat.id;
  // svelte-ignore state_referenced_locally
  const thread = new ChatState(
    {
      list: () => listChatMessages(seed),
      post: (body, replyTo) => inTurn(() => postChatMessage(seed, body, replyTo)),
      remove: (id) => inTurn(() => deleteChatMessage(seed, id)),
      edit: (id, body) => inTurn(() => editChatMessage(seed, id, body)),
      pin: (id, pinned) => inTurn(() => pinChatMessage(seed, id, pinned)),
      viewer,
      names: namesOf,
      explain,
      seen: () =>
        void inTurn(() => markChatRead(seed)).then(
          () => inbox.refresh(),
          () => undefined,
        ),
    },
    inbox.names,
  );

  const title = $derived(inbox.title(chat));
  let pins = $state<ChatPin[]>([]);
  let focus = $state<{ id: string; n: number } | null>(null);
  const members = $derived(
    chat.members.map((agent) => ({ agent, name: inbox.names[agent] ?? '…' })),
  );

  // untrack: starting reads chat state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => thread.start(() => true));
    return () => thread.dispose();
  });

  // Pins follow each load of the chat (a pin or an edit changes them).
  $effect(() => {
    void thread.messages;
    void chatPins(seed).then(
      (list) => (pins = list),
      () => undefined,
    );
  });

  function jump(id: string): void {
    focus = { id, n: (focus?.n ?? 0) + 1 };
  }
</script>

<header class="chat-header">
  <button
    class="back-button"
    type="button"
    aria-label="Back to chats"
    title="Back to chats"
    onclick={onBack}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24" width="22" height="22">
      <path
        d="M15 5l-7 7 7 7"
        fill="none"
        stroke="currentColor"
        stroke-width="2.4"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
  <div class="chat-identity">
    <button
      class="identity-trigger"
      type="button"
      aria-label="Chat options"
      onclick={(event) => onMenu(event.clientX, event.clientY)}
    >
      <div>
        <h2>
          <span class="identity-title">{title}</span>
          <ChatMarks muted={chat.muted} header />
        </h2>
        {#if chat.kind === 'group'}<p class="identity-note">Group</p>{/if}
      </div>
      <AvatarBadge size="md" name={title} />
    </button>
  </div>
  <PinBar {pins} canUnpin={chat.canPin} onJump={jump} onUnpin={(id) => void thread.pin(id, true)} />
</header>

<ChatPanel
  lines={thread.lines}
  {title}
  placeholder="Write a message..."
  submitLabel="Send"
  emptyCopy={thread.loaded ? 'No messages yet.' : 'Loading messages...'}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  hideAuthors={chat.kind === 'direct'}
  paused={thread.paused}
  error={thread.error}
  {focus}
  onSend={(body, replyTo) => thread.send(body, replyTo)}
  onDelete={(id) => thread.remove(id)}
  onEdit={(id, body) => thread.edit(id, body)}
  onTogglePin={chat.canPin ? (id, pinned) => thread.pin(id, pinned) : null}
/>

{#if chat.kind === 'group'}
  <GroupInfoSheet
    open={infoOpen}
    {title}
    me={inbox.me}
    manager={chat.manager}
    {members}
    search={(q) => inbox.search(q)}
    onRename={(name) => inbox.rename(seed, name)}
    onAdd={(c) => inbox.addMember(seed, c.agent)}
    onRemove={(c) => inbox.removeMember(seed, c.agent)}
    onClose={onInfoClose}
  />
{/if}
