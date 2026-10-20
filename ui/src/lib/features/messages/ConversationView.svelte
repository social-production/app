<script lang="ts">
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { ChatSummary } from '#lib/domain/messages.ts';
  import { explain } from '#lib/services/errors.ts';
  import {
    deleteChatMessage,
    listChatMessages,
    markChatRead,
    postChatMessage,
  } from '#lib/services/messages.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';

  import { inTurn } from './writes.ts';

  /** web's MessagesPage with a conversation open: its header, then the chat (polled every
   * 5 s, web's THREAD_POLL_MS), marked read as messages arrive. */
  interface Props {
    chat: ChatSummary;
    title: string;
    names: Record<string, string>;
    onBack: () => void;
    onRead: () => void;
  }

  let { chat, title, names, onBack, onRead }: Props = $props();

  // The page keys this view by chat, so the first chat and names are the ones to keep.
  // svelte-ignore state_referenced_locally
  const seed = chat.id;
  // svelte-ignore state_referenced_locally
  const state = new ChatState(
    {
      list: () => listChatMessages(seed),
      post: (body, replyTo) => inTurn(() => postChatMessage(seed, body, replyTo)),
      remove: (id) => inTurn(() => deleteChatMessage(seed, id)),
      viewer,
      names: namesOf,
      explain,
      seen: () => void inTurn(() => markChatRead(seed)).then(onRead, () => undefined),
    },
    names,
  );

  // untrack: starting reads chat state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => state.start(() => true));
    return () => state.dispose();
  });
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
    <div class="identity-trigger">
      <div>
        <h2><span class="identity-title">{title}</span></h2>
        {#if chat.kind === 'group'}<p class="identity-note">Group</p>{/if}
      </div>
      <AvatarBadge size="md" name={title} />
    </div>
  </div>
</header>

<ChatPanel
  lines={state.lines}
  {title}
  placeholder="Write a message..."
  submitLabel="Send"
  emptyCopy={state.loaded ? 'No messages yet.' : 'Loading messages...'}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  hideAuthors={chat.kind === 'direct'}
  paused={state.paused}
  error={state.error}
  onSend={(body, replyTo) => state.send(body, replyTo)}
  onDelete={(id) => state.remove(id)}
/>
