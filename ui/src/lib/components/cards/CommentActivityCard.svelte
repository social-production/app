<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import { inChat, spaceKind, type CommentActivity } from '#lib/domain/commentNotes.ts';
  import { tagRef } from '#lib/domain/scopes.ts';
  import { describeUpdate, relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';

  import { commentLike } from './commentLike.ts';
  import CommentPills from './CommentPills.svelte';
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

  // A thread's or post's comment carries the feed rows' replies · votes · share row (owner,
  // 2026-11-24).
  const voter = commentLike();
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
            <CommentPills
              score={comment.counts.score}
              mine={comment.counts.mine}
              onVote={(direction) =>
                voter
                  ? voter.vote(comment.space.id, comment.message, direction)
                  : Promise.reject(new Error('no voter'))}
              replies={comment.counts.replies}
              replyHref={href}
              sharePath={href}
              shareTitle={`${comment.authorName}: ${comment.excerpt}`}
            />
          </div>
        {/if}
      </div>
    </div>
  </FeedSurface>
</div>
