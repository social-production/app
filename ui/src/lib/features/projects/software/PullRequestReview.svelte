<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import VoteChoiceBar from '#lib/components/governance/VoteChoiceBar.svelte';
  import {
    canVote,
    stageFilled,
    externalUrl,
    voteRequirementLine,
    voteSummaryLine,
    type SoftwarePullRequest,
  } from '#lib/domain/software.ts';

  /**
   * web's SoftwareGovernanceWizard `vote-pr` and `record-merge` steps, as drawn (Invites,
   * Help Requests and Software Screens, 4): the three-step line, the pull request's details
   * as a soft list, then its vote (or, for a merger, the merge ID and link fields).
   */
  interface Props {
    pr: SoftwarePullRequest;
    mode: 'vote-pr' | 'record-merge';
    author: string;
    onVote: (choice: 'Yes' | 'No') => void;
    children?: import('svelte').Snippet;
  }

  let { pr, mode, author, onVote, children }: Props = $props();
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- the pull request's links are off the app -->
<div class="pr-review-steps" aria-label={`Step ${stageFilled(pr.stage)} of 3`}>
  {#each [1, 2, 3] as step (step)}
    <i class:done={step < stageFilled(pr.stage)} class:now={step === stageFilled(pr.stage)}></i>
  {/each}
</div>
<p class="pr-review-step">
  Step {stageFilled(pr.stage)} of 3 ·
  <b>{['Approval', 'Merge', 'Confirmation'][stageFilled(pr.stage) - 1]}</b>, then {stageFilled(
    pr.stage,
  ) === 1
    ? t('a merge is recorded, then members confirm it')
    : stageFilled(pr.stage) === 2
      ? t('members confirm it')
      : t('it is done')}
</p>
{#if mode === 'vote-pr'}
  <h2>{pr.title}</h2>
  <p class="helper-copy">
    {#if pr.stage === 'confirmation'}
      {t('Confirm that the recorded merge actually happened and matches this pull request.')}
    {:else}
      {t('Approve or reject this pull request before anyone records a merge.')}
    {/if}
  </p>
{:else}
  <h2>Record merge for {pr.title}</h2>
  <p class="helper-copy">
    {t(
      'This pull request already passed approval. Enter the merge ID and a link others can open to verify it. After you submit, members will still need to confirm that the merge was completed correctly.',
    )}
  </p>
{/if}

<div class="review-card metadata-card">
  {#if pr.summary}
    <div class="meta-row">
      <span class="meta-label">{t('Summary')}</span>
      <p class="meta-value summary-value">{pr.summary}</p>
    </div>
  {/if}
  <div class="meta-row">
    <span class="meta-label">{t('PR ID')}</span>
    <span class="meta-value">{pr.pullRequestId}</span>
  </div>
  <div class="meta-row">
    <span class="meta-label">{t('PR link')}</span>
    <a
      class="meta-value meta-link"
      href={externalUrl(pr.pullRequestUrl)}
      rel="noreferrer"
      target="_blank">{pr.pullRequestUrl}</a
    >
  </div>
  {#if mode === 'vote-pr'}
    {#if pr.merge}
      <div class="meta-row">
        <span class="meta-label">{t('Merge ID')}</span>
        <span class="meta-value">{pr.merge.mergeId}</span>
      </div>
      <div class="meta-row">
        <span class="meta-label">{t('Merge link')}</span>
        <a
          class="meta-value meta-link"
          href={externalUrl(pr.merge.mergeUrl)}
          rel="noreferrer"
          target="_blank">{pr.merge.mergeUrl}</a
        >
      </div>
    {/if}
    <div class="meta-row">
      <span class="meta-label">{t('Submitted by')}</span>
      <span class="meta-value">{author}</span>
    </div>
  {/if}
  <span class="stage-pill">{pr.stageLabel}</span>
</div>

{#if mode === 'vote-pr'}
  <div class="vote-card">
    <strong>{voteSummaryLine(pr.tally)}</strong>
    <small>{voteRequirementLine(pr.tally)}</small>
    {#if canVote(pr) || pr.tally.myVote}
      <VoteChoiceBar
        mine={pr.tally.myVote === 'Neutral' ? null : pr.tally.myVote}
        onChoose={(c) => c !== 'Neutral' && onVote(c)}
      />
    {/if}
  </div>
{:else}
  {@render children?.()}
{/if}
<!-- eslint-enable svelte/no-navigation-without-resolve -->
