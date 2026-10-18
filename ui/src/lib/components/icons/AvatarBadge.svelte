<script lang="ts">
  import type { Avatar } from '#lib/domain/people.ts';

  /** Ported from web's AvatarBadge: the picture, or initials when there is none. */
  interface Props {
    name: string;
    avatar?: Avatar | null;
    size?: 'sm' | 'md';
  }

  let { name, avatar = null, size = 'md' }: Props = $props();

  const initials = $derived(
    name
      .replace(/[^a-z0-9]/gi, '')
      .slice(0, 2)
      .toUpperCase() || 'SP',
  );
  const url = $derived(
    avatar ? URL.createObjectURL(new Blob([avatar.bytes.slice()], { type: avatar.mime })) : null,
  );
  $effect(() => () => {
    if (url) URL.revokeObjectURL(url);
  });
</script>

<span class:size-sm={size === 'sm'} class:size-md={size === 'md'} class="avatar-badge">
  {#if url}
    <img alt="" class="avatar-image" src={url} />
  {:else}
    {initials}
  {/if}
</span>

<style>
  .avatar-badge {
    display: inline-grid;
    place-items: center;
    overflow: hidden;
    border-radius: 999px;
    background: linear-gradient(135deg, var(--brand-soft), var(--brand-badge));
    color: var(--brand-strong);
    font-weight: 800;
    letter-spacing: 0.04em;
    flex-shrink: 0;
  }

  .size-sm {
    width: 34px;
    height: 34px;
    font-size: 11px;
  }

  .size-md {
    width: 42px;
    height: 42px;
    font-size: 13px;
  }

  .avatar-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
</style>
