<script lang="ts">
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { NotificationItem } from '#lib/domain/notifications.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import ContentMetaRow from './ContentMetaRow.svelte';
  import FeedSurface from './FeedSurface.svelte';

  import '#lib/components/styles/notifications.css';

  /** web's inbox NotificationCard: project, chat and follow kinds. */
  interface Props {
    item: NotificationItem;
    actorName: string;
    onRead: () => void;
    onActivate: () => void;
    /** Unread follow requests: web's "Accept follower" / "Decline". */
    deciding?: boolean;
    onDecide?: ((accept: boolean) => void) | undefined;
  }

  let { item, actorName, onRead, onActivate, deciding = false, onDecide }: Props = $props();

  const social = $derived(item.subject === 'profile');
  const profileHref = $derived(resolve('/profile/[agent]', { agent: item.actor }));

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
  accent={social
    ? null
    : item.projectMode
      ? modeAccent(item.projectMode)
      : 'var(--type-accent-thread)'}
>
  <div
    class:unread={item.isUnread}
    class="notification-card"
    onclick={handleCardClick}
    onkeydown={handleCardKeydown}
    role="link"
    tabindex="0"
  >
    {#if social}
      <div class="identity-row">
        {#if item.isUnread}
          <span class="unread-dot"></span>
        {/if}
        <AvatarBadge size="sm" name={actorName} avatar={null} />
        <a class="name" href={resolve('/profile/[agent]', { agent: item.actor })}>{actorName}</a>
      </div>
    {:else}
      <div class="kicker">
        {#if item.isUnread}
          <span class="unread-dot"></span>
        {/if}
        <span class="subject-label">{item.subject === 'thread' ? 'Thread' : 'Project'}</span>
      </div>
    {/if}

    <div class="copy-row">
      <div class="copy">
        {#if item.lead}
          <p class="title-text">{item.lead}</p>
          {#if item.body}
            <p class="body excerpt">{item.body}</p>
          {/if}
        {:else if social}
          <p class="body">@{actorName} {item.body}</p>
        {:else}
          <p class="title-text">{item.title}</p>
          <p class="body">{item.body}</p>
        {/if}
      </div>
    </div>

    <!-- Bottom row on every card: actions (if any) left, name · time right, never squeezed. -->
    <div class="meta-row">
      <div class="meta-actions">
        {#if item.kind === 'follow-request' && item.isUnread && onDecide}
          <button
            class="accept-button"
            disabled={deciding}
            type="button"
            onclick={() => onDecide(true)}
          >
            Accept follower
          </button>
          <button
            class="decline-button"
            disabled={deciding}
            type="button"
            onclick={() => onDecide(false)}
          >
            Decline
          </button>
        {/if}
        {#if item.isUnread}
          <button class="mark-read" type="button" onclick={onRead}>Mark read</button>
        {/if}
      </div>

      <div class="footer-meta">
        {#if social}
          <ContentMetaRow
            timeOnly
            authorName={actorName}
            authorHref={profileHref}
            createdAt={item.createdAt}
          />
        {:else}
          <ContentMetaRow
            fullName
            authorName={actorName}
            authorHref={profileHref}
            createdAt={item.createdAt}
          />
        {/if}
      </div>
    </div>
  </div>
</FeedSurface>
