<script lang="ts">
  import { resolve } from '$app/paths';
  import type { NotificationItem } from '#lib/domain/notifications.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import FeedSurface from './FeedSurface.svelte';

  import '#lib/components/styles/notifications.css';

  /** web's inbox NotificationCard (project and chat kinds: the app has no follows yet). */
  interface Props {
    item: NotificationItem;
    actorName: string;
    onRead: () => void;
    onActivate: () => void;
  }

  let { item, actorName, onRead, onActivate }: Props = $props();

  function handleCardClick(event: MouseEvent): void {
    if ((event.target as HTMLElement).closest('a, button')) return;
    onActivate();
  }

  function handleCardKeydown(event: KeyboardEvent): void {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    onActivate();
  }
</script>

<FeedSurface
  compact
  clampExcerpts={false}
  accent={item.projectMode ? modeAccent(item.projectMode) : 'var(--type-accent-thread)'}
>
  <div
    class:unread={item.isUnread}
    class="notification-card"
    onclick={handleCardClick}
    onkeydown={handleCardKeydown}
    role="link"
    tabindex="0"
  >
    <div class="kicker">
      {#if item.isUnread}
        <span class="unread-dot"></span>
      {/if}
      <span class="subject-label">{item.subject === 'thread' ? 'Thread' : 'Project'}</span>
    </div>

    <div class="copy-row">
      <div class="copy">
        {#if item.lead}
          <p class="title-text">{item.lead}</p>
          {#if item.body}
            <p class="body excerpt">{item.body}</p>
          {/if}
        {:else}
          <p class="title-text">{item.title}</p>
          <p class="body">{item.body}</p>
        {/if}
      </div>
    </div>

    <!-- Bottom row on every card: actions (if any) left, name · time right, never squeezed. -->
    <div class="meta-row">
      <div class="meta-actions">
        {#if item.isUnread}
          <button class="mark-read" type="button" onclick={onRead}>Mark read</button>
        {/if}
      </div>

      <div class="footer-meta">
        <ContentMetaRow
          fullName
          authorName={actorName}
          authorHref={resolve('/profile/[agent]', { agent: item.actor })}
          createdAt={item.createdAt}
        />
      </div>
    </div>
  </div>
</FeedSurface>
