<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import {
    canVote,
    compactVoteStatus,
    externalUrl,
    inProgress,
    stageFilled,
    stageHelperCopy,
    stageTooltip,
    type SoftwarePullRequest,
  } from '#lib/domain/software.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import type { SoftwareState } from './softwareState.svelte.ts';

  import '#lib/components/styles/software.css';

  /**
   * web's ProjectSoftwareGovernancePanel (ADR 0086) as drawn (Invites, Help Requests and
   * Software Screens, 3): the repository and who may merge as one soft group, the pull
   * requests in progress as cards with a three-step line and their action, and the
   * repository replacement history. The wizard opens from here and from the dock.
   */
  let { sw }: { sw: SoftwareState } = $props();

  let expanded = $state('');

  const g = $derived(sw.governance);
  const open = $derived(g?.pullRequests.filter(inProgress) ?? []);
  const history = $derived(g?.repositoryRequests.filter((r) => r.tally.status === 'Passed') ?? []);
  const name = (agent: string): string => sw.names[agent] || 'unknown';
  const prId = (hash: string): string =>
    g?.pullRequests.find((p) => p.id === hash)?.pullRequestId ?? hash;

  $effect(() => {
    if (sw.activeId) {
      expanded = sw.activeId;
    }
  });

  function canAct(pr: SoftwarePullRequest): boolean {
    return pr.stage === 'awaiting-merge' ? pr.canRecordMerge : canVote(pr);
  }

  function act(pr: SoftwarePullRequest): void {
    sw.open(pr.stage === 'awaiting-merge' ? 'record-merge' : 'vote-pr', pr);
  }

  const STAGE_NAMES = ['Approval', 'Merge', 'Confirmation'];

  /** The card's action: vote while approving, record the merge, then confirm it. */
  function actionLabel(pr: SoftwarePullRequest): string {
    if (pr.stage === 'awaiting-merge') return t('Record merge');
    return pr.stage === 'confirmation' ? t('Confirm merge') : t('Vote');
  }

  function toggle(id: string): void {
    expanded = expanded === id ? '' : id;
  }
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- the repository's links are off the app -->
{#if g}
  <section id="software-governance-panel" class="software-panel software-ux">
    <div class="repo-group">
      <a
        class="repo-cell repo-link-row"
        href={externalUrl(g.repositoryUrl)}
        rel="noreferrer"
        target="_blank"
      >
        <span class="repo-glyph" aria-hidden="true">
          <svg viewBox="0 0 24 24"><path d="M8 7 3 12l5 5M16 7l5 5-5 5M13.5 4.5l-3 15" /></svg>
        </span>
        <span class="repo-copy">
          <small class="repo-title">{t('Repository')}</small>
          <b class="repo-value">{g.repositoryUrl.replace(/^https?:\/\//, '')}</b>
        </span>
        <svg class="chev" aria-hidden="true" viewBox="0 0 24 24"
          ><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg
        >
      </a>
      <div class="repo-cell">
        {#if g.mergers.length > 0}
          <span class="repo-faces" aria-hidden="true">
            {#each g.mergers.slice(0, 3) as member (member.agent)}
              <AvatarBadge size="sm" name={name(member.agent)} />
            {/each}
          </span>
        {/if}
        <span class="repo-copy">
          <small class="repo-title">{t('Mergers')}</small>
          {#if g.platform}
            <b class="merger-names">{t('Platform moderators')}</b>
          {:else if g.mergers.length > 0}
            <b class="merger-names">
              {#each g.mergers as member, index (member.agent)}
                <a href={resolve('/profile/[agent]', { agent: member.agent })}
                  >{name(member.agent)}</a
                >{#if index < g.mergers.length - 1}<span class="repo-comma">, </span>{/if}
              {/each}
            </b>
          {:else}
            <b class="merger-empty">{t('No one yet')}</b>
          {/if}
        </span>
      </div>
    </div>

    <div class="pr-head">
      <h3>{t('Pull requests')}</h3>
      <span>{open.length} in progress</span>
    </div>
    {#each open as pr (pr.id)}
      {@const isOpen = expanded === pr.id}
      {@const filled = stageFilled(pr.stage)}
      <article class="pr-card" class:expanded={isOpen} id={`software-pr-card-${pr.id}`}>
        <div class="pr-card-top">
          <button
            class="pr-card-body-shell"
            type="button"
            aria-expanded={isOpen}
            onclick={() => toggle(pr.id)}><strong class="pr-card-title">{pr.title}</strong></button
          >
          {#if canAct(pr)}
            <button
              class="pr-action"
              class:merge={pr.stage !== 'approval'}
              type="button"
              onclick={() => act(pr)}>{actionLabel(pr)}</button
            >
          {/if}
        </div>
        <span class="pr-meta">
          <a href={externalUrl(pr.pullRequestUrl)} rel="noreferrer" target="_blank"
            >{pr.pullRequestId}</a
          >
          · by {name(pr.author)} · {relativeTime(pr.createdAt)}
        </span>
        <span class="pr-steps" aria-label={stageTooltip(pr.stage)}>
          {#each [1, 2, 3] as step (step)}
            <i class:done={step < filled} class:now={step === filled}></i>
          {/each}
        </span>
        <span class="pr-meta"
          >Step {filled} of 3 · {STAGE_NAMES[filled - 1]} · {compactVoteStatus(pr.tally)}</span
        >
        {#if isOpen}
          <div class="pr-card-body">
            <span class="stage-copy">{stageHelperCopy(pr.stage)}</span>
            {#if pr.summary}<span class="summary-copy">{pr.summary}</span>{/if}
            {#if pr.merge}
              <span class="merge-row">
                {t('Merge')}
                <a href={externalUrl(pr.merge.mergeUrl)} rel="noreferrer" target="_blank"
                  >{pr.merge.mergeId}</a
                >
              </span>
            {/if}
          </div>
        {/if}
      </article>
    {:else}
      <p class="pr-empty">{t('No pull requests in progress.')}</p>
    {/each}

    {#if history.length > 0}
      <div class="detail-card">
        <span>{t('Repository replacement history')}</span>
        <div class="history-list">
          {#each history as entry (entry.id)}
            <div class="history-item">
              <strong>
                <a
                  class="repo-link"
                  href={externalUrl(entry.repositoryUrl)}
                  rel="noreferrer"
                  target="_blank">{entry.repositoryUrl}</a
                >
              </strong>
              <small>
                {t('Replaced')}
                <a
                  class="repo-link"
                  href={externalUrl(entry.previousRepositoryUrl)}
                  rel="noreferrer"
                  target="_blank">{entry.previousRepositoryUrl}</a
                >
                after {prId(entry.pullRequest)}
              </small>
              <small>{entry.reason}</small>
              <small>
                Recorded by {name(entry.author)}
                {entry.tally.closedAt ? relativeTime(entry.tally.closedAt) : ''}
              </small>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </section>
{/if}
<!-- eslint-enable svelte/no-navigation-without-resolve -->
