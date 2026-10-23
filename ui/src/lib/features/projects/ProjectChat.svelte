<script lang="ts">
  import ChatPanel from '#lib/components/chat/ChatPanel.svelte';
  import { ChatState } from '#lib/components/chat/chatState.svelte.ts';
  import type { Contact } from '#lib/domain/messages.ts';
  import { deleteMessage, listMessages, postMessage } from '#lib/services/chat.ts';
  import { explain } from '#lib/services/errors.ts';
  import { contacts } from '#lib/services/messages.ts';
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

  // web's mention suggestions search everyone; here that's the directory's people list, read
  // once, when someone types "@" or a message mentions someone (ADR 0047).
  let people = $state<Contact[]>([]);
  let loadingPeople: Promise<Contact[]> | undefined;

  function everyone(): Promise<Contact[]> {
    loadingPeople ??= contacts().then(
      (all) => (people = all),
      (e: unknown) => {
        loadingPeople = undefined;
        throw e;
      },
    );
    return loadingPeople;
  }

  async function mentionSearch(query: string): Promise<Contact[]> {
    const needle = query.toLowerCase();
    const found = (await everyone())
      .filter((c) => c.agent !== chat.me && c.name.toLowerCase().includes(needle))
      .slice(0, 8);
    // Someone new may not have reached this node's list yet: read it again next keystroke.
    if (found.length === 0) loadingPeople = undefined;
    return found;
  }

  /** Names aren't unique: someone in this chat first, then the directory. */
  function mentionAgent(name: string): string | null {
    const lower = name.toLowerCase();
    const inChat = Object.entries(chat.names).find(([, n]) => n.toLowerCase() === lower);
    return inChat?.[0] ?? people.find((c) => c.name.toLowerCase() === lower)?.agent ?? null;
  }

  $effect(() => {
    if (chat.messages.some((m) => m.body.includes('@'))) everyone().catch(() => undefined);
  });

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
    {mentionSearch}
    {mentionAgent}
  />
</section>
