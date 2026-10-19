<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import SignalStrip from '#lib/components/cards/SignalStrip.svelte';
  import DetailActionDock from '#lib/components/DetailActionDock.svelte';
  import DetailTopTabs from '#lib/components/DetailTopTabs.svelte';
  import MembershipSplitButton from '#lib/components/MembershipSplitButton.svelte';
  import EditSheet from '#lib/components/overlay/EditSheet.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import { takesSignals } from '#lib/domain/projects.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { resolve } from '$app/paths';

  import { DetailState } from './detailState.svelte.ts';

  import '#lib/components/styles/detail.css';

  /** web's ProjectDetailPage, context tab (participation, chat, links and history come with M3–M5). */
  let { id }: { id: string } = $props();

  // The route re-creates this page per id ({#key}), so reading `id` once is intended.
  // svelte-ignore state_referenced_locally
  const detail = new DetailState(id, () =>
    connection.current.status === 'connected' ? connection.current.agent : '',
  );
  let membersOpen = $state(false);
  let editOpen = $state(false);

  $effect(() => {
    void detail.load();
  });
</script>

<div class="project-detail">
  <section class="page">
    {#if detail.error && !detail.project}
      <section class="hero-card"><p class="tab-loading">{detail.error}</p></section>
    {:else if !detail.project?.charter}
      <section class="hero-card"><p class="tab-loading">Loading project…</p></section>
    {:else}
      {@const project = detail.project}
      {@const charter = project.charter}
      <section class="hero-card">
        <DetailTopTabs
          active="context"
          available={['context']}
          ariaLabel="Project detail tabs"
          onSelect={() => {}}
        />
        <div class="tab-panel context-tab">
          <div class="project-overview">
            <div class="context-panel">
              <FeedCardTop tags={charter?.tags ?? []} stage={charter?.phase ?? ''} />
              <div class="heading overview-heading">
                <div class="identity-row">
                  <div class="identity-copy">
                    <h1>{charter?.title}</h1>
                    {#if charter?.location_label}<p class="live-fact">
                        {charter.location_label}
                      </p>{/if}
                    <p class="overview-copy">{charter?.description}</p>
                  </div>
                </div>
              </div>
              <div class="context-meta">
                <ContentMetaRow
                  fullName
                  authorName={detail.creator?.profile?.name ?? '…'}
                  authorHref={resolve('/profile/[agent]', { agent: project.creator })}
                  createdAt={project.createdAt}
                />
              </div>
            </div>
          </div>
          <DetailActionDock active>
            <div class="project-overview">
              <div class="context-dock">
                {#if charter && takesSignals(charter.mode)}
                  <div id="participation-signals" class="signal-row">
                    <SignalStrip
                      labeled
                      support={project.demand}
                      oppose={project.opposition}
                      mine={project.mySignal}
                      onSignal={(next) => detail.signal(next)}
                    />
                  </div>
                {/if}
                <div class="control-actions">
                  <MembershipSplitButton
                    joined={project.amMember}
                    count={project.members.length}
                    pending={detail.joining}
                    {membersOpen}
                    onToggleJoin={() => void detail.toggleJoin()}
                    onOpenMembers={() => (membersOpen = true)}
                  />
                  {#if detail.isCreator}
                    <button
                      type="button"
                      class="quiet-control"
                      class:open={editOpen}
                      onclick={() => (editOpen = true)}>Edit</button
                    >
                  {/if}
                </div>
              </div>
            </div>
          </DetailActionDock>
        </div>
      </section>

      <PeopleSheet
        open={membersOpen}
        title="Project members"
        description="Members coordinate planning, updates, and activity together."
        emptyCopy="No members listed yet."
        people={detail.members}
        onClose={() => (membersOpen = false)}
      />
      <EditSheet
        open={editOpen}
        draft={{
          title: charter?.title ?? '',
          description: charter?.description ?? '',
          location: charter?.location_label ?? '',
        }}
        pending={detail.saving}
        message={detail.editMessage}
        onSubmit={async (draft) => {
          if (await detail.edit(draft)) editOpen = false;
        }}
        onClose={() => (editOpen = false)}
      />
    {/if}
  </section>
</div>
