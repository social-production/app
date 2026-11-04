<script lang="ts">
  import LinkedPostBody from '#lib/components/cards/LinkedPostBody.svelte';
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import CountPill from '#lib/components/cards/CountPill.svelte';
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import { countComments } from '#lib/domain/discussions.ts';
  import { isHidden } from '#lib/domain/moderation.ts';

  import { resolve } from '$app/paths';

  import DiscussionTools from './DiscussionTools.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * web's PostDetailPage header: the author, the body, votes, comments, the clock, Share and
   * Sort, with web's ⋮ report menu beside the author (ADR 0071).
   */
  let { state }: { state: DiscussionState } = $props();

  const project = $derived(state.project);
  const charter = $derived(state.project?.charter);
  const count = $derived(countComments(state.comments));
  const name = $derived(project ? (state.names[project.creator] ?? '…') : '…');
</script>

{#if project && charter}
  <div class="card-header">
    <div class="header-row">
      <div class="identity-row">
        <AvatarBadge size="sm" {name} />
        <a
          class="name header-name"
          href={resolve('/profile/[agent]', { agent: project.creator })}
          title={name}>{name}</a
        >
      </div>
      <ReportControl
        id={state.id}
        itemLabel="post"
        moderation={state.moderation.value}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={(next) => state.moderation.set(next)}
      />
    </div>
  </div>

  <RestrictionNotice active={isHidden(state.moderation.value)}>
    <LinkedPostBody body={charter.description} variant="detail" />
  </RestrictionNotice>

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
        timeOnly
        authorName={name}
        authorHref={resolve('/profile/[agent]', { agent: project.creator })}
        createdAt={project.createdAt}
      />
    </div>
    <DiscussionTools kind="post" title={`${name}'s post`} bind:sortMode={state.sortMode} />
  </div>
{/if}
