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
  import { messageBadge } from '#lib/stores/messageBadge.svelte.ts';

  import ChatMenus, { type MenuAt } from './ChatMenus.svelte';
  import ConversationView from './ConversationView.svelte';
  import { InboxState } from './inboxState.svelte.ts';
  import PublicChatView from './PublicChatView.svelte';
  import { PublicState } from './publicState.svelte.ts';

  import '#lib/components/styles/messages.css';

  /**
   * web's MessagesPage: the Personal list (direct and group chats, newest first) with the
   * New message button, the Public list (your projects' chats), and a chat opened from either
   * (`?conversation=<id>` or `?project=<id>`).
   */
  const inbox = new InboxState();
  const publicChats = new PublicState();
  let composing = $state(false);
  let tab = $state<'personal' | 'public'>('personal');
  let menu = $state<MenuAt | null>(null);
  let infoOpen = $state(false);
  let menuError = $state('');

  const openId = $derived(page.url.searchParams.get('conversation'));
  const openProject = $derived(page.url.searchParams.get('project'));
  const active = $derived(inbox.all.find((c) => c.id === openId && c.member) ?? null);
  const activeProject = $derived(publicChats.chats.find((c) => c.id === openProject) ?? null);
  const chatOpen = $derived(!!active || !!activeProject);

  // untrack: a refresh reads list state, which must not re-run (and so stop) this effect.
  $effect(() => {
    untrack(() => {
      inbox.start();
      publicChats.start();
    });
    return () => {
      inbox.dispose();
      publicChats.dispose();
    };
  });

  $effect(() => {
    if (openProject) tab = 'public';
  });

  // While the lists are here, the send icon counts what they show.
  $effect(() => {
    if (inbox.loaded && publicChats.loaded)
      messageBadge.unread = inbox.unreadTotal + publicChats.unreadTotal;
  });

  $effect(() => {
    chatChrome.immersive = chatOpen;
    return () => {
      chatChrome.immersive = false;
    };
  });

  function open(id: string | null, param = 'conversation'): void {
    const href = id ? `/messages?${param}=${encodeURIComponent(id)}` : '/messages';
    void goto(resolve(href as '/messages'));
  }

  function openMenu(list: MenuAt['list'], id: string, x: number, y: number, header = false): void {
    menuError = '';
    menu = { list, id, x, y, header };
  }

  /** web's openChatInfoFromMenu: open the group, then its Chat info sheet. */
  function openInfo(id: string): void {
    if (openId !== id) open(id);
    infoOpen = true;
  }

  /** web's handleComposeTrigger: New message always starts a personal chat. */
  function compose(): void {
    tab = 'personal';
    composing = true;
  }

  async function send(chat: NewChat): Promise<void> {
    const id = await inbox.send(chat);
    composing = false;
    open(id);
  }
</script>

<div class="messages-page">
  <section class:conversation-page={chatOpen} class="page">
    {#if !chatOpen}
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

    <section class:conversation-view={chatOpen} class:list-view={!chatOpen} class="messages-shell">
      {#if active}
        {#key active.id}
          <ConversationView
            chat={active}
            {inbox}
            {infoOpen}
            onInfoClose={() => (infoOpen = false)}
            onMenu={(x, y) => openMenu('personal', active.id, x, y, true)}
            onBack={() => open(null)}
          />
        {/key}
      {:else if activeProject}
        {#key activeProject.id}
          <PublicChatView
            chat={activeProject}
            onBack={() => open(null)}
            onRead={() => publicChats.cleared(activeProject.id)}
            onMenu={(x, y) => openMenu('public', activeProject.id, x, y, true)}
          />
        {/key}
      {:else}
        <div class="surface-tabs" role="tablist" aria-label="Messages tabs">
          <div class="surface-tab-list">
            {@render tabButton('personal', 'Personal', inbox.unreadTotal)}
            {@render tabButton('public', 'Public', publicChats.unreadTotal)}
          </div>
        </div>

        {#if menuError}<p class="pin-error" role="alert">{menuError}</p>{/if}
        <div class="conversation-list">
          {#if tab === 'public'}
            {@render publicList()}
          {:else if openId && !inbox.loaded}
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
                preview={inbox.preview(chat)}
                time={relativeCompact(chat.last?.createdAt ?? chat.invitedAt)}
                unread={chat.unread}
                pinned={chat.pinned}
                muted={chat.muted}
                onOpen={() => open(chat.id)}
                onMenu={(x, y) => openMenu('personal', chat.id, x, y)}
              />
            {/each}
          {/if}
        </div>

        <button
          class="message-fab"
          type="button"
          aria-label="New message"
          title="New message"
          onclick={compose}
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

  <ChatMenus
    {menu}
    {inbox}
    {publicChats}
    onClose={() => (menu = null)}
    onInfo={openInfo}
    onGone={(id) => (id === openId || id === openProject) && open(null)}
    onError={(message) => (menuError = message)}
  />
</div>

{#snippet tabButton(id: 'personal' | 'public', label: string, unread: number)}
  <button
    class:active={tab === id}
    class="surface-tab"
    role="tab"
    type="button"
    aria-selected={tab === id}
    aria-label={unread > 0 ? `${label}, ${unread} unread` : label}
    onclick={() => (tab = id)}
  >
    <span class="surface-tab-label">{label}</span>
    {#if unread > 0}<CountBadge count={unread} />{/if}
  </button>
{/snippet}

{#snippet publicList()}
  {#if openProject && !publicChats.loaded}
    <div class="empty-state">Loading chats…</div>
  {:else if publicChats.error && publicChats.chats.length === 0}
    <div class="empty-state">{publicChats.error}</div>
  {:else if publicChats.chats.length === 0}
    <div class="empty-state">
      {publicChats.loaded ? 'No public chats yet.' : 'Loading chats…'}
    </div>
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
        onOpen={() => open(chat.id, 'project')}
        onMenu={(x, y) => openMenu('public', chat.id, x, y)}
      />
    {/each}
  {/if}
{/snippet}

<ComposeMessageSheet
  open={composing}
  search={(q) => inbox.search(q)}
  onSend={send}
  onClose={() => (composing = false)}
/>
