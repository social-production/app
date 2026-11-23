<script lang="ts">
  import type { Avatar } from '#lib/domain/people.ts';
  import { avatarSource } from '#lib/components/people/avatarSource.ts';

  /** Ported from web's AvatarBadge: the picture, or initials when there is none. */
  interface Props {
    name: string;
    avatar?: Avatar | null;
    /** Whose face it is: their photo is looked up when no `avatar` is given. */
    agent?: string | null;
    size?: 'xs' | 'sm' | 'md';
  }

  let { name, avatar = null, agent = null, size = 'md' }: Props = $props();

  const source = avatarSource();
  const photo = $derived(avatar ?? (agent && source ? (source.of(agent) ?? null) : null));

  const initials = $derived(
    name
      .replace(/[^a-z0-9]/gi, '')
      .slice(0, 2)
      .toUpperCase() || 'SP',
  );
  const url = $derived(
    photo ? URL.createObjectURL(new Blob([photo.bytes.slice()], { type: photo.mime })) : null,
  );
  $effect(() => () => {
    if (url) URL.revokeObjectURL(url);
  });
</script>

<span
  class:size-xs={size === 'xs'}
  class:size-sm={size === 'sm'}
  class:size-md={size === 'md'}
  class="avatar-badge"
>
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

  .size-xs {
    width: 18px;
    height: 18px;
    font-size: 7px;
    letter-spacing: 0;
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
