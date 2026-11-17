<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import {
    commentContext,
    inChat,
    spaceKind,
    type CommentActivity,
  } from '#lib/domain/commentNotes.ts';
  import { tagRef } from '#lib/domain/scopes.ts';
  import { describeUpdate, relativeCompact } from '#lib/domain/time.ts';

  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';

  import FeedSurface from './FeedSurface.svelte';
  import { spaceHref } from './spaceHref.ts';

  import '#lib/components/styles/cards.css';

  /**
   * A comment in a feed (Profiles and Comments mockups §3, ADR 0107): a speech-bubble line
   * says who commented, the comment is the row (with its author's photo), and what it was on
   * hangs underneath as a small quoted card. Tapping the row opens the comment in its space.
   */
  interface Props {
    comment: CommentActivity;
    /** The viewer wrote it: "You commented". */
    you?: boolean;
  }

  let { comment, you = false }: Props = $props();

  const space = $derived(spaceHref(comment.space.mode, comment.space.id));
  const href = $derived(
    (inChat(comment.space.mode)
      ? `${space}?tab=chat&comment=${encodeURIComponent(comment.message)}`
      : `${space}?comment=${encodeURIComponent(comment.message)}`) as ResolvedPathname,
  );
  const scope = $derived(comment.space.tag ? tagRef(comment.space.tag).label : '');
  let copied = $state(false);

  /** Share copies the comment's link (it opens the comment where it was written). */
  async function share(event: MouseEvent): Promise<void> {
    event.preventDefault();
    event.stopPropagation();
    await navigator.clipboard?.writeText(`${location.origin}${href}`).catch(() => {});
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="comment-activity-card">
  <FeedSurface {href} tone="personal" accent={null}>
    <p class="feed-context">
      <svg aria-hidden="true" viewBox="0 0 24 24"><path d="M5 5h14v10H9l-4 4z" /></svg>
      <span>{commentContext(comment, you)}</span>
    </p>
    <div class="post-row">
      <AvatarBadge name={comment.authorName} />
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
        <!-- Reply and Share, plain, as drawn (Profiles mockups §3). Reply opens the comment
             where it was written; a comment row carries no votes to show. -->
        <div class="comment-activity-actions">
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href is built from spaceHref, which resolves -->
          <a class="comment-activity-action" {href}
            ><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M5 5h14v10H9l-4 4z" /></svg
            >Reply</a
          >
          <button
            class="comment-activity-action"
            type="button"
            aria-label="Copy link to this comment"
            onclick={(e) => void share(e)}
            ><svg aria-hidden="true" viewBox="0 0 24 24"
              ><path d="M12 15V4M8 8l4-4 4 4M5 13v6h14v-6" /></svg
            >{copied ? 'Copied' : ''}</button
          >
        </div>
      </div>
    </div>
  </FeedSurface>
</div>
