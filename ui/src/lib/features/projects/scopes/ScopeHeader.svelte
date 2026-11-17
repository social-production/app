<script lang="ts">
  import type { Snippet } from 'svelte';

  import IconPopoverMenu from '#lib/components/IconPopoverMenu.svelte';
  import NetworkVoteSheet from '#lib/components/board/NetworkVoteSheet.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { Board } from '#lib/domain/board.ts';
  import type { Person } from '#lib/domain/people.ts';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import type { ScopeState } from './scopeState.svelte.ts';

  /**
   * A channel's, community's or the Platform's header (Remaining Screens, 3; Channel Pictures
   * mockups §1), the same shape as a profile: a soft band, the mark (a rounded square), the
   * name large with a chip (Channel, Community or Private), "N members" and who decides in an
   * organizer-run community, the description, then pill buttons with words and the pencil.
   */
  interface Props {
    s: ScopeState;
    kind: ScopeKind;
    /** Decisions waiting for this viewer's vote. */
    waiting: number;
    invitePanel: Snippet;
    membersOpen: boolean;
    inviteOpen: boolean;
    settingsOpen: boolean;
    boardOpen: boolean;
    /** The Platform's moderator board: its faces and the weekly active count (ADR 0095). */
    board?: Board | null;
    people?: Record<string, Person>;
    onVotes: () => void;
  }

  let {
    s,
    kind,
    waiting,
    invitePanel,
    membersOpen = $bindable(),
    inviteOpen = $bindable(),
    settingsOpen = $bindable(),
    boardOpen = $bindable(),
    board = null,
    people = {},
    onVotes,
  }: Props = $props();

  const scope = $derived(s.scope);
  const joined = $derived(!!s.project?.amMember);
  const count = $derived(s.project?.members.length ?? scope?.members ?? 0);
  const pending = $derived(s.joining || (!s.project && !!scope?.id));
  const typeName = $derived(kind === 'channel' ? 'Channel' : 'Community');
  const organized = $derived(s.project?.governance === 'organizer_controlled');
  // The Platform counts people active last week (what its votes are sized from), not members
  // (Platform and Moderators mockups §1).
  const sub = $derived.by(() => {
    if (!s.platform) {
      const members = `${count} ${count === 1 ? 'member' : 'members'}`;
      return organized ? `${members} · organizers decide` : members;
    }
    const n = board?.weeklyActive;
    if (n === undefined) return 'The whole network';
    return n === 0 ? 'Nobody active last week' : `${n.toLocaleString()} active last week`;
  });
  let networkOpen = $state(false);
  const faces = $derived((board?.moderators ?? []).slice(0, 3));
  const more = $derived(Math.max(0, (board?.moderators.length ?? 0) - 3));
</script>

{#if scope}
  <section class="scope-header scope-hero">
    <div class="scope-cover" aria-hidden="true"></div>
    <div class="scope-hero-body">
      <span class="scope-avatar" aria-hidden="true">
        {#if s.platform}<ToolbarIcon
            name="globe"
          />{:else if kind === 'channel'}#{:else}<SurfaceIcon icon={kind} size="md" />{/if}
      </span>
      <div class="scope-hero-title">
        <h1>
          {s.platform ? 'Platform' : scope.handle}
          {#if s.platform}<span class="scope-kind-chip">Whole network</span>{/if}
          {#if !s.platform}<span class="scope-kind-chip"
              >{scope.isPrivate ? 'Private' : typeName}</span
            >{/if}
        </h1>
        <p class="scope-hero-sub">{sub}</p>
      </div>
      <div class="scope-intro-prose">
        <p>{s.project?.charter?.description ?? scope.description}</p>
      </div>
      <div class="action-row">
        {#if scope.isPrivate && !joined}
          <IconPopoverMenu
            open={inviteOpen}
            active={inviteOpen}
            chip
            ariaLabel={`Join ${scope.handle} with invite, ${count} members`}
            menuLabel="Join with invite"
            minWidth={300}
            onOpenChange={(o) => (inviteOpen = o)}
          >
            {#snippet icon()}
              <ToolbarIcon name="person-plus" />
              <span>Join with invite</span>
            {/snippet}
            {@render invitePanel()}
          </IconPopoverMenu>
        {:else}
          <button
            id="participation-join"
            aria-label={joined ? `Leave ${scope.handle}` : `Join ${scope.handle}`}
            aria-pressed={joined}
            class="scope-pill membership-join"
            class:joined
            class:primary={!joined}
            disabled={pending}
            type="button"
            onclick={() => void s.toggleJoin()}>{joined ? 'Joined' : 'Join'}</button
          >
        {/if}
        {#if !s.platform}<button
            aria-expanded={membersOpen}
            aria-label={`${count} members`}
            class="scope-pill membership-count"
            type="button"
            onclick={() => (membersOpen = true)}>Members</button
          >{/if}
        {#if scope.isPrivate && joined}
          <IconPopoverMenu
            open={inviteOpen}
            active={inviteOpen}
            ariaLabel={`Invite people to ${scope.handle}`}
            menuLabel="Invite people"
            minWidth={300}
            onOpenChange={(o) => (inviteOpen = o)}
          >
            {#snippet icon()}<ToolbarIcon name="person-plus" />{/snippet}
            {@render invitePanel()}
          </IconPopoverMenu>
        {/if}
        {#if !s.platform}<span class="scope-action-gap"></span>{/if}
        {#if s.platform}
          <button
            aria-expanded={boardOpen}
            aria-label="Moderators"
            class="scope-pill board-pill"
            type="button"
            onclick={() => (boardOpen = !boardOpen)}
          >
            {#if faces.length > 0}
              <span class="board-faces" aria-hidden="true">
                {#each faces as m (m.agent)}
                  <AvatarBadge
                    size="sm"
                    name={people[m.agent]?.profile?.name ?? ''}
                    avatar={people[m.agent]?.profile?.avatar ?? null}
                  />
                {/each}
                {#if more > 0}<span class="board-more">+{more}</span>{/if}
              </span>
            {:else}
              <ToolbarIcon name="shield" />
            {/if}
            Moderators
          </button>
        {:else}
          <button
            aria-label="Settings"
            title="Settings"
            class="icon-action"
            class:menu-active={settingsOpen}
            type="button"
            onclick={() => (settingsOpen = true)}
          >
            <ToolbarIcon name="pencil" />
          </button>
        {/if}
      </div>
      {#if s.platform}
        <button class="scope-rule" type="button" onclick={() => (networkOpen = true)}>
          <ToolbarIcon name="people" />
          <span>Any Full member can vote on #platform projects</span>
          <b>How</b>
        </button>
      {/if}
    </div>
    {#if waiting > 0}
      <!-- Sits just left of the + button while a decision is waiting for this viewer. -->
      <button
        class="vote-fab"
        type="button"
        aria-label={`Vote: ${waiting} waiting for your vote`}
        onclick={onVotes}
      >
        <svg class="vote-fab-arrow" aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M12 19V5M6 11l6-6 6 6" /></svg
        >
        Vote
        <span class="vote-fab-count">{waiting}</span>
      </button>
    {/if}
  </section>
{/if}

{#if s.platform}
  <NetworkVoteSheet
    open={networkOpen}
    weeklyActive={board?.weeklyActive ?? null}
    onClose={() => (networkOpen = false)}
  />
{/if}
