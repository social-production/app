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
   * A channel's, community's or the Platform's header (Remaining Screens, 3), the same shape
   * as a profile: a soft band, the mark (a rounded square), the name large, "Channel · N
   * members", the description, then pill buttons with words and a round settings button.
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
  // The Platform counts people active last week (what its votes are sized from), not members
  // (Platform and Moderators mockups §1).
  const sub = $derived.by(() => {
    if (!s.platform) return `${typeName} · ${count} ${count === 1 ? 'member' : 'members'}`;
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
          {#if scope.isPrivate}
            <span class="private-mark" title="Private: members only" aria-label="Private">
              <svg aria-hidden="true" viewBox="0 0 24 24" width="14" height="14">
                <path
                  d="M7 11V8a5 5 0 0 1 10 0v3M6 11h12v9H6z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                  stroke-linejoin="round"
                />
              </svg>
            </span>
          {/if}
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
            <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
              <path
                d="M10.3 2h3.4l.5 2.4c.5.2 1 .4 1.5.6l2.1-1.2 2.4 2.4-1.2 2.1c.2.5.4 1 .6 1.5L22 10.3v3.4l-2.4.5c-.2.5-.4 1-.6 1.5l1.2 2.1-2.4 2.4-2.1-1.2c-.5.2-1 .4-1.5.6L13.7 22h-3.4l-.5-2.4c-.5-.2-1-.4-1.5-.6l-2.1 1.2-2.4-2.4 1.2-2.1c-.2-.5-.4-1-.6-1.5L2 13.7v-3.4l2.4-.5c.2-.5.4-1 .6-1.5L3.8 6.2l2.4-2.4 2.1 1.2c.5-.2 1-.4 1.5-.6L10.3 2Zm1.7 6.2A3.8 3.8 0 1 0 12 15.8 3.8 3.8 0 0 0 12 8.2Z"
                fill="currentColor"
              ></path>
            </svg>
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
