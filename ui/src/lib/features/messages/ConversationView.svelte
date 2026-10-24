<script lang="ts">
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import { withReceipts } from '#lib/components/chat/chatFormat.ts';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ChatMarks from '#lib/components/messages/ChatMarks.svelte';
  import GroupInfoSheet from '#lib/components/messages/GroupInfoSheet.svelte';
  import PinBar from '#lib/components/messages/PinBar.svelte';
  import type { ChatPin, ChatSummary } from '#lib/domain/messages.ts';
  import { attachmentBlob } from '#lib/services/attachments.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    chatPeerReadAt,
    chatPins,
    deleteChatMessage,
    editChatMessage,
    listChatMessages,
    markChatRead,
    mentionPool,
    pinChatMessage,
    postChatMessage,
  } from '#lib/services/messages.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';
  import { callRequest } from '#lib/stores/callRequest.svelte.ts';

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
      post: (body, replyTo, files) => inTurn(() => postChatMessage(seed, body, replyTo, files)),
      file: (attachment) => attachmentBlob(seed, attachment),
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

  // Pins and the read receipt follow each load of the chat (web reads `peer_read_at` with
  // the thread); receipts are for direct chats only, as on web.
  let peerReadAt = $state<number | null>(null);
  const lines = $derived(
    chat.kind === 'direct' ? withReceipts(thread.lines, peerReadAt) : thread.lines,
  );
  $effect(() => {
    void thread.messages;
    void chatPins(seed).then(
      (list) => (pins = list),
      () => undefined,
    );
    if (chat.kind === 'direct') {
      void chatPeerReadAt(seed).then(
        (at) => (peerReadAt = at),
        () => undefined,
      );
    }
  });

  // The chat's members come first in @mention suggestions, then your other chats' (ADR 0047).
  const mentions = new MentionSource(
    () => mentionPool(inbox.all),
    () => ({ me: thread.me, agents: chat.members, names: thread.names }),
  );

  $effect(() => mentions.watch(thread.messages.map((m) => m.body)));

  function jump(id: string): void {
    focus = { id, n: (focus?.n ?? 0) + 1 };
  }
</script>

<header class="chat-header" class:with-call={chat.kind === 'direct'}>
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
  {#if chat.kind === 'direct'}
    <button
      class="call-button"
      type="button"
      aria-label={`Call ${title}`}
      title={`Call ${title}`}
      onclick={() =>
        (callRequest.next = {
          seed,
          name: title,
          peer: chat.members.find((m) => m !== inbox.me) ?? null,
        })}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.13.96.36 1.9.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.91.34 1.85.57 2.81.7A2 2 0 0 1 22 16.92z"
        />
      </svg>
    </button>
  {/if}
  <PinBar {pins} canUnpin={chat.canPin} onJump={jump} onUnpin={(id) => void thread.pin(id, true)} />
</header>

<ChatPanel
  {lines}
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
  onSend={(body, replyTo, files) => thread.send(body, replyTo, files)}
  fileUrl={thread.fileUrl}
  onDelete={(id) => thread.remove(id)}
  onEdit={(id, body) => thread.edit(id, body)}
  onTogglePin={chat.canPin ? (id, pinned) => thread.pin(id, pinned) : null}
  mentionSearch={mentions.search}
  mentionAgent={mentions.agentOf}
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
