<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import DetailTopTabs from '#lib/components/DetailTopTabs.svelte';
  import { localDateTime } from '#lib/domain/helpRequests.ts';
  import type { DetailTabId } from '#lib/domain/projects.ts';
  import ProjectChat from '../ProjectChat.svelte';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';

  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import HelpRolesDock from './HelpRolesDock.svelte';
  import { HelpState } from './helpState.svelte.ts';

  import '#lib/components/styles/detail.css';

  /**
   * web's HelpRequestDetailPage: Details (the request, its roles and votes) and Chat (ADR 0067).
   * The page is re-created per id, so reading `id` once is intended.
   */
  let { id }: { id: string } = $props();

  // svelte-ignore state_referenced_locally
  const help = new HelpState(id);
  const params = page.url.searchParams;
  let commentLink = $state(params.get('comment'));
  let tab = $state<DetailTabId>(
    params.get('comment') || params.get('tab') === 'chat' ? 'chat' : 'context',
  );
  let compact = $state(false);
  let chatOpened = $state(false);
  const chatFull = $derived(tab === 'chat' && compact);

  $effect(() => {
    void help.open();
  });

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
    if (tab === 'chat') chatOpened = true;
    chatChrome.immersive = chatFull;
    return () => {
      chatChrome.immersive = false;
    };
  });
</script>

<div class="help-request-page">
  <section class="page" class:page-chat={chatFull}>
    {#if help.error && !help.project}
      <section class="hero-card"><p class="tab-loading">{help.error}</p></section>
    {:else if !help.project?.charter}
      <section class="hero-card"><p class="tab-loading">Loading help request…</p></section>
    {:else}
      {@const project = help.project}
      {@const charter = help.project.charter}
      <section class="hero-card" class:chat-tab-active={chatFull}>
        <DetailTopTabs
          active={tab}
          available={['context', 'chat']}
          ariaLabel="Help request detail tabs"
          onSelect={(t) => (tab = t)}
        />
        {#if tab === 'context'}
          <div class="context-tab help-overview">
            <div class="context-panel">
              <FeedCardTop tags={charter.tags} stage="" />
              <div class="heading overview-heading">
                <div class="identity-copy">
                  <h1>{charter.title}</h1>
                  {#if help.activity}
                    <p class="live-fact">{localDateTime(help.activity.startsAt)}</p>
                  {/if}
                  {#if charter.location_label}
                    <p class="live-fact">{charter.location_label}</p>
                  {/if}
                  <p class="overview-copy">{charter.description}</p>
                </div>
              </div>
              <div class="context-meta">
                <ContentMetaRow
                  fullName
                  authorName={help.names[project.creator] ?? '…'}
                  authorHref={resolve('/profile/[agent]', { agent: project.creator })}
                  createdAt={project.createdAt}
                />
              </div>
            </div>
            <HelpRolesDock {help} />
          </div>
        {/if}
        {#if chatOpened}
          <div hidden={tab !== 'chat'} inert={tab !== 'chat'}>
            <ProjectChat
              projectId={id}
              seed={project.networkSeed}
              names={help.names}
              fullscreen={compact}
              active={tab === 'chat'}
              highlighted={commentLink}
              onWrote={() => (commentLink = null)}
              copy={{
                title: 'Help request chat',
                placeholder: 'Write a message...',
                empty: 'No help request chat yet.',
              }}
            />
          </div>
        {/if}
      </section>
    {/if}
  </section>
</div>
