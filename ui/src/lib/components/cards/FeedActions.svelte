<script lang="ts">
  import { t, plural } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';

  import GroupsIcon from '#lib/components/icons/GroupsIcon.svelte';
  import { shareLink, type ShareKind } from '#lib/components/overlay/shareLink.ts';
  import { formatCompactCount } from '#lib/domain/count.ts';

  import '#lib/components/styles/ux/feed.css';
  import '#lib/components/styles/ux/item.css';
  import '#lib/components/styles/ux/feedpills.css';

  /**
   * A feed row's action row: the votes (the slot), comments, members, Share, as the same soft
   * pills the item's own page shows under it (owner, 2026-11-24: "the same bar as they do
   * inside details").
   */
  interface Props {
    /** Resolved path (+ query or hash) of the comments. */
    commentHref: string;
    commentCount: number;
    memberCount?: number | null;
    /** Resolved path of the item, shared as a full link. */
    shareHref: string;
    shareTitle: string;
    /** What the share sheet calls it and how a sent share is worded. */
    shareKind?: ShareKind;
    children: Snippet;
  }

  let {
    commentHref,
    commentCount,
    memberCount = null,
    shareHref,
    shareTitle,
    shareKind = 'project',
    children,
  }: Props = $props();

  const sheet = shareLink();

  const commentLabel = $derived(`${plural(commentCount, '{n} comment', '{n} comments')}`);
  let shared = $state<'' | 'Link copied' | 'Copy failed'>('');
  let timer: ReturnType<typeof setTimeout> | null = null;

  /** The app's share sheet (Owner Notes Mockups §6); outside the shell, web's shareLink: the
   * device's share sheet, else the link copied. */
  async function share(event: MouseEvent): Promise<void> {
    event.stopPropagation();
    if (sheet) {
      sheet.open({ kind: shareKind, title: shareTitle, path: shareHref });
      return;
    }
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

<div class="feed-actions item-actions feed-pills">
  {@render children()}
  <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved path + query -->
  <a class="item-pill" href={commentHref} aria-label={commentLabel}>
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
  {#if memberCount !== null}
    <span class="item-pill static" title={`${memberCount} members`}>
      <GroupsIcon className="feed-action-icon" />
      <span>{formatCompactCount(memberCount)}</span>
    </span>
  {/if}
  <button class="share-button" type="button" title={t('Share')} onclick={share}>
    <svg aria-hidden="true" viewBox="0 0 24 24"
      ><path
        d="M12 15V4.5M8 8l4-3.5L16 8M6 12.5V18a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-5.5"
        fill="none"
        stroke="currentColor"
        stroke-width="1.8"
        stroke-linecap="round"
        stroke-linejoin="round"
      /></svg
    >
    <span>{t('Share')}</span>
    <span class="feed-action-note" role="status">{shared}</span>
  </button>
</div>
