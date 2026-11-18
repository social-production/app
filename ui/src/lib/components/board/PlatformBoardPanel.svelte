<script lang="ts">
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { boardOrder, type Board, type BoardMember } from '#lib/domain/board.ts';
  import { displayName, type Person } from '#lib/domain/people.ts';

  import BoardConfirmSheet, { type BoardAsk } from './BoardConfirmSheet.svelte';
  import BoardHowSheet from './BoardHowSheet.svelte';
  import BoardRow from './BoardRow.svelte';

  import '#lib/components/styles/ux/board.css';

  /**
   * The Moderators sheet (Platform and Moderators mockups §2, ADR 0095): one line of what a
   * seat takes this week, Seated and Candidates as rows with the feed's vote control, and the
   * main button at the bottom (Volunteer, Withdraw or Step down, each asked once). "How it
   * works" opens the steps. `onVolunteer` and `onVote` throw a readable message on failure.
   */
  interface Props {
    open: boolean;
    board: Board | null;
    people: Record<string, Person>;
    me: string;
    canVolunteer: boolean;
    /** Why the board couldn't load; the sheet offers to try again instead of loading forever. */
    error?: string | undefined;
    onRetry?: () => void;
    onVolunteer: (on: boolean) => Promise<void>;
    onVote: (member: BoardMember, vote: 1 | -1 | 0) => Promise<void>;
    onClose: () => void;
  }

  let {
    open,
    board,
    people,
    me,
    canVolunteer,
    error,
    onRetry,
    onVolunteer,
    onVote,
    onClose,
  }: Props = $props();

  let ask = $state<BoardAsk | null>(null);
  let working = $state(false);
  let notice = $state('');
  let problem = $state('');
  let howOpen = $state(false);

  $effect(() => {
    if (!open) {
      notice = '';
      problem = '';
      howOpen = false;
    }
  });

  const name = (m: BoardMember) => {
    const p = people[m.agent];
    return p ? displayName(p) : 'Unnamed';
  };
  const quorum = $derived(board?.quorum ?? 0);
  const seated = $derived(boardOrder(board?.moderators ?? [], me, name));
  const candidates = $derived(boardOrder(board?.candidates ?? [], me, name));
  const viewerState = $derived(board?.viewerState ?? null);

  const failed = (e: unknown, fallback: string) =>
    e instanceof Error && e.message ? e.message : fallback;

  async function vote(m: BoardMember, value: 1 | -1 | 0): Promise<void> {
    problem = '';
    try {
      await onVote(m, value);
    } catch (e) {
      problem = failed(e, "Your vote didn't go through. Try again.");
    }
  }

  async function confirm(kind: BoardAsk): Promise<void> {
    working = true;
    problem = '';
    notice = '';
    try {
      await onVolunteer(kind === 'volunteer');
      ask = null;
      if (kind === 'withdraw') notice = 'You withdrew. You can volunteer again any time.';
      else if (kind === 'step-down') notice = 'You stepped down.';
      else if (board?.viewerState === 'member') notice = "You're seated.";
      else notice = "You're a candidate. Members can vote on you now.";
    } catch (e) {
      ask = null;
      problem = failed(e, "That didn't go through. Try again.");
    }
    working = false;
  }
</script>

{#snippet section(title: string, list: BoardMember[])}
  {#if list.length > 0}
    <h3 class="board-head">{title} · {list.length}</h3>
    {#each list as m (m.agent)}
      <BoardRow
        member={m}
        person={people[m.agent]}
        {quorum}
        you={m.agent === me}
        onVote={(member, v) => void vote(member, v)}
      />
    {/each}
  {/if}
{/snippet}

<OverlaySheet
  {open}
  title="Moderators"
  labelledById="platform-board-title"
  rootClass="platform-board"
  {onClose}
>
  {#snippet headerActions()}
    <button class="board-how-link" type="button" onclick={() => (howOpen = true)}
      >How it works</button
    >
  {/snippet}
  <div class="board-body">
    {#if board}
      {#if quorum === 0}
        <p class="board-summary board-note">
          Nobody voted last week, so anyone who volunteers is seated straight away. Once people are
          active, a seat needs their votes.
        </p>
      {:else}
        <p class="board-summary">
          <span><b>{board.moderators.length}</b>seated</span>
          <span
            ><b>{board.candidates.length}</b>{board.candidates.length === 1
              ? 'candidate'
              : 'candidates'}</span
          >
          <span><b>{quorum} votes</b>and 66% yes for a seat</span>
        </p>
      {/if}
      {#if seated.length === 0 && candidates.length === 0}
        <p class="board-empty">Nobody has volunteered yet.</p>
      {/if}
      {@render section('Seated', seated)}
      {@render section('Candidates', candidates)}
    {:else if error}
      <div class="board-empty board-failed" role="alert">
        <p>The moderators couldn’t load. {error}</p>
        {#if onRetry}<button class="board-how-link" type="button" onclick={onRetry}
            >Try again</button
          >{/if}
      </div>
    {:else}
      <SkeletonRows count={3} kind="row" />
    {/if}
    {#if notice}<p class="board-notice" role="status">{notice}</p>{/if}
    {#if problem}<p class="board-problem" role="alert">{problem}</p>{/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions board-actions">
      {#if viewerState === 'member'}
        <button class="sheet-cancel" type="button" onclick={() => (ask = 'step-down')}
          >Step down</button
        >
      {:else if viewerState === 'candidate'}
        <button class="sheet-cancel" type="button" onclick={() => (ask = 'withdraw')}
          >Withdraw</button
        >
      {:else if canVolunteer}
        <button class="sheet-submit" type="button" onclick={() => (ask = 'volunteer')}
          >Volunteer as moderator</button
        >
      {:else}
        <p class="board-hint">Full members can volunteer.</p>
      {/if}
    </div>
  {/snippet}
</OverlaySheet>

<BoardConfirmSheet
  {ask}
  {quorum}
  {working}
  onConfirm={(k) => void confirm(k)}
  onClose={() => (ask = null)}
/>
<BoardHowSheet open={howOpen} onClose={() => (howOpen = false)} />
