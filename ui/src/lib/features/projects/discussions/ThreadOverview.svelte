<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import PausedBadge from '#lib/components/people/PausedBadge.svelte';
  import { isHidden } from '#lib/domain/moderation.ts';

  import { resolve } from '$app/paths';

  import '#lib/components/styles/ux/detailfacts.css';

  import DiscussionActions from './DiscussionActions.svelte';
  import type { DiscussionState } from './discussionState.svelte.ts';

  /**
   * web's ThreadOverviewPanel (ADR 0100). Detail Pages Cleanup §3: the shared header (the
   * thread icon, the channel and "Thread · 12m"), the title, "Asked by" with the date, the
   * body, then the votes, comment count and Share in one row above the comments.
   */
  let { state }: { state: DiscussionState } = $props();

  const project = $derived(state.project);
  const charter = $derived(state.project?.charter);
  const name = $derived(project ? (state.names[project.creator] ?? '…') : '…');
  const started = $derived(
    project
      ? new Date(project.createdAt / 1000).toLocaleDateString(undefined, {
          day: 'numeric',
          month: 'short',
        })
      : '',
  );
</script>

{#if project && charter}
  <section class="overview-shell item-head">
    <FeedCardTop tags={charter.tags} stage="" mode="thread" createdAt={project.createdAt}>
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
      <p class="detail-by thread-by">
        <AvatarBadge size="sm" {name} />
        <span>
          {t('Asked by')}
          <a href={resolve('/profile/[agent]', { agent: project.creator })}>{name}</a>
          <PausedBadge agent={project.creator} />
          · {started}
        </span>
      </p>
      <p class="overview-copy">{charter.description}</p>
    </RestrictionNotice>
    <DiscussionActions {state} kind="thread" title={charter.title} />
  </section>
{/if}
