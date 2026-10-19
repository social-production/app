<script lang="ts">
  import DetailTopTabs from '#lib/components/DetailTopTabs.svelte';
  import EditSheet from '#lib/components/overlay/EditSheet.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import UpdateSheet from '#lib/components/overlay/UpdateSheet.svelte';
  import type { DetailTabId } from '#lib/domain/projects.ts';

  import ContextPanel from './ContextPanel.svelte';
  import { DetailState } from './detailState.svelte.ts';
  import ParticipationPanel from './ParticipationPanel.svelte';
  import ProjectDock from './ProjectDock.svelte';

  import '#lib/components/styles/detail.css';

  /** web's ProjectDetailPage: Details and Participation tabs (chat, links, history: M4–M5). */
  let { id }: { id: string } = $props();

  // The route re-creates this page per id ({#key}), so reading `id` once is intended.
  // svelte-ignore state_referenced_locally
  const detail = new DetailState(id);
  let tab = $state<DetailTabId>('context');
  let sheet = $state<'members' | 'edit' | 'update' | null>(null);

  $effect(() => {
    void detail.open();
  });

  async function filed(ok: Promise<boolean>): Promise<void> {
    if (await ok) sheet = null;
  }
</script>

<div class="project-detail">
  <section class="page">
    {#if detail.error && !detail.project}
      <section class="hero-card"><p class="tab-loading">{detail.error}</p></section>
    {:else if !detail.project?.charter}
      <section class="hero-card"><p class="tab-loading">Loading project…</p></section>
    {:else}
      {@const project = detail.project}
      {@const charter = detail.project.charter}
      <section class="hero-card">
        <DetailTopTabs
          active={tab}
          available={['context', 'participation']}
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
        <ProjectDock
          {detail}
          membersOpen={sheet === 'members'}
          onOpenMembers={() => (sheet = 'members')}
          onEdit={() => (sheet = 'edit')}
        />
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
