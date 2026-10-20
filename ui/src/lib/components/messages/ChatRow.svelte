<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';

  /** One row of web's Messages list: avatar, name, time, last message and unread count. */
  interface Props {
    title: string;
    preview: string;
    time: string;
    unread: number;
    onOpen: () => void;
  }

  let { title, preview, time, unread, onOpen }: Props = $props();
</script>

<button class:unread={unread > 0} class="conversation-row" type="button" onclick={onOpen}>
  <AvatarBadge size="sm" name={title} />
  <div class="conversation-copy">
    <div class="conversation-topline">
      <strong>{title}</strong>
      <span class="conversation-time">{time}</span>
    </div>
    <p class="conversation-preview">{preview}</p>
  </div>
  {#if unread > 0}<span class="unread-pill">{unread}</span>{/if}
</button>
