<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import { inChat, spaceKind, type CommentActivity } from '#lib/domain/commentNotes.ts';
  import { formatCompactCount } from '#lib/domain/count.ts';
  import { tagRef } from '#lib/domain/scopes.ts';
  import { t } from '#lib/i18n/index.ts';
  import { describeUpdate, relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';

  import { commentLike } from './commentLike.ts';
  import FeedSurface from './FeedSurface.svelte';
  import { spaceHref } from './spaceHref.ts';

  import '#lib/components/styles/cards.css';

  /**
   * A comment in a feed (Profiles and Comments mockups §3, ADR 0107): the comment is the row
   * (with its author's photo) and what it was on hangs underneath as a small quoted card, so
   * no "x commented" line or Reply/Share row is needed (owner, 2026-11-23). Tapping the row
   * opens the comment in its space.
   */
  interface Props {
    comment: CommentActivity;
    /** The viewer wrote it (no longer shown: the row carries the name). */
    you?: boolean;
  }

  let { comment }: Props = $props();

  const space = $derived(spaceHref(comment.space.mode, comment.space.id));
  const href = $derived(
    (inChat(comment.space.mode)
      ? `${space}?tab=chat&comment=${encodeURIComponent(comment.message)}`
      : `${space}?comment=${encodeURIComponent(comment.message)}`) as ResolvedPathname,
  );
  const scope = $derived(comment.space.tag ? tagRef(comment.space.tag).label : '');

  // A thread's or post's comment shows its likes and replies, as in the thread (owner,
  // 2026-11-24). The heart likes it here; the count follows at once.
  const liker = commentLike();
  let mine = $state<boolean | null>(null);
  const liked = $derived(mine ?? comment.counts?.liked ?? false);
  const likes = $derived(
    (comment.counts?.likes ?? 0) +
      (mine === null ? 0 : (mine ? 1 : 0) - (comment.counts?.liked ? 1 : 0)),
  );
  function like(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    const next = !liked;
    mine = next;
    liker?.like(comment.space.id, comment.message, next).catch(() => (mine = !next));
  }
</script>

<div class="comment-activity-card">
  <FeedSurface {href} tone="personal" accent={null}>
    <div class="post-row">
      <AvatarBadge name={comment.authorName} agent={comment.author} />
      <div class="post-main">
        <div class="post-head">
          <a
            class="header-name"
            href={resolve('/profile/[agent]', { agent: comment.author })}
            title={comment.authorName}>{comment.authorName}</a
          >
          <span class="feed-card-time" title={describeUpdate(comment.createdAt, null)}
            >{relativeCompact(comment.createdAt)}</span
          >
        </div>
        <p class="comment-excerpt">{comment.excerpt}</p>
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- spaceHref resolves it -->
        <a class="comment-quote" href={space}>
          <span class="comment-quote-glyph"
            ><SurfaceTypeLabel projectMode={comment.space.mode} feedGlyph /></span
          >
          <span class="comment-quote-copy">
            <small>{scope ? `${scope} · ` : ''}{spaceKind(comment.space.mode)}</small>
            <strong>{comment.space.title}</strong>
          </span>
        </a>
        {#if comment.counts}
          <div class="comment-activity-acts">
            <button
              class="comment-act like"
              class:on={liked}
              type="button"
              aria-pressed={liked}
              aria-label={liked ? t('Unlike') : t('Like')}
              disabled={!liker}
              onclick={like}
            >
              <svg aria-hidden="true" viewBox="0 0 24 24">
                <path
                  d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"
                  fill={liked ? 'currentColor' : 'none'}
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linejoin="round"
                />
              </svg>
              {#if likes > 0}<span>{formatCompactCount(likes)}</span>{/if}
            </button>
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- built from spaceHref -->
            <a class="comment-act" {href} aria-label={t('Replies')}>
              <svg aria-hidden="true" viewBox="0 0 24 24">
                <path
                  d="M6 5.5h12a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linejoin="round"
                />
              </svg>
              {#if comment.counts.replies > 0}<span
                  >{formatCompactCount(comment.counts.replies)}</span
                >{/if}
            </a>
          </div>
        {/if}
      </div>
    </div>
  </FeedSurface>
</div>
