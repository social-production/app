<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import { untrack } from 'svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import CountBadge from '#lib/components/icons/CountBadge.svelte';
  import ChatRow from '#lib/components/messages/ChatRow.svelte';
  import type { Contact } from '#lib/domain/messages.ts';
  import { relativeCompact } from '#lib/domain/time.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';
  import { messageBadge } from '#lib/stores/messageBadge.svelte.ts';

  import ChatMenus, { type MenuAt } from './ChatMenus.svelte';
  import ConversationView from './ConversationView.svelte';
  import DraftChatView from './DraftChatView.svelte';
  import { InboxState } from './inboxState.svelte.ts';
  import NewMessagePage from './NewMessagePage.svelte';
  import PublicChatView from './PublicChatView.svelte';
  import { PublicState } from './publicState.svelte.ts';
  import '#lib/components/styles/messages.css';

  /**
   * web's MessagesPage: the Personal list (direct and group chats, newest first) with the
   * New message button, the Public list (your projects' chats), and a chat opened from either
   * (`?conversation=<id>` or `?project=<id>`). New message is Telegram's (`?new=message`,
   * `?new=group`); `?to=<agent>` is the chat with someone, empty until the first message.
   */
  const inbox = new InboxState();
  const publicChats = new PublicState();
  let tab = $state<'personal' | 'public'>('personal');
  let menu = $state<MenuAt | null>(null);
  let infoOpen = $state(false);
  let menuError = $state('');

  const openId = $derived(page.url.searchParams.get('conversation'));
  const openProject = $derived(page.url.searchParams.get('project'));
  const about = $derived(page.url.searchParams.get('about') === '1');
  const active = $derived(inbox.all.find((c) => c.id === openId && c.member) ?? null);
  const activeProject = $derived(publicChats.chats.find((c) => c.id === openProject) ?? null);
  const composing = $derived(page.url.searchParams.get('new'));
  const toAgent = $derived(page.url.searchParams.get('to'));
  let draft = $state<Contact | null>(null);
  const chatOpen = $derived(!!active || !!activeProject || !!draft);

  // `?to=`: your chat with that person if you have one, else an empty one to write into.
  $effect(() => {
    const agent = toAgent;
    if (!agent || !inbox.loaded) return void (draft = null);
    const existing = inbox.directWith(agent);
    if (existing) return open(existing.id, 'conversation', true);
    if (draft?.agent === agent) return;
    draft = { agent, name: inbox.names[agent] ?? '…' };
    void inbox.everyone().then(
      (all) => {
        const found = all.find((c) => c.agent === agent);
        if (found && toAgent === agent) draft = found;
      },
      () => undefined,
    );
  });

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
    chatChrome.immersive = chatOpen || !!composing;
    return () => {
      chatChrome.immersive = false;
    };
  });

  function open(id: string | null, param = 'conversation', replaceState = false, more = ''): void {
    const href = id ? `/messages?${param}=${encodeURIComponent(id)}${more}` : '/messages';
    void goto(resolve(href as '/messages'), { replaceState });
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

  async function createGroup(members: Contact[], title: string): Promise<void> {
    open(await inbox.send({ kind: 'group', members, title, body: '' }), 'conversation', true);
  }

  async function first(person: Contact, body: string, files: File[]): Promise<void> {
    const id = await inbox.send({ kind: 'direct', members: [person], title: '', body }, files);
    open(id, 'conversation', true);
  }
</script>

<div class="messages-page">
  <section class:conversation-page={chatOpen} class="page">
    {#if !chatOpen}
      <div class="desktop-only-header">
        <header class="page-header">
          <div class="copy">
            <h1>{t('Messages')}</h1>
            <p>
              {t(
                'Direct messages, group chats, and the same project or event chat rooms you already use elsewhere.',
              )}
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
            {about}
            onAbout={(show) =>
              show ? open(active.id, 'conversation', false, '&about=1') : history.back()}
            onInfo={() => (infoOpen = true)}
            onOpenChat={(id) => open(id)}
          />
        {/key}
      {:else if draft}
        {@const person = draft}
        {#key person.agent}
          <DraftChatView
            {person}
            onFirst={(body, files) => first(person, body, files)}
            onBack={() => open(null)}
          />
        {/key}
      {:else if composing}
        <NewMessagePage
          group={composing === 'group'}
          people={() => inbox.everyone()}
          onBack={() => (composing === 'group' ? open('message', 'new', true) : open(null))}
          onPerson={(person) => open(person.agent, 'to', true)}
          onGroup={() => open('group', 'new')}
          onCreate={createGroup}
        />
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
        <div class="surface-tabs" role="tablist" aria-label={t('Messages tabs')}>
          <div class="surface-tab-list">
            {@render tabButton('personal', 'Personal', inbox.unreadTotal)}
            {@render tabButton('public', 'Public', publicChats.unreadTotal)}
          </div>
        </div>

        {#if menuError}<p class="pin-error" role="alert">{menuError}</p>{/if}
        <div class="conversation-list">
          {#if tab === 'public'}
            {@render publicList()}
          {:else if (openId || toAgent) && !inbox.loaded}
            <SkeletonRows kind="row" count={5} />
          {:else if inbox.error && inbox.chats.length === 0}
            <EmptyState icon="info" title={t("Your chats didn't load")} text={inbox.error} />
          {:else if inbox.chats.length === 0}
            {#if inbox.loaded}
              <EmptyState
                icon="send"
                title={t('No messages yet')}
                text={t('Start a chat with someone you work with.')}
              >
                <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved, plus a query -->
                <a class="empty-action" href={`${resolve('/messages')}?new=message`}
                  >{t('New message')}</a
                >
              </EmptyState>
            {:else}
              <SkeletonRows kind="row" count={5} />
            {/if}
          {:else}
            {#each inbox.chats as chat (chat.id)}
              <ChatRow
                title={inbox.title(chat)}
                agent={inbox.partner(chat)}
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
          aria-label={t('New message')}
          title={t('New message')}
          onclick={() => open('message', 'new')}
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
    <span class="surface-tab-label">{t(label)}</span>
    {#if unread > 0}<CountBadge count={unread} />{/if}
  </button>
{/snippet}

{#snippet publicList()}
  {#if openProject && !publicChats.loaded}
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
        onOpen={() => open(chat.id, 'project')}
        onMenu={(x, y) => openMenu('public', chat.id, x, y)}
      />
    {/each}
  {/if}
{/snippet}
