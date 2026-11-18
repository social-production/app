<script lang="ts">
  import { plural, t } from '#lib/i18n/index.ts';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import VoteKicker from '#lib/components/governance/VoteKicker.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
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
  const empty = $derived(seated.length === 0 && candidates.length === 0);
  // One plain line under the title says where the board stands and what a seat takes.
  const lede = $derived(
    empty
      ? t('Moderators look after reports across the whole network.')
      : [
          t('{n} seated', { n: seated.length }),
          ...(candidates.length
            ? [plural(candidates.length, '{n} candidate', '{n} candidates')]
            : []),
          quorum === 0
            ? t('open seats are filled straight away while nobody votes')
            : t('a seat needs {n} votes and 66% yes', { n: quorum }),
        ]
          .join(' · ')
          .replace(/^./, (c) => c.toUpperCase()) + '.',
  );
  // An empty board says it once, like an empty feed: what's missing and how a seat is won.
  const emptyText = $derived(
    quorum === 0
      ? canVolunteer
        ? t('Volunteer and you’re seated straight away.')
        : t('Full members can volunteer. The first ones are seated straight away.')
      : canVolunteer
        ? t('Volunteer, and a seat needs votes from {n} people and 66% yes.', { n: quorum })
        : t('Full members can volunteer. A seat needs votes from {n} people and 66% yes.', {
            n: quorum,
          }),
  );

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
      if (kind === 'withdraw') notice = t('You withdrew. You can volunteer again any time.');
      else if (kind === 'step-down') notice = t('You stepped down.');
      else if (board?.viewerState === 'member') notice = t("You're seated.");
      else notice = t("You're a candidate. Members can vote on you now.");
    } catch (e) {
      ask = null;
      problem = failed(e, "That didn't go through. Try again.");
    }
    working = false;
  }
</script>

{#snippet section(title: string, list: BoardMember[])}
  {#if list.length > 0}
    <h3 class="board-head">{title}<span>{list.length}</span></h3>
    <div class="board-card">
      {#each list as m (m.agent)}
        <BoardRow
          member={m}
          person={people[m.agent]}
          {quorum}
          you={m.agent === me}
          onVote={(member, v) => void vote(member, v)}
        />
      {/each}
    </div>
  {/if}
{/snippet}

<OverlaySheet
  {open}
  title={t('Moderators')}
  labelledById="platform-board-title"
  rootClass="platform-board"
  {onClose}
>
  {#snippet kicker()}<VoteKicker label={t('Platform')} meta={t('Whole network')} />{/snippet}
  {#snippet headerActions()}
    <button class="board-how-link" type="button" onclick={() => (howOpen = true)}
      ><ToolbarIcon name="info" />{t('How it works')}</button
    >
  {/snippet}
  <div class="board-body">
    {#if board}<p class="board-lede">{lede}</p>{/if}
    {#if board && empty}
      <div class="board-card">
        <EmptyState icon="shield" title={t('No moderators yet')} text={emptyText} />
      </div>
    {:else if board}
      {@render section(t('Seated'), seated)}
      {@render section(t('Candidates'), candidates)}
      {#if quorum === 0}
        <p class="board-note">
          <ToolbarIcon name="clock" />{t(
            'Seats are checked every week. Once people vote, a seat needs their votes and 66% yes.',
          )}
        </p>
      {/if}
    {:else if error}
      <div class="board-empty board-failed" role="alert">
        <p>{t('The moderators couldn’t load.')} {error}</p>
        {#if onRetry}<button class="board-how-link" type="button" onclick={onRetry}
            >{t('Try again')}</button
          >{/if}
      </div>
    {:else}
      <SkeletonRows count={3} kind="row" />
    {/if}
    {#if board && !empty && !canVolunteer && !viewerState}
      <p class="board-empty">{t('Full members can volunteer.')}</p>
    {/if}
    {#if notice}<p class="board-notice" role="status">{notice}</p>{/if}
    {#if problem}<p class="board-problem" role="alert">{problem}</p>{/if}
  </div>
  {#snippet footer()}
    {#if viewerState || canVolunteer}<div class="sheet-actions board-actions">
        {#if viewerState === 'member'}
          <button class="sheet-cancel" type="button" onclick={() => (ask = 'step-down')}
            >{t('Step down')}</button
          >
        {:else if viewerState === 'candidate'}
          <button class="sheet-cancel" type="button" onclick={() => (ask = 'withdraw')}
            >{t('Withdraw')}</button
          >
        {:else if canVolunteer}
          <button
            class="sheet-submit"
            type="button"
            aria-label={t('Volunteer as moderator')}
            onclick={() => (ask = 'volunteer')}>{t('Volunteer')}</button
          >
        {/if}
      </div>{/if}
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
