<script lang="ts">
  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import { deleteMessage, listMessages, postMessage } from '#lib/services/chat.ts';
  import { explain } from '#lib/services/errors.ts';
  import { mentionPool } from '#lib/services/messages.ts';
  import { namesOf, viewer } from '#lib/services/people.ts';
  import { markProjectChatRead } from '#lib/services/publicChats.ts';

  /** web's ProjectChatTab: the project's "# project-chat", polled while the tab is open. */
  interface Props {
    projectId: string;
    /** The project's network seed: its read mark's key (ADR 0044). */
    seed: string;
    /** Names the project page already knows, by agent key. */
    names: Record<string, string>;
    fullscreen: boolean;
    active: boolean;
    /** web's `?comment=` link: the message to show first. */
    highlighted: string | null;
    onWrote: () => void;
  }

  let { projectId, seed, names, fullscreen, active, highlighted, onWrote }: Props = $props();

  // The page mounts this once per project, so the first id and names are the ones to keep.
  // svelte-ignore state_referenced_locally
  const chat = new ChatState(
    {
      list: () => listMessages(projectId),
      post: (body, replyTo) => postMessage(projectId, body, replyTo),
      remove: (id) => deleteMessage(projectId, id),
      viewer,
      names: namesOf,
      explain,
    },
    names,
  );
  let compact = $state(false);
  const embedded = $derived(compact || fullscreen);

  $effect(() => {
    const media = window.matchMedia('(max-width: 1080px)');
    const apply = (): void => {
      compact = media.matches;
    };
    apply();
    media.addEventListener('change', apply);
    return () => media.removeEventListener('change', apply);
  });

  $effect(() => {
    if (active) chat.start(() => active);
  });

  $effect(() => () => chat.dispose());

  // People in this project come first in @mention suggestions (ADR 0047).
  const mentions = new MentionSource(
    () => mentionPool(),
    () => ({ me: chat.me, agents: Object.keys(chat.names), names: chat.names }),
  );

  $effect(() => mentions.watch(chat.messages.map((m) => m.body)));

  // web's LinkedChatReadMarker: reading the Chat tab clears the project in Messages' Public tab.
  $effect(() => {
    const newest = chat.messages.at(-1)?.createdAt;
    if (active && newest) void markProjectChatRead(seed, newest);
  });
</script>

<section
  class="chat-shell"
  class:chat-shell-compact={embedded}
  class:chat-shell-fullscreen={fullscreen}
>
  <ChatPanel
    lines={chat.lines}
    title="# project-chat"
    placeholder="Message the project..."
    submitLabel="Send"
    emptyCopy={chat.loaded ? 'No project chat yet.' : 'Loading chat…'}
    {embedded}
    fitViewport={embedded}
    paused={chat.paused}
    error={chat.error}
    onSend={(body, replyTo) => chat.send(body, replyTo)}
    onDelete={(id) => chat.remove(id)}
    {highlighted}
    {onWrote}
    mentionSearch={mentions.search}
    mentionAgent={mentions.agentOf}
  />
</section>
