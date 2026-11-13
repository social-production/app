<script lang="ts">
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { Board, BoardMember } from '#lib/domain/board.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import PlatformBoardExplainerCard from './PlatformBoardExplainerCard.svelte';
  import PlatformBoardMemberSection from './PlatformBoardMemberSection.svelte';

  import '#lib/components/styles/board.css';

  /**
   * web's PlatformBoardPanel (ADR 0095): active moderators and people seeking the role, yes /
   * no votes on each, Volunteer / Withdraw / Step down, and "How moderator roles work". The
   * board's asset frames (PlatformExecutionFramesPanel) are asset management, not parity.
   * `onVolunteer` and `onVote` throw a readable message when they fail.
   */
  interface Props {
    open: boolean;
    board: Board | null;
    people: Record<string, Person>;
    canVolunteer: boolean;
    onVolunteer: (on: boolean) => Promise<void>;
    onVote: (member: BoardMember, vote: 1 | -1 | 0) => Promise<void>;
    onClose: () => void;
  }

  let { open, board, people, canVolunteer, onVolunteer, onVote, onClose }: Props = $props();

  let volunteering = $state(false);
  let volunteerMessage = $state('');
  let voteMessage = $state('');
  let confirm = $state<'withdraw' | 'step-down' | null>(null);
  let query = $state('');
  let explainerOpen = $state(false);

  $effect(() => {
    if (!open) {
      query = '';
      explainerOpen = false;
    }
  });

  const name = (m: BoardMember) => {
    const p = people[m.agent];
    return p ? displayName(p) : 'Unnamed';
  };
  const needle = $derived(query.trim().toLowerCase());
  const pick = (list: BoardMember[]) =>
    list
      .filter((m) => !needle || name(m).toLowerCase().includes(needle))
      .sort((a, b) => name(a).localeCompare(name(b), undefined, { sensitivity: 'base' }));
  const moderators = $derived(pick(board?.moderators ?? []));
  const candidates = $derived(pick(board?.candidates ?? []));
  const viewerState = $derived(board?.viewerState ?? null);

  const message = (e: unknown, fallback: string) =>
    e instanceof Error && e.message ? e.message : fallback;

  async function vote(m: BoardMember, value: 1 | -1 | 0): Promise<void> {
    voteMessage = '';
    try {
      await onVote(m, value);
    } catch (e) {
      voteMessage = message(e, 'Could not record that standing vote. Try again.');
    }
  }

  async function volunteer(on: boolean, done: string, failed: string): Promise<void> {
    volunteering = true;
    confirm = null;
    volunteerMessage = '';
    try {
      await onVolunteer(on);
      volunteerMessage = done;
    } catch (e) {
      volunteerMessage = message(e, failed);
    }
    volunteering = false;
  }

  function leave(kind: 'withdraw' | 'step-down'): void {
    if (confirm !== kind) {
      confirm = kind;
      return;
    }
    if (kind === 'withdraw') {
      void volunteer(false, 'Your candidacy has been withdrawn.', 'Could not withdraw candidacy.');
    } else {
      void volunteer(false, 'You have stepped down as moderator.', 'Could not step down.');
    }
  }
</script>

{#snippet leaveRow(kind: 'withdraw' | 'step-down', label: string, confirmLabel: string)}
  <div class="volunteer-row">
    <button
      class="secondary-button"
      type="button"
      disabled={volunteering}
      onclick={() => leave(kind)}
    >
      {volunteering ? 'Working…' : confirm === kind ? confirmLabel : label}
    </button>
    {#if confirm === kind && !volunteering}
      <button class="text-button" type="button" onclick={() => (confirm = null)}>Cancel</button>
    {/if}
    {#if volunteerMessage}<p class="volunteer-feedback">{volunteerMessage}</p>{/if}
  </div>
{/snippet}

<OverlaySheet
  {open}
  title="Moderators"
  labelledById="platform-board-title"
  wide
  rootClass="platform-board"
  {onClose}
>
  {#snippet toolbar()}
    <div class="board-toolbar">
      <label class="sr-only" for="board-people-search">Search moderators</label>
      <input
        id="board-people-search"
        bind:value={query}
        placeholder="Search people"
        type="search"
      />
      <button class="rules-toggle" type="button" onclick={() => (explainerOpen = true)}>
        How moderator roles work
      </button>
    </div>
  {/snippet}
  <div class="board-body">
    <div class="people-stack">
      {#if !board}
        <p class="empty-row">Loading moderators…</p>
      {:else if moderators.length === 0 && candidates.length === 0}
        <p class="empty-row">{query.trim() ? 'No matches.' : 'No moderators listed yet.'}</p>
      {:else}
        <PlatformBoardMemberSection
          title="Active moderators"
          members={moderators}
          mode="active"
          sectionIndex={0}
          {people}
          onVote={(m, v) => void vote(m, v)}
        />
        <PlatformBoardMemberSection
          title="Seeking position"
          members={candidates}
          mode="candidate"
          sectionIndex={1}
          {people}
          onVote={(m, v) => void vote(m, v)}
        />
      {/if}
    </div>
    {#if voteMessage}<p class="volunteer-feedback">{voteMessage}</p>{/if}
    {#if viewerState === 'member'}
      {@render leaveRow('step-down', 'Step down as moderator', 'Confirm step down')}
    {:else if viewerState === 'candidate'}
      {@render leaveRow('withdraw', 'Withdraw candidacy', 'Confirm withdraw')}
    {:else if canVolunteer}
      <div class="volunteer-row">
        <button
          class="primary-button"
          type="button"
          disabled={volunteering}
          onclick={() =>
            void volunteer(
              true,
              'You are now seeking a moderator role.',
              'Could not submit volunteer request. You may already be a candidate.',
            )}
        >
          {volunteering ? 'Submitting…' : 'Volunteer as moderator'}
        </button>
        {#if volunteerMessage}<p class="volunteer-feedback">{volunteerMessage}</p>{/if}
      </div>
    {/if}
  </div>
</OverlaySheet>

<OverlaySheet
  open={explainerOpen}
  title="How moderator roles work"
  labelledById="platform-board-explainer-title"
  rootClass="platform-board"
  elevated
  onClose={() => (explainerOpen = false)}
>
  <div class="explainer-sheet-body">
    <PlatformBoardExplainerCard open={true} showToggle={false} />
  </div>
</OverlaySheet>
