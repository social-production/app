<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import FactLine from '#lib/components/cards/FactLine.svelte';
  import FeedCardTop from '#lib/components/cards/FeedCardTop.svelte';
  import FoldedText from '#lib/components/cards/FoldedText.svelte';
  import PlaceRow from '#lib/components/cards/PlaceRow.svelte';
  import { tagHref } from '#lib/components/cards/tagLinks.ts';
  import ReportControl from '#lib/components/moderation/ReportControl.svelte';
  import RestrictionNotice from '#lib/components/moderation/RestrictionNotice.svelte';
  import { isHidden, type Moderation } from '#lib/domain/moderation.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';
  import { scheduleLine } from '#lib/domain/eventSchedule.ts';
  import type { PlanBoard } from '#lib/domain/plans.ts';
  import type { Charter, Project } from '#lib/domain/projects.ts';
  import { tagRef } from '#lib/domain/scopes.ts';
  import { relativeTime } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';

  /**
   * A project's or event's Details tab (Plan, Phase and Feedback Screens, 1): the channel
   * header, the title, who started it and when, the date (events), the place as a row that
   * opens the app's map, the description folded after about six lines, the tags as chips,
   * the latest update in its own soft block, and for an event the faces of who is going.
   * An event shows its leading plan's time and place, as web-backend copies them onto the
   * event (`_sync_event_schedule_from_leading_plan`).
   */
  interface Props {
    project: Project;
    charter: Charter;
    plans: PlanBoard | undefined;
    names: Record<string, string>;
    /** web's ReportControl in the header (ADR 0071). */
    moderation: Moderation;
    onModeration: (next: Moderation) => void;
    /** Opens the member list (an event's "N going"). */
    onMembers?: () => void;
  }

  let { project, charter, plans, names, moderation, onModeration, onMembers }: Props = $props();

  const event = $derived(charter.mode === 'event');
  const leading = $derived(
    plans?.plans.find((p) => p.plan.kind === 'Event' && p.leader === 'Leading' && p.approved)?.plan,
  );
  const timeLabel = $derived(leading?.schedule ? scheduleLine(leading.schedule, zoneOf) : '');
  const place = $derived((leading?.location || charter.location_label).trim());
  const update = $derived(project.updates[0] ?? null);
  /** web strips a leading "Update:" so the line doesn't say it twice. */
  const latest = $derived(update?.body.replace(/^\s*update:\s*/i, '') ?? null);
  const author = $derived(names[project.creator] ?? '…');
  const started = $derived(
    new Date(project.createdAt / 1000).toLocaleDateString(undefined, {
      day: 'numeric',
      month: 'short',
    }),
  );
  const going = $derived(project.members.slice(0, 3));
</script>

<div class="project-overview">
  <div class="context-panel">
    <FeedCardTop
      mode={charter.mode}
      tags={charter.tags}
      stage={project.stage}
      isPrivate={project.audience !== 'public'}
      createdAt={project.createdAt}
    >
      <ReportControl
        id={project.id}
        itemLabel={event ? 'event' : 'project'}
        {moderation}
        owner={project.creator}
        copyUrl={`${location.origin}${location.pathname}`}
        onChange={onModeration}
      />
    </FeedCardTop>
    <div class="heading overview-heading">
      <RestrictionNotice active={isHidden(moderation)}>
        <h1>{charter.title}</h1>
      </RestrictionNotice>
      <p class="detail-by">
        <AvatarBadge size="sm" name={author} />
        <span>
          {event ? 'Organised by' : 'Started by'}
          <a href={resolve('/profile/[agent]', { agent: project.creator })}>{author}</a>
          · {started}
        </span>
      </p>
      {#if timeLabel}<FactLine icon="when">{timeLabel}</FactLine>{/if}
      <PlaceRow {place} />
      {#if charter.description.trim()}
        <FoldedText text={charter.description} className="overview-copy" />
      {/if}
      {#if charter.tags.length > 1}
        <!-- The first tag leads the header; the rest are chips. -->
        <div class="detail-tags">
          {#each charter.tags.slice(1).map(tagRef) as tag (tag.tag)}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- tagHref resolves -->
            <a href={tagHref(tag)}>{tag.label}</a>
          {/each}
        </div>
      {/if}
      {#if update && latest !== null}
        <div class="overview-update">
          <small>Latest update · {relativeTime(update.at)}</small>
          {latest}
        </div>
      {/if}
      {#if event && project.members.length > 0}
        <button class="detail-going" type="button" onclick={() => onMembers?.()}>
          <span class="detail-faces">
            {#each going as agent (agent)}<AvatarBadge size="sm" name={names[agent] ?? ''} />{/each}
          </span>
          <span><strong>{project.members.length} going</strong></span>
        </button>
      {/if}
    </div>
  </div>
</div>
