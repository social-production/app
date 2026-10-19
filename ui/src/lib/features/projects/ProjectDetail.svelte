<script lang="ts">
  import type { Snippet } from 'svelte';

  import DetailTopTabs from '#lib/components/DetailTopTabs.svelte';
  import EditSheet from '#lib/components/overlay/EditSheet.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import UpdateSheet from '#lib/components/overlay/UpdateSheet.svelte';
  import type { DetailTabId } from '#lib/domain/projects.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';

  import ContextPanel from './ContextPanel.svelte';
  import { DetailState } from './detailState.svelte.ts';
  import ParticipationPanel from './ParticipationPanel.svelte';
  import ProjectChat from './ProjectChat.svelte';
  import ProjectDock from './ProjectDock.svelte';

  import '#lib/components/styles/detail.css';

  /**
   * web's ProjectDetailPage: Details, Participation and Chat tabs, plus Threads (ADR 0041; web
   * keeps threads in channels). The route passes the Threads tab in, as features don't import
   * each other. Links and history arrive in M5.
   */
  let { id, threads }: { id: string; threads?: Snippet } = $props();

  // The route re-creates this page per id ({#key}), so reading `id` once is intended.
  // svelte-ignore state_referenced_locally
  const detail = new DetailState(id);
  let tab = $state<DetailTabId>('context');
  let sheet = $state<'members' | 'edit' | 'update' | null>(null);
  let compact = $state(false);
  // As on web, the chat mounts (and starts polling) the first time its tab opens.
  let chatOpened = $state(false);
  let threadsOpened = $state(false);
  const chatFull = $derived(tab === 'chat' && compact);

  $effect(() => {
    void detail.open();
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
    if (tab === 'threads') threadsOpened = true;
    chatChrome.immersive = chatFull;
    return () => {
      chatChrome.immersive = false;
    };
  });

  async function filed(ok: Promise<boolean>): Promise<void> {
    if (await ok) sheet = null;
  }
</script>

<div class="project-detail">
  <section class="page" class:page-chat={chatFull}>
    {#if detail.error && !detail.project}
      <section class="hero-card"><p class="tab-loading">{detail.error}</p></section>
    {:else if !detail.project?.charter}
      <section class="hero-card"><p class="tab-loading">Loading project…</p></section>
    {:else}
      {@const project = detail.project}
      {@const charter = detail.project.charter}
      <section class="hero-card" class:chat-tab-active={chatFull}>
        <DetailTopTabs
          active={tab}
          available={threads
            ? ['context', 'participation', 'chat', 'threads']
            : ['context', 'participation', 'chat']}
          ariaLabel="Project detail tabs"
          onSelect={(t) => (tab = t)}
        />
        <div
          class="tab-panel context-tab"
          class:tab-panel-hidden={tab !== 'context'}
          hidden={tab !== 'context'}
          inert={tab !== 'context'}
        >
          <ContextPanel
            {project}
            {charter}
            names={detail.names}
            canPropose={detail.canPropose}
            onAddUpdate={() => (sheet = 'update')}
          />
        </div>
        <div
          class="tab-panel participation-tab"
          class:tab-panel-hidden={tab !== 'participation'}
          hidden={tab !== 'participation'}
          inert={tab !== 'participation'}
        >
          <ParticipationPanel {detail} />
        </div>
        {#if chatOpened}
          <div
            class="tab-panel chat-tab"
            class:tab-panel-hidden={tab !== 'chat'}
            hidden={tab !== 'chat'}
            inert={tab !== 'chat'}
          >
            <ProjectChat
              projectId={id}
              names={detail.names}
              fullscreen={compact}
              active={tab === 'chat'}
            />
          </div>
        {/if}
        {#if threads && threadsOpened}
          <div
            class="tab-panel threads-tab"
            class:tab-panel-hidden={tab !== 'threads'}
            hidden={tab !== 'threads'}
            inert={tab !== 'threads'}
          >
            {@render threads()}
          </div>
        {/if}
        {#if tab !== 'chat'}
          <ProjectDock
            {detail}
            membersOpen={sheet === 'members'}
            onOpenMembers={() => (sheet = 'members')}
            onEdit={() => (sheet = 'edit')}
          />
        {/if}
      </section>

      <PeopleSheet
        open={sheet === 'members'}
        title="Project members"
        description="Members coordinate planning, updates, and activity together."
        emptyCopy="No members listed yet."
        people={detail.members}
        onClose={() => (sheet = null)}
      />
      <EditSheet
        open={sheet === 'edit'}
        draft={{
          title: charter.title,
          description: charter.description,
          location: charter.location_label,
        }}
        pending={detail.saving}
        message={detail.sheetMessage}
        onSubmit={(draft) => void filed(detail.proposeEdit(draft))}
        onClose={() => (sheet = null)}
      />
      <UpdateSheet
        open={sheet === 'update'}
        pending={detail.saving}
        message={detail.sheetMessage}
        onSubmit={(body) => void filed(detail.proposeUpdate(body))}
        onClose={() => (sheet = null)}
      />
    {/if}
  </section>
</div>
