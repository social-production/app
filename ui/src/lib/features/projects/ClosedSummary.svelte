<script lang="ts">
  import type { DetailState } from './detailState.svelte.ts';

  /**
   * A closed project's Take part (Plan, Phase and Feedback Screens, 4): the closing reason
   * and its vote lead, then what the project did and links to its past activity and its
   * decisions. A row to where the work continues waits for Convert (ADR 0106).
   */
  interface Props {
    detail: DetailState;
    onActivity: () => void;
  }

  let { detail, onActivity }: Props = $props();

  const project = $derived(detail.project);
  const closing = $derived(
    detail.proposals
      .filter((p) => 'Phase' in p.kind && p.kind.Phase.to === 'phase-7' && p.status === 'Passed')
      .sort((a, b) => b.createdAt - a.createdAt)[0] ?? null,
  );
  const reason = $derived(closing && 'Phase' in closing.kind ? closing.kind.Phase.reason : '');
  const when = $derived(
    new Date((closing?.closedAt ?? closing?.createdAt ?? 0) / 1000).toLocaleDateString(undefined, {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    }),
  );
  const noun = $derived(project?.charter?.mode === 'event' ? 'event' : 'project');
  const decided = $derived(detail.proposals.filter((p) => p.status !== 'Open').length);
  const historyHref = $derived(`${location.pathname}?tab=history`);
</script>

{#if project}
  <section class="closed-summary">
    <div class="closed-banner">
      <span class="closed-chip"
        >Closed{#if closing}
          · {when}{/if}</span
      >
      <strong>This {noun} has finished</strong>
      {#if closing}
        <p>
          “{reason}”
          <span class="closed-vote"
            >{detail.names[closing.author] ?? '…'} · passed {closing.yes} to {closing.no}</span
          >
        </p>
      {/if}
    </div>
    <div class="closed-stats">
      <div><b>{project.members.length}</b>people took part</div>
      <div><b>{decided}</b>decisions</div>
      <div><b>{project.updates.length}</b>updates</div>
    </div>
    <div class="closed-rows">
      <button type="button" onclick={onActivity}>
        <span><small>Past activity</small>Its activities and roles</span>
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg>
      </button>
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- this page's own path -->
      <a href={historyHref}>
        <span><small>Decisions</small>{decided} votes in History</span>
        <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M9.5 6.5 15 12l-5.5 5.5" /></svg>
      </a>
    </div>
  </section>
{/if}
