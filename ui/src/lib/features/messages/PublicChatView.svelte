<script lang="ts">
  import { untrack } from 'svelte';

  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ChatMarks from '#lib/components/messages/ChatMarks.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import type { PublicChat } from '#lib/domain/messages.ts';
  import type { Person } from '#lib/domain/people.ts';
  import { deleteMessage, listMessages, postMessage } from '#lib/services/chat.ts';
  import { explain } from '#lib/services/errors.ts';
  import { mentionPool } from '#lib/services/messages.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';
  import { markProjectChatRead, projectChatMembers } from '#lib/services/publicChats.ts';

  import { inTurn } from './writes.ts';

  /** web's MessagesPage with a project chat open from the Public tab: its header ("Project
   * chat · N members"), then the same chat as the project's Chat tab, read as it arrives. */
  interface Props {
    chat: PublicChat;
    onBack: () => void;
    onRead: () => void;
    /** web's header menu (Mute). */
    onMenu: (x: number, y: number) => void;
  }

  let { chat, onBack, onRead, onMenu }: Props = $props();

  // The page keys this view by chat, so the first id is the one to keep.
  // svelte-ignore state_referenced_locally
  const id = chat.id;
  const project = { seed: id };
  const talk = new ChatState(
    {
      list: () => listMessages(project),
      post: (body, replyTo) => inTurn(() => postMessage(project, body, replyTo)),
      remove: (messageId) => inTurn(() => deleteMessage(project, messageId)),
      viewer,
      names: namesOf,
      explain,
    },
    {},
  );

  // People who wrote here come first in @mention suggestions (ADR 0047).
  const mentions = new MentionSource(
    () => mentionPool(),
    () => ({ me: talk.me, agents: Object.keys(talk.names), names: talk.names }),
  );

  $effect(() => mentions.watch(talk.messages.map((m) => m.body)));

  const members = $derived(`${chat.members} ${chat.members === 1 ? 'member' : 'members'}`);

  // web's members panel: the count opens the project's members, read when asked for.
  let membersOpen = $state(false);
  let people = $state<Person[]>([]);
  let membersRequest = 0;

  async function toggleMembers() {
    if (membersOpen) {
      membersOpen = false;
      return;
    }
    const request = ++membersRequest;
    try {
      const found = await projectChatMembers(id);
      if (request !== membersRequest) return;
      people = found;
      membersOpen = true;
    } catch (cause) {
      if (request === membersRequest) talk.error = explain(cause);
    }
  }

  // untrack: starting reads chat state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => talk.start(() => true));
    return () => talk.dispose();
  });

  $effect(() => {
    const newest = talk.messages.at(-1)?.createdAt;
    if (newest) void markProjectChatRead(id, newest).then(onRead);
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
        <button
          class="title-hit"
          type="button"
          onclick={(event) => onMenu(event.clientX, event.clientY)}
        >
          <h2>
            <span class="identity-title">{chat.title}</span>
            <ChatMarks muted={chat.muted} header />
          </h2>
        </button>
        <p class="identity-note">
          Project chat ·
          <button
            aria-expanded={membersOpen}
            class="members-link"
            type="button"
            onclick={toggleMembers}>{members}</button
          >
        </p>
      </div>
      <button
        class="avatar-hit"
        type="button"
        aria-label="Chat options"
        title="Chat options"
        onclick={(event) => onMenu(event.clientX, event.clientY)}
      >
        <AvatarBadge size="md" name={chat.title} />
      </button>
    </div>
  </div>
</header>

<ChatPanel
  lines={talk.lines}
  title={chat.title}
  placeholder="Message the project..."
  submitLabel="Send"
  emptyCopy={talk.loaded ? 'No project chat yet.' : 'Loading messages...'}
  embedded={true}
  fitViewport={false}
  showHeader={false}
  paused={talk.paused}
  error={talk.error}
  onSend={(body, replyTo) => talk.send(body, replyTo)}
  onDelete={(messageId) => talk.remove(messageId)}
  mentionSearch={mentions.search}
  mentionAgent={mentions.agentOf}
/>

<PeopleSheet
  open={membersOpen}
  title="Project members"
  description="Members coordinate planning, updates, and activity together."
  emptyCopy="No members listed yet."
  {people}
  onClose={() => (membersOpen = false)}
/>
