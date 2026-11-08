<script lang="ts">
  import { resolve } from '$app/paths';

  import CountBadge from '#lib/components/icons/CountBadge.svelte';
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
   * web's ProjectSoftwareGovernancePanel (ADR 0086): the repository and who may merge, the
   * in-progress pull requests (three stage dots; Assess or Record merge when expanded) and the
   * repository replacement history. The wizard opens from here and from the dock.
   */
  let { sw }: { sw: SoftwareState } = $props();

  let queueOpen = $state(false);
  let expanded = $state('');

  const g = $derived(sw.governance);
  const open = $derived(g?.pullRequests.filter(inProgress) ?? []);
  const history = $derived(g?.repositoryRequests.filter((r) => r.tally.status === 'Passed') ?? []);
  const name = (agent: string): string => sw.names[agent] || 'unknown';
  const prId = (hash: string): string =>
    g?.pullRequests.find((p) => p.id === hash)?.pullRequestId ?? hash;

  $effect(() => {
    if (sw.activeId) {
      queueOpen = true;
      expanded = sw.activeId;
    }
  });

  function canAct(pr: SoftwarePullRequest): boolean {
    return pr.stage === 'awaiting-merge' ? pr.canRecordMerge : canVote(pr);
  }

  function act(pr: SoftwarePullRequest): void {
    sw.open(pr.stage === 'awaiting-merge' ? 'record-merge' : 'vote-pr', pr);
  }

  function toggle(id: string): void {
    expanded = expanded === id ? '' : id;
  }

  function onKey(event: KeyboardEvent, id: string): void {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      toggle(id);
    }
  }
</script>

<!-- eslint-disable svelte/no-navigation-without-resolve -- the repository's links are off the app -->
{#if g}
  <section id="software-governance-panel" class="software-panel">
    <div class="repo-card">
      <div class="repo-cell">
        <span class="repo-title">Repository</span>
        <a class="repo-value" href={externalUrl(g.repositoryUrl)} rel="noreferrer" target="_blank">
          {g.repositoryUrl.replace(/^https?:\/\//, '')}
        </a>
      </div>
      <div class="repo-cell">
        <span class="repo-title">Mergers</span>
        {#if g.mergers.length > 0}
          <span class="merger-names">
            {#each g.mergers as member, index (member.agent)}
              <a
                class="repo-value"
                href={resolve('/profile/[agent]', { agent: member.agent })}
                title={`Open ${name(member.agent)}'s profile`}>{name(member.agent)}</a
              >{#if index < g.mergers.length - 1}<span class="repo-comma">, </span>{/if}
            {/each}
          </span>
        {:else}
          <span class="merger-empty">No one yet</span>
        {/if}
      </div>
    </div>

    {#if open.length > 0}
      <div class="detail-card pr-queue-card">
        <button
          class="pr-queue-toggle"
          type="button"
          aria-expanded={queueOpen}
          onclick={() => (queueOpen = !queueOpen)}
        >
          <span class="pr-queue-toggle-copy">
            <span class="pr-queue-title">In-progress pull requests</span>
            <CountBadge count={open.length} />
          </span>
          <span class="pr-queue-chevron" class:open={queueOpen} aria-hidden="true">▾</span>
        </button>

        {#if queueOpen}
          <div class="pr-card-list" class:scrollable={open.length > 5}>
            {#each open as pr (pr.id)}
              {@const isOpen = expanded === pr.id}
              {@const filled = stageFilled(pr.stage)}
              <article class="pr-card" class:expanded={isOpen} id={`software-pr-card-${pr.id}`}>
                <div
                  class="pr-card-body-shell"
                  role="button"
                  tabindex="0"
                  aria-expanded={isOpen}
                  aria-label={isOpen ? `Collapse ${pr.title}` : `Expand ${pr.title}`}
                  onclick={() => toggle(pr.id)}
                  onkeydown={(event) => onKey(event, pr.id)}
                >
                  <div class="pr-card-toggle">
                    <span class="pr-card-top">
                      <span class="pr-card-title-group">
                        <strong class="pr-card-title">{pr.title}</strong>
                      </span>
                      <span
                        class="stage-dots"
                        title={stageTooltip(pr.stage)}
                        aria-label={stageTooltip(pr.stage)}
                      >
                        {#each [1, 2, 3] as step (step)}
                          <span class="stage-dot" class:filled={step <= filled}></span>
                        {/each}
                      </span>
                    </span>
                  </div>

                  <div class="pr-card-link-row">
                    <a
                      class="repo-link"
                      href={externalUrl(pr.pullRequestUrl)}
                      rel="noreferrer"
                      target="_blank"
                      onclick={(event) => event.stopPropagation()}>{pr.pullRequestId}</a
                    >
                  </div>

                  {#if isOpen}
                    <div class="pr-card-body">
                      <span class="stage-copy">{stageHelperCopy(pr.stage)}</span>
                      {#if pr.summary}
                        <span class="summary-copy">{pr.summary}</span>
                      {/if}
                      <div class="pr-card-meta">
                        <a
                          class="repo-link"
                          href={externalUrl(pr.pullRequestUrl)}
                          rel="noreferrer"
                          target="_blank"
                          onclick={(event) => event.stopPropagation()}>{pr.pullRequestUrl}</a
                        >
                        {#if pr.merge}
                          <span class="merge-row">
                            <span>Merge ID:</span>
                            <a
                              class="repo-link"
                              href={externalUrl(pr.merge.mergeUrl)}
                              rel="noreferrer"
                              target="_blank"
                              onclick={(event) => event.stopPropagation()}>{pr.merge.mergeId}</a
                            >
                          </span>
                          <a
                            class="repo-link"
                            href={externalUrl(pr.merge.mergeUrl)}
                            rel="noreferrer"
                            target="_blank"
                            onclick={(event) => event.stopPropagation()}>{pr.merge.mergeUrl}</a
                          >
                        {/if}
                        <small>{compactVoteStatus(pr.tally)}</small>
                      </div>
                    </div>
                  {/if}

                  <div class="pr-card-footer">
                    {#if isOpen && canAct(pr)}
                      <div class="pr-card-actions">
                        <button
                          class="detail-action-button primary-action"
                          type="button"
                          onclick={(event) => {
                            event.stopPropagation();
                            act(pr);
                          }}
                        >
                          {pr.stage === 'awaiting-merge' ? 'Record merge' : 'Assess'}
                        </button>
                      </div>
                    {:else}
                      <span></span>
                    {/if}
                    <div class="pr-card-meta-corner">
                      <span class="expand-chevrons" class:open={isOpen} aria-hidden="true">
                        {#each [1, 2, 3] as step (step)}
                          <span
                            class="expand-chevron"
                            class:filled={step <= filled}
                            class:pulse={step === filled && !isOpen}>▾</span
                          >
                        {/each}
                      </span>
                      <span>{name(pr.author)} · {relativeTime(pr.createdAt)}</span>
                    </div>
                  </div>
                </div>
              </article>
            {/each}
          </div>
        {/if}
      </div>
    {/if}

    {#if history.length > 0}
      <div class="detail-card">
        <span>Repository replacement history</span>
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
                Replaced
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
