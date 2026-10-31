<script lang="ts">
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import DiscussionPanel from '#lib/components/discussion/DiscussionPanel.svelte';
  import { modeAccent } from '#lib/domain/projects.ts';
  import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { page } from '$app/state';

  import { DiscussionState } from './discussionState.svelte.ts';
  import PostHeader from './PostHeader.svelte';
  import ThreadOverview from './ThreadOverview.svelte';

  import '#lib/components/styles/discussion.css';

  /**
   * web's ThreadDetailPage and PostDetailPage (ADR 0070): one surface with the item on top and
   * its comments below. The page is re-created per id, so reading `id` once is intended.
   */
  let { id, kind }: { id: string; kind: 'thread' | 'post' } = $props();

  // svelte-ignore state_referenced_locally
  const state = new DiscussionState(id);

  /** web's readCommentTarget: `#comment-<id>` or `?comment=<id>`. */
  const hash = page.url.hash;
  const highlighted = hash.startsWith('#comment-')
    ? hash.slice('#comment-'.length) || null
    : page.url.searchParams.get('comment');

  $effect(() => {
    void state.open();
  });
</script>

<div class="discussion-page">
  <section class="page">
    {#if state.error && !state.project}
      <p class="tab-loading">{state.error}</p>
    {:else if !state.project?.charter}
      <p class="tab-loading">Loading {kind}…</p>
    {:else}
      <FeedSurface
        tone={kind === 'post' ? 'personal' : 'public'}
        accent={modeAccent(kind)}
        clampExcerpts={false}
      >
        {#if kind === 'thread'}
          <ThreadOverview {state} />
        {:else}
          <PostHeader {state} />
        {/if}
        {#if state.message}<p class="status-note" role="alert">{state.message}</p>{/if}
        <DiscussionPanel
          comments={state.comments}
          names={state.names}
          sortMode={state.sortMode}
          {highlighted}
          embedded
          signedIn={connection.current.status === 'connected'}
          onSignIn={(message) => requireViewer(message)}
          onVote={(c, d) => state.voteComment(c, d)}
          onComment={(parent, body) => state.comment(parent, body)}
        />
      </FeedSurface>
    {/if}
  </section>
</div>
