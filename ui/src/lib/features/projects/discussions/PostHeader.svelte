<script lang="ts">
  import LinkedPostBody from '#lib/components/cards/LinkedPostBody.svelte';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import { isHidden } from '#lib/domain/moderation.ts';
  import { postedAt } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  import DiscussionActions from './DiscussionActions.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * web's PostDetailPage header, laid out as the feed's post row opened up (ADR 0100): the
   * author with web's ⋮ report menu (ADR 0071), the body, when it was posted, and the bare
   * action row (comments, votes, Share, Sort).
   */
  let { state }: { state: DiscussionState } = $props();

  const project = $derived(state.project);
  const charter = $derived(state.project?.charter);
  const name = $derived(project ? (state.names[project.creator] ?? '…') : '…');
</script>

{#if project && charter}
  <section class="item-head post-head-detail">
    <div class="item-author">
      <AvatarBadge {name} />
      <a
        class="name header-name"
        href={resolve('/profile/[agent]', { agent: project.creator })}
        title={name}>{name}</a
      >
      <ReportControl
        id={state.id}
        itemLabel="post"
        moderation={state.moderation.value}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={(next) => state.moderation.set(next)}
      />
    </div>

    <RestrictionNotice active={isHidden(state.moderation.value)}>
      <LinkedPostBody body={charter.description} variant="detail" />
    </RestrictionNotice>

    <p class="item-when">{postedAt(project.createdAt)}</p>
    <DiscussionActions {state} kind="post" title={`${name}'s post`} />
  </section>
{/if}
