<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonDetail from '#lib/components/loading/SkeletonDetail.svelte';
  import LoadingLine from '#lib/components/loading/LoadingLine.svelte';
  import FeedSurface from '#lib/components/cards/FeedSurface.svelte';
  import { MentionSource } from '#lib/components/chat/mentionSource.svelte.ts';
  import DiscussionPanel from '#lib/components/discussion/DiscussionPanel.svelte';
  import { commentPath, type DiscussionComment as Comment } from '#lib/domain/discussions.ts';
  import { removedLabel } from '#lib/domain/moderation.ts';
  import { modeAccent } from '#lib/domain/projects.ts';
  import { mentionPool } from '#lib/services/messages.ts';
  import { myAgent } from '#lib/services/people.ts';
  import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
  import { connection } from '#lib/stores/connection.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';
  import { page } from '$app/state';

  import { DiscussionState } from './discussionState.svelte.ts';
  import PostCommentView from './PostCommentView.svelte';
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

  // Posts: a comment opens as its own view, `?reply=<id>` (owner, 2026-11-24); Back returns.
  // A link to a reply (a notification, a profile) opens the comment it answers.
  const linkedParent = $derived.by(() => {
    const path = highlighted ? commentPath(state.comments, highlighted) : null;
    return path && path.length > 1 ? (path.at(-2)?.id ?? null) : null;
  });
  const opened = $derived(
    kind === 'post' ? (page.url.searchParams.get('reply') ?? linkedParent) : null,
  );
  function openComment(comment: string): void {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- resolved, plus a query
    void goto(`${resolve('/posts/[id]', { id })}?reply=${encodeURIComponent(comment)}`);
  }

  // web's comment @mentions: people in this discussion come first (as in project chats).
  let me = ''; // read when a search runs, so it needn't be reactive
  const signedIn = $derived(connection.current.status === 'connected');
  const mentions = new MentionSource(
    () => mentionPool(),
    () => ({ me, agents: Object.keys(state.names), names: state.names }),
  );

  const bodies = (list: Comment[]): string[] => list.flatMap((c) => [c.body, ...bodies(c.replies)]);

  $effect(() => {
    if (signedIn) void myAgent().then((agent) => (me = agent));
  });

  $effect(() => {
    if (signedIn) mentions.watch(bodies(state.comments));
  });
</script>

<div class="discussion-page">
  <section class="page">
    {#if state.error && !state.project}
      <p class="tab-loading">{state.error}</p>
    {:else if state.moderation.value.state === 'removed'}
      <p class="tab-loading">{t(removedLabel(state.moderation.value.report?.reason))}</p>
    {:else if !state.project?.charter}
      <LoadingLine /><SkeletonDetail tabs={false} dock={false} />
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
        {#if opened}
          <PostCommentView
            {state}
            space={id}
            focus={opened}
            {highlighted}
            {signedIn}
            mentions={signedIn ? mentions : null}
            onSignIn={(message) => requireViewer(message)}
            onOpen={openComment}
          />
        {:else}
          <DiscussionPanel
            comments={state.comments}
            space={id}
            names={state.names}
            sortMode={state.sortMode}
            {highlighted}
            embedded
            flat={kind === 'post'}
            {signedIn}
            mentions={signedIn ? mentions : null}
            onSignIn={(message) => requireViewer(message)}
            onVote={(c, d) => state.voteComment(c, d)}
            onComment={(parent, body) => state.comment(parent, body)}
            onModeration={(c, next) => state.setCommentModeration(c, next)}
            onOpen={kind === 'post' ? openComment : undefined}
          />
        {/if}
      </FeedSurface>
    {/if}
  </section>
</div>
