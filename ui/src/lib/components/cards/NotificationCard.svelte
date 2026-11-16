<script lang="ts">
  import { resolve } from '$app/paths';
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import type { NotificationItem } from '#lib/domain/notifications.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
  import { relativeCompact } from '#lib/domain/time.ts';

  import '#lib/components/styles/notifications.css';

  /** web's inbox NotificationCard as drawn (Remaining Screens §2): a round type icon (a face
   * for people), one bold line, the time at the right and a green dot while unread; tapping
   * the dot marks it read, tapping the row opens it. */
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
  const subject = $derived(
    item.subject === 'account'
      ? 'Account'
      : item.subject === 'thread' || item.projectMode === 'thread'
        ? 'Thread'
        : item.projectMode === 'post'
          ? 'Post'
          : item.projectMode === 'event'
            ? 'Event'
            : item.projectMode === 'help-request'
              ? 'Help request'
              : 'Project',
  );
  const icon = $derived<SurfaceIconId>(
    subject === 'Thread'
      ? 'bubble'
      : subject === 'Post'
        ? 'post'
        : subject === 'Event'
          ? 'event'
          : subject === 'Help request'
            ? 'help-request'
            : subject === 'Account'
              ? 'user'
              : 'cube',
  );
  const accent = $derived(item.projectMode ? modeAccent(item.projectMode) : 'var(--brand)');

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

<div
  class:unread={item.isUnread}
  class:social
  class="notification-card notification-row"
  onclick={handleCardClick}
  onkeydown={handleCardKeydown}
  role="link"
  tabindex="0"
>
  {#if social}
    <a
      class="note-icon"
      href={resolve('/profile/[agent]', { agent: item.actor })}
      aria-label={actorName}><AvatarBadge size="md" name={actorName} avatar={null} /></a
    >
  {:else}
    <span class="note-icon" style={`--note-accent: ${accent}`} aria-hidden="true">
      <SurfaceIcon {icon} size="md" />
    </span>
  {/if}
  <div class="note-copy">
    <p class="title-text">
      {#if item.lead}{item.lead}{:else if social}<a
          href={resolve('/profile/[agent]', { agent: item.actor })}>{actorName}</a
        >
        {item.body}{:else}{item.title}{/if}
    </p>
    {#if item.lead && item.body}
      <p class="body excerpt">{item.body}</p>
    {:else if !item.lead && !social && item.body}
      <p class="body">{item.body}</p>
    {/if}
    {#if !social && item.actor}<p class="note-by">{subject} · {actorName}</p>{/if}
    {#if item.kind === 'follow-request' && item.isUnread && onDecide}
      <div class="note-actions">
        <button
          class="accept-button"
          disabled={deciding}
          type="button"
          onclick={() => onDecide(true)}>Accept follower</button
        >
        <button
          class="decline-button"
          disabled={deciding}
          type="button"
          onclick={() => onDecide(false)}>Decline</button
        >
      </div>
    {/if}
  </div>
  <div class="note-end">
    <span class="note-time">{relativeCompact(item.createdAt)}</span>
    {#if item.isUnread}
      <button
        class="unread-dot"
        type="button"
        aria-label="Mark read"
        title="Mark read"
        onclick={onRead}
      ></button>
    {/if}
  </div>
</div>
