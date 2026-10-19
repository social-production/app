<script lang="ts">
  import ContentMetaRow from '#lib/components/cards/ContentMetaRow.svelte';
  import CountPill from '#lib/components/cards/CountPill.svelte';
  import VoteStrip from '#lib/components/cards/VoteStrip.svelte';
  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import {
    commentLabel,
    type CommentSort,
    type ThreadItem,
    type VoteDirection,
  } from '#lib/domain/threads.ts';

  import { resolve } from '$app/paths';
  import type { ResolvedPathname } from '$app/types';

  import '#lib/components/styles/cards.css';
  import '#lib/components/styles/discussion.css';

  /**
   * web's ThreadOverviewPanel: where the thread lives (web's channel tags; here its project),
   * title, opening post, votes, comment count, byline and the comment sort. Share and report
   * arrive with DMs (M4c) and moderation (M5).
   */
  interface Props {
    thread: ThreadItem;
    authorName: string;
    scopeTitle: string;
    scopeHref: ResolvedPathname;
    sort: CommentSort;
    onSort: (sort: CommentSort) => void;
    onVote: (next: VoteDirection) => Promise<void>;
  }

  let { thread, authorName, scopeTitle, scopeHref, sort, onSort, onVote }: Props = $props();

  const sortOptions: { value: CommentSort; label: string }[] = [
    { value: 'oldest', label: 'Oldest first' },
    { value: 'newest', label: 'Newest first' },
    { value: 'top', label: 'Top voted' },
  ];
</script>

<section class="overview-shell">
  <div class="feed-card-top">
    <div class="feed-card-tags">
      <div class="tag-list plain">
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- href arrives resolved (ResolvedPathname) -->
        <a class="tag-plain" href={scopeHref}><span>{scopeTitle}</span></a>
      </div>
    </div>
  </div>

  <h1>{thread.title}</h1>
  <p class="overview-copy">{thread.body}</p>

  <div class="footer detail-footer no-truncate">
    <div class="engagement-row feed-corner-actions">
      <VoteStrip corner count={thread.voteCount} active={thread.myVote} {onVote} />
      <span class="comment-link">
        <CountPill label={commentLabel(thread.commentCount)} />
      </span>
    </div>
    <div class="footer-meta">
      <ContentMetaRow
        fullName
        {authorName}
        authorHref={resolve('/profile/[agent]', { agent: thread.author })}
        createdAt={thread.lastActivityAt}
      />
    </div>
    <div class="detail-tools">
      <span class="detail-tool">
        <IconMenuButton
          value={sort}
          ariaLabel="Sort comments"
          defaultValue="oldest"
          options={sortOptions}
          onChange={onSort}
        >
          <ToolbarIcon name="sort" />
        </IconMenuButton>
      </span>
    </div>
  </div>
</section>
