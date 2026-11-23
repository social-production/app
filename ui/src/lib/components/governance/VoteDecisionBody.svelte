<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import { resultSentence } from '#lib/domain/decided.ts';
  import type { Proposal } from '#lib/domain/governance.ts';
  import { isNetworkVote } from '#lib/domain/board.ts';
  import type { Charter } from '#lib/domain/projects.ts';
  import { settingsSummary } from '#lib/domain/serviceRequests.ts';
  import { phaseMove } from '#lib/domain/voteSheet.ts';

  import VoteTally from './VoteTally.svelte';

  /**
   * The one vote sheet's body (owner, 2026-11-18): the proposal itself, unboxed, then one
   * count. The kind line and headline sit in the sheet's header (VoteSheetHead). An open vote
   * needs no "If this passes…": the headline says it. A decided one says what it did.
   */
  interface Props {
    proposal: Proposal;
    charter: Charter;
    stage: string;
    phaseLabel: (id: string) => string;
    /** The viewer isn't a Full member yet: say their vote won't decide this. */
    newVoter?: boolean;
  }

  let { proposal, charter, stage, phaseLabel, newVoter = false }: Props = $props();

  const kind = $derived(proposal.kind);
  const move = $derived(phaseMove(kind, stage, phaseLabel));
  const why = $derived(
    'Return' in kind
      ? kind.Return.reason
      : 'Convert' in kind
        ? kind.Convert.reason
        : 'Phase' in kind
          ? kind.Phase.reason
          : 'Software' in kind
            ? kind.Software.reason
            : 'RequestSettings' in kind
              ? kind.RequestSettings.reason
              : '',
  );
</script>

<div class="vote-sheet">
  {#if proposal.status !== 'Open'}
    <p class="outcome">{resultSentence(proposal, stage, phaseLabel)}</p>
  {/if}
  {#if newVoter && proposal.status === 'Open'}
    <p class="network-vote-line">
      {t(
        'Only Full members vote. You get Full standing from two vouches by Full members, or after steady activity.',
      )}
    </p>
  {/if}
  {#if isNetworkVote(charter.tags)}
    <p class="network-vote-line">{t('Network vote: any Full member can vote')}</p>
  {/if}

  {#if move}
    <p class="vote-move">
      <s>{t(move.from)}</s>
      <svg aria-hidden="true" viewBox="0 0 24 24"
        ><path
          d="M5 12h14M13 6l6 6-6 6"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
      <span>{move.to}</span>
    </p>
  {/if}
  {#if 'Update' in kind}
    <p class="vote-text">{kind.Update.body}</p>
  {:else if 'Software' in kind && kind.Software.description}
    <p class="vote-text">{kind.Software.description}</p>
  {:else if 'RequestSettings' in kind}
    <p class="vote-text">
      {settingsSummary({
        enabled: kind.RequestSettings.enabled,
        mode: kind.RequestSettings.mode,
        offSchedule: kind.RequestSettings.off_schedule,
      })}
    </p>
  {:else if 'Edit' in kind}
    {@const e = kind.Edit}
    {#each [{ name: t('Title'), now: charter.title, next: e.title }, { name: t('Description'), now: charter.description, next: e.description }, { name: t('Location'), now: '', next: e.location }] as field (field.name)}
      {#if field.next !== null}
        <div class="vote-field">
          <p class="vote-field-name">{field.name}</p>
          {#if field.now}<p class="vote-was"><s>{field.now}</s></p>{/if}
          <p class="vote-now">{field.next || '—'}</p>
        </div>
      {/if}
    {/each}
  {/if}
  {#if why}
    <blockquote class="vote-why"><small>{t('Why')}</small>{why}</blockquote>
  {/if}

  <VoteTally
    yes={proposal.yes}
    no={proposal.no}
    population={proposal.population}
    failed={proposal.status === 'Rejected'}
  >
    <!-- App-only (ADR 0040): challenged counts and members still left out. -->
    {#if proposal.added > 0}
      <p class="vote-tally-note">
        Reopened: a challenge added {plural(proposal.added, '{n} member', '{n} members')} the first count
        left out.
      </p>
    {/if}
    {#if proposal.canChallenge}
      <p class="vote-tally-note">
        {proposal.missing}
        {proposal.missing === 1 ? t('member who joined') : t('members who joined')} before this was proposed
        {proposal.missing === 1 ? "isn't" : "aren't"} counted. Challenge the count in the Participation
        tab.
      </p>
    {/if}
  </VoteTally>
</div>
