<script lang="ts">
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import PausedBadge from '#lib/components/people/PausedBadge.svelte';
  import { isHidden } from '#lib/domain/moderation.ts';
  import { postedAt } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import DiscussionActions from './DiscussionActions.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * web's ThreadOverviewPanel, laid out as the feed's thread row opened up (ADR 0100): the
   * type and tags, the title, the body, who started it and when, and the bare action row
   * (comments, votes, Share, Sort).
   */
  let { state }: { state: DiscussionState } = $props();

  const project = $derived(state.project);
  const charter = $derived(state.project?.charter);
  const name = $derived(project ? (state.names[project.creator] ?? '…') : '…');
</script>

{#if project && charter}
  <section class="overview-shell item-head">
    <FeedCardTop tags={charter.tags} stage="" mode="thread">
      <ReportControl
        id={state.id}
        itemLabel="thread"
        moderation={state.moderation.value}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={(next) => state.moderation.set(next)}
      />
    </FeedCardTop>
    <RestrictionNotice active={isHidden(state.moderation.value)}>
      <h1>{charter.title}</h1>
      <p class="overview-copy">{charter.description}</p>
    </RestrictionNotice>
    <p class="item-when">
      <a class="item-by" href={resolve('/profile/[agent]', { agent: project.creator })}>{name}</a>
      <PausedBadge agent={project.creator} />
      <span>{postedAt(project.createdAt)}</span>
    </p>
    <DiscussionActions {state} kind="thread" title={charter.title} />
  </section>
{/if}
