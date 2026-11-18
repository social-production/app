<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import VoteChoiceBar from '#lib/components/governance/VoteChoiceBar.svelte';
  import VoteKicker from '#lib/components/governance/VoteKicker.svelte';
  import VoteTally from '#lib/components/governance/VoteTally.svelte';
  import { relativeTime } from '#lib/domain/time.ts';
  import {
    canVote,
    externalUrl,
    stageFilled,
    type SoftwarePullRequest,
  } from '#lib/domain/software.ts';

  /**
   * A pull request's three steps as the one vote sheet (owner, 2026-11-18): approve it,
   * record its merge, confirm the merge. The step sits in the kind line; the pull request or
   * its commit is one link row; the vote or "Send to confirm" is the dock.
   */
  interface Props {
    pr: SoftwarePullRequest;
    mode: 'vote-pr' | 'record-merge';
    names: Record<string, string>;
    busy: boolean;
    canSubmit: boolean;
    onVote: (choice: 'Yes' | 'No') => void;
    onSubmit: () => void;
    onClose: () => void;
    /** The merge ID and link fields (record-merge). */
    children?: Snippet;
  }

  let { pr, mode, names, busy, canSubmit, onVote, onSubmit, onClose, children }: Props = $props();

  const step = $derived(stageFilled(pr.stage));
  const confirming = $derived(pr.stage === 'confirmation');
  const title = $derived(
    mode === 'record-merge'
      ? t('Record the merge')
      : confirming
        ? t('Was {title} merged?', { title: pr.title })
        : pr.title,
  );
  const who = $derived(
    confirming && pr.merge
      ? t('Recorded by {name}', { name: names[pr.merge.by] || 'unknown' })
      : `${names[pr.author] || 'unknown'} · ${relativeTime(pr.createdAt)}`,
  );
  const mine = $derived(pr.tally.myVote === 'Neutral' ? null : pr.tally.myVote);
  const host = (url: string): string => {
    try {
      return new URL(externalUrl(url)).host;
    } catch {
      return '';
    }
  };
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- the pull request's links are off the app -->
{#snippet link(href: string, label: string)}
  <a class="vote-link" href={externalUrl(href)} rel="noreferrer" target="_blank">
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><path
        d="M14 4h6v6M20 4l-9 9M18 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h5"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
      /></svg
    >
    <span>{label}</span>
    <small>{host(href)}</small>
  </a>
{/snippet}

<OverlaySheet
  open={true}
  {title}
  labelledById="pull-request-sheet"
  rootClass="vote-dock-root"
  {onClose}
>
  {#snippet kicker()}
    <VoteKicker
      label={confirming ? t('Merge') : t('Pull request')}
      status={t('step {n} of 3', { n: step })}
      meta={who}
    />
  {/snippet}
  <div class="vote-list">
    <div class="vote-sheet">
      {#if confirming && pr.merge}
        {@render link(pr.merge.mergeUrl, t('Commit {id}', { id: pr.merge.mergeId }))}
        <p class="outcome">{t('Check the commit, then confirm.')}</p>
      {:else}
        {#if mode === 'record-merge'}<p class="vote-text vote-pr-title">{pr.title}</p>{/if}
        {#if pr.summary && mode === 'vote-pr'}<p class="vote-text">{pr.summary}</p>{/if}
        {@render link(pr.pullRequestUrl, `${pr.pullRequestId}`)}
      {/if}
      {#if mode === 'vote-pr'}
        <VoteTally yes={pr.tally.yes} no={pr.tally.no} population={pr.tally.population} />
      {:else}
        {@render children?.()}
      {/if}
    </div>
  </div>
  {#snippet footer()}
    {#if mode === 'vote-pr'}
      {#if canVote(pr) || mine}
        <VoteChoiceBar docked {mine} onChoose={(c) => c !== 'Neutral' && onVote(c)} />
      {/if}
    {:else}
      <div class="sheet-actions">
        <button class="sheet-submit" type="button" disabled={busy || !canSubmit} onclick={onSubmit}
          >{t('Send to confirm')}</button
        >
      </div>
    {/if}
  {/snippet}
</OverlaySheet>
<!-- eslint-enable svelte/no-navigation-without-resolve -->
