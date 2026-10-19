<script lang="ts">
  import { commentLabel, type ThreadItem, type VoteDirection } from '#lib/domain/threads.ts';

  import { resolve } from '$app/paths';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import CountPill from './CountPill.svelte';
  import FeedSurface from './FeedSurface.svelte';
  import VoteStrip from './VoteStrip.svelte';

  /** web's public-feed ThreadCard (styles/discussion.css, `.thread-card`). */
  interface Props {
    projectId: string;
    item: ThreadItem;
    authorName: string;
    onVote: (next: VoteDirection) => Promise<void>;
  }

  let { projectId, item, authorName, onVote }: Props = $props();

  const href = $derived(
    resolve('/projects/[id]/threads/[thread]', { id: projectId, thread: item.id }),
  );

  import '#lib/components/styles/discussion.css';
</script>

<div class="thread-card">
  <FeedSurface {href} accent="var(--type-accent-thread)">
    <a
      class="title"
      href={resolve('/projects/[id]/threads/[thread]', { id: projectId, thread: item.id })}
      >{item.title}</a
    >
    <p class="body">{item.body}</p>
    <div class="footer">
      <div class="engagement-row feed-corner-actions">
        <VoteStrip corner count={item.voteCount} active={item.myVote} {onVote} />
        <a
          class="comment-link"
          href={resolve('/projects/[id]/threads/[thread]', { id: projectId, thread: item.id })}
        >
          <CountPill label={commentLabel(item.commentCount)} />
        </a>
      </div>
      <div class="footer-meta">
        <ContentMetaRow
          {authorName}
          authorHref={resolve('/profile/[agent]', { agent: item.author })}
          createdAt={item.createdAt}
        />
      </div>
    </div>
  </FeedSurface>
</div>
