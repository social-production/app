<script lang="ts">
  import { untrack } from 'svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import CountBadge from '#lib/components/icons/CountBadge.svelte';
  import ChatRow from '#lib/components/messages/ChatRow.svelte';
  import ComposeMessageSheet from '#lib/components/messages/ComposeMessageSheet.svelte';
  import type { NewChat } from '#lib/domain/messages.ts';
  import { relativeCompact } from '#lib/domain/time.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';

  import ConversationView from './ConversationView.svelte';
  import { InboxState } from './inboxState.svelte.ts';

  import '#lib/components/styles/messages.css';

  /**
   * web's MessagesPage: the Personal list (direct and group chats, newest first) with the
   * New message button, and a conversation opened from it (`?conversation=<id>`). Project
   * chats (web's Public tab) stay on their project pages for now.
   */
  const inbox = new InboxState();
  let composing = $state(false);

  const openId = $derived(page.url.searchParams.get('conversation'));
  const active = $derived(inbox.chats.find((c) => c.id === openId) ?? null);

  // untrack: a refresh reads inbox state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => inbox.start());
    return () => inbox.dispose();
  });

  $effect(() => {
    chatChrome.immersive = !!active;
    return () => {
      chatChrome.immersive = false;
    };
  });

  function open(id: string | null): void {
    const href = id ? `/messages?conversation=${encodeURIComponent(id)}` : '/messages';
    void goto(resolve(href as '/messages'));
  }

  async function send(chat: NewChat): Promise<void> {
    const id = await inbox.send(chat);
    composing = false;
    open(id);
  }
</script>

<div class="messages-page">
  <section class:conversation-page={!!active} class="page">
    {#if !active}
      <div class="desktop-only-header">
        <header class="page-header">
          <div class="copy">
            <h1>Messages</h1>
            <p>
              Direct messages, group chats, and the same project or event chat rooms you already use
              elsewhere.
            </p>
          </div>
        </header>
      </div>
    {/if}

    <section class:conversation-view={!!active} class:list-view={!active} class="messages-shell">
      {#if active}
        {#key active.id}
          <ConversationView
            chat={active}
            title={inbox.title(active)}
            names={inbox.names}
            onBack={() => open(null)}
            onRead={() => void inbox.refresh()}
          />
        {/key}
      {:else}
        <div class="surface-tabs" role="tablist" aria-label="Messages tabs">
          <div class="surface-tab-list">
            <button class="surface-tab active" role="tab" type="button" aria-selected="true">
              <span class="surface-tab-label">Personal</span>
              {#if inbox.unreadTotal > 0}<CountBadge count={inbox.unreadTotal} />{/if}
            </button>
          </div>
        </div>

        <div class="conversation-list">
          {#if openId && !inbox.loaded}
            <div class="empty-state">Loading chats…</div>
          {:else if inbox.error && inbox.chats.length === 0}
            <div class="empty-state">{inbox.error}</div>
          {:else if inbox.chats.length === 0}
            <div class="empty-state">
              {inbox.loaded ? 'No personal messages yet.' : 'Loading chats…'}
            </div>
          {:else}
            {#each inbox.chats as chat (chat.id)}
              <ChatRow
                title={inbox.title(chat)}
                preview={chat.last?.preview ?? ''}
                time={relativeCompact(chat.last?.createdAt ?? chat.invitedAt)}
                unread={chat.unread}
                onOpen={() => open(chat.id)}
              />
            {/each}
          {/if}
        </div>

        <button
          class="message-fab"
          type="button"
          aria-label="New message"
          title="New message"
          onclick={() => (composing = true)}
        >
          <svg aria-hidden="true" viewBox="0 0 24 24">
            <path
              d="M12 5v14M5 12h14"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
            />
          </svg>
        </button>
      {/if}
    </section>
  </section>
</div>

<ComposeMessageSheet
  open={composing}
  search={(q) => inbox.search(q)}
  onSend={send}
  onClose={() => (composing = false)}
/>
