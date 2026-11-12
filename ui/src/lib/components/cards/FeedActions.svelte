<script lang="ts">
  import type { Snippet } from 'svelte';

  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import { formatCompactCount } from '#lib/domain/count.ts';

  import '#lib/components/styles/ux/feed.css';

  /**
   * A feed row's action row (ADR 0096): comments, the votes (the slot), members, share.
   * Bare icon buttons spread across the row, as on Twitter's phone app.
   */
  interface Props {
    /** Resolved path (+ query or hash) of the comments. */
    commentHref: string;
    commentCount: number;
    memberCount?: number | null;
    /** Resolved path of the item, shared as a full link. */
    shareHref: string;
    shareTitle: string;
    children: Snippet;
  }

  let {
    commentHref,
    commentCount,
    memberCount = null,
    shareHref,
    shareTitle,
    children,
  }: Props = $props();

  const commentLabel = $derived(`${commentCount} ${commentCount === 1 ? 'comment' : 'comments'}`);
  let shared = $state<'' | 'Link copied' | 'Copy failed'>('');
  let timer: ReturnType<typeof setTimeout> | null = null;

  /** web's shareLink: the device's share sheet, else the link copied. */
  async function share(event: MouseEvent): Promise<void> {
    event.stopPropagation();
    const url = new URL(shareHref, window.location.origin).href;
    try {
      if (typeof navigator.share === 'function') {
        await navigator.share({ url, title: shareTitle });
        return;
      }
      await navigator.clipboard.writeText(url);
      shared = 'Link copied';
    } catch (e) {
      if (e instanceof DOMException && e.name === 'AbortError') return;
      shared = 'Copy failed';
    }
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => (shared = ''), 1600);
  }
</script>

<div class="feed-actions">
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved path + query -->
  <a class="feed-action comment-link" href={commentHref} aria-label={commentLabel}>
    <svg aria-hidden="true" viewBox="0 0 24 24" class="feed-action-icon"
      ><path
        d="M6 5.5h12a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linejoin="round"
      /></svg
    >
    <span aria-hidden="true">{formatCompactCount(commentCount)}</span>
  </a>
  {@render children()}
  {#if memberCount !== null}
    <span class="feed-action static" title={`${memberCount} members`}>
      <GroupsIcon className="feed-action-icon" />
      <span>{formatCompactCount(memberCount)}</span>
    </span>
  {/if}
  <button class="feed-action" type="button" aria-label="Share" title="Share" onclick={share}>
    <svg aria-hidden="true" viewBox="0 0 24 24" class="feed-action-icon"
      ><path
        d="M12 15V4.5M8 8l4-3.5L16 8M6 12.5V18a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-5.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      /></svg
    >
    <span class="feed-action-note" role="status">{shared}</span>
  </button>
</div>
