<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import CountPill from '#lib/components/cards/CountPill.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import { countComments } from '#lib/domain/discussions.ts';

  import { resolve } from '$app/paths';

  import DiscussionTools from './DiscussionTools.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /** web's ThreadOverviewPanel: tags, title, body, votes, comments, byline, Share and Sort. */
  let { state }: { state: DiscussionState } = $props();

  const project = $derived(state.project);
  const charter = $derived(state.project?.charter);
  const count = $derived(countComments(state.comments));
</script>

{#if project && charter}
  <section class="overview-shell">
    <FeedCardTop tags={charter.tags} stage="" />
    <h1>{charter.title}</h1>
    <p class="overview-copy">{charter.description}</p>
    <div class="footer detail-footer no-truncate">
      <div class="engagement-row feed-corner-actions">
        <VoteStrip
          corner
          support={project.demand}
          oppose={project.opposition}
          mine={project.mySignal}
          onSignal={(next) => state.vote(next)}
        />
        <span class="comment-link">
          <CountPill label={`${count} ${count === 1 ? 'comment' : 'comments'}`} />
        </span>
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          fullName
          authorName={state.names[project.creator] ?? '…'}
          authorHref={resolve('/profile/[agent]', { agent: project.creator })}
          createdAt={project.lastActivityAt ?? project.createdAt}
        />
      </div>
      <DiscussionTools kind="thread" title={charter.title} bind:sortMode={state.sortMode} />
    </div>
  </section>
{/if}
