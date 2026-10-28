<script lang="ts">
  import DetailTopTabs from '#lib/components/DetailTopTabs.svelte';
  import ConfirmDialog from '#lib/components/overlay/ConfirmDialog.svelte';
  import EditSheet from '#lib/components/overlay/EditSheet.svelte';
  import PeopleSheet from '#lib/components/overlay/PeopleSheet.svelte';
  import UpdateSheet from '#lib/components/overlay/UpdateSheet.svelte';
  import { isEvent, isPrivateSeed, type DetailTabId, type Project } from '#lib/domain/projects.ts';
  import { chatChrome } from '#lib/stores/chatChrome.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import ContextPanel from './ContextPanel.svelte';
  import { DetailState } from './detailState.svelte.ts';
  import EventMembersSheet from './EventMembersSheet.svelte';
  import LinksTab from './LinksTab.svelte';
  import ParticipationDock from './ParticipationDock.svelte';
  import ParticipationPanel from './ParticipationPanel.svelte';
  import ProjectChat from './ProjectChat.svelte';
  import ProjectDock from './ProjectDock.svelte';

  import '#lib/components/styles/detail.css';

  /**
   * web's ProjectDetailPage: Details, Participation, Chat and Links tabs (history: M5). Events
   * are project clones (ADR 0057), so web's EventDetailPage is this page with its event copy.
   */
  let { id }: { id: string } = $props();

  // The route re-creates this page per id ({#key}), so reading `id` once is intended.
  // svelte-ignore state_referenced_locally
  const detail = new DetailState(id);
  // web's links into a project: `?tab=…`, `?comment=…` (the chat), `?open=vote&voteTarget=…`,
  // `?activity=…` (the Take part tab, ADR 0056) and `?linkRequest=…` (the Links tab).
  const params = page.url.searchParams;
  const voteLink = params.get('open') === 'vote' ? params.get('voteTarget') : null;
  // web votes on edits and updates in the Details tab's dock, on the rest in Take part's;
  // a plan's vote opens that plan's sheet.
  const detailsVote = ['edit', 'update'].includes(params.get('voteKind') ?? '');
  const planVote = params.get('voteKind') === 'plan' ? voteLink : null;
  const linkRequest = params.get('linkRequest');
  let commentLink = $state(params.get('comment'));
  let tab = $state<DetailTabId>(
    params.get('comment')
      ? 'chat'
      : voteLink && detailsVote
        ? 'context'
        : voteLink || params.get('activity')
          ? 'participation'
          : linkRequest
            ? 'links'
            : tabFromParam(params.get('tab')),
  );
  let sheet = $state<'members' | 'edit' | 'update' | null>(null);
  let compact = $state(false);
  // As on web, the chat mounts (and starts polling) the first time its tab opens.
  let chatOpened = $state(false);
  let linksOpened = $state(false);
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
    if (tab === 'links') linksOpened = true;
    chatChrome.immersive = chatFull;
    return () => {
      chatChrome.immersive = false;
    };
  });

  function tabFromParam(value: string | null): DetailTabId {
    if (value === 'participation' || value === 'chat' || value === 'links') return value;
    return 'context';
  }

  /** web's clearVoteLink: a vote opened from a link drops the link once its sheet closes. */
  function clearVoteLink(): void {
    dropParams(['open', 'voteKind', 'voteTarget', 'assess']);
  }

  /** web's clearHighlightedCommentTarget: writing in the chat drops the linked message. */
  function clearCommentLink(): void {
    if (!commentLink) return;
    commentLink = null;
    dropParams(['comment']);
  }

  function dropParams(keys: string[]): void {
    const url = new URL(window.location.href);
    for (const key of keys) url.searchParams.delete(key);
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the current page's own path
    void goto(`${url.pathname}${url.search}${url.hash}`, { replace: true, reset: false });
  }

  async function filed(ok: Promise<boolean>): Promise<void> {
    if (await ok) sheet = null;
  }

  /** web's sheet title and button: organizer-run events save; personal services save details. */
  function editLabel(p: Project, submit: boolean): string {
    if (p.governance === 'organizer_controlled') return submit ? 'Save' : 'Edit event';
    return p.charter?.mode === 'personal-service' ? 'Save details' : 'Propose Edit';
  }

  /** web: after leaving a private event it can't be opened again, so go home. */
  async function leave(ok: boolean): Promise<void> {
    detail.confirmingLeave = false;
    if (!ok) return;
    await detail.toggleJoin();
    if (isPrivateSeed(id) && !detail.project?.amMember) await goto(resolve('/'));
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
      {@const event = isEvent(charter.mode)}
      <section class="hero-card" class:chat-tab-active={chatFull}>
        <DetailTopTabs
          active={tab}
          available={['context', 'participation', 'chat', 'links']}
          ariaLabel={event ? 'Event detail tabs' : 'Project detail tabs'}
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
            plans={detail.plans}
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
          <ParticipationDock
            {detail}
            active={tab === 'participation'}
            membersOpen={sheet === 'members'}
            onOpenMembers={() => (sheet = 'members')}
            revealVote={detailsVote || planVote ? null : voteLink}
            onRevealDone={clearVoteLink}
          />
          <ParticipationPanel {detail} revealPlan={planVote} onRevealDone={clearVoteLink} />
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
              seed={project.networkSeed}
              names={detail.names}
              fullscreen={compact}
              active={tab === 'chat'}
              highlighted={commentLink}
              onWrote={clearCommentLink}
            />
          </div>
        {/if}
        {#if linksOpened}
          <div
            class="tab-panel"
            class:tab-panel-hidden={tab !== 'links'}
            hidden={tab !== 'links'}
            inert={tab !== 'links'}
          >
            <LinksTab {detail} highlight={linkRequest} />
          </div>
        {/if}
        {#if tab === 'context'}
          <ProjectDock
            {detail}
            membersOpen={sheet === 'members'}
            onOpenMembers={() => (sheet = 'members')}
            onEdit={() => (sheet = 'edit')}
            revealVote={detailsVote ? voteLink : null}
            onRevealDone={clearVoteLink}
          />
        {/if}
      </section>

      {#if event}
        <EventMembersSheet {detail} open={sheet === 'members'} onClose={() => (sheet = null)} />
      {:else}
        <PeopleSheet
          open={sheet === 'members'}
          title="Project members"
          description="Members coordinate planning, updates, and activity together."
          emptyCopy="No members listed yet."
          people={detail.members}
          onClose={() => (sheet = null)}
        />
      {/if}
      <EditSheet
        open={sheet === 'edit'}
        sheetTitle={editLabel(project, false)}
        submitLabel={editLabel(project, true)}
        appliesDirectly={project.governance === 'organizer_controlled'}
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
      <ConfirmDialog
        open={detail.confirmingLeave}
        title={`Leave ${charter.title}?`}
        body={isPrivateSeed(id)
          ? "It's private, so you'll need a new invite to come back."
          : 'You can join again any time.'}
        confirmLabel="Leave"
        danger
        onSettle={(ok) => void leave(ok)}
      />
    {/if}
  </section>
</div>
