<script lang="ts">
  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { activityInstant, describeUpdate, relativeCompact } from '#lib/domain/time.ts';

  import type { ResolvedPathname } from '$app/types';

  /** web's ContentMetaRow: author · members · clock (or loudspeaker once updated). */
  interface Props {
    authorName: string;
    authorHref: ResolvedPathname;
    memberCount?: number | null;
    createdAt: number;
    updatedAt?: number | null;
    /** The whole name (detail pages, where the byline has its own line). */
    fullName?: boolean;
    /** Just the clock (follow notices, whose name is in their own row). */
    timeOnly?: boolean;
  }

  let {
    authorName,
    authorHref,
    memberCount = null,
    createdAt,
    updatedAt = null,
    fullName = false,
    timeOnly = false,
  }: Props = $props();

  const NAME_LIMIT = 18;
  const shortName = $derived(
    !fullName && authorName.length > NAME_LIMIT
      ? `${authorName.slice(0, NAME_LIMIT - 1)}…`
      : authorName,
  );
  const updated = $derived(updatedAt !== null && updatedAt > createdAt);
  const label = $derived(describeUpdate(createdAt, updatedAt));

  import '#lib/components/styles/cards.css';
</script>

<span class="content-meta-row" class:time-only={timeOnly}>
  {#if !timeOnly}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
    <a class="inline-link" href={authorHref} title={authorName}>{shortName}</a>
  {/if}
  {#if memberCount !== null}
    <span class="meta-chip" title={`${memberCount} members`}>
      <GroupsIcon className="meta-icon" />
      <span>{memberCount}</span>
    </span>
  {/if}
  <span class="meta-chip activity-stamp" title={label} aria-label={label}>
    <span class="meta-icon-wrap" aria-hidden="true"
      ><ToolbarIcon name={updated ? 'loudspeaker' : 'clock'} /></span
    >
    <span>{relativeCompact(activityInstant(createdAt, updatedAt))}</span>
  </span>
</span>
