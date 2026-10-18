<script lang="ts">
  import type { Avatar } from '#lib/domain/people.ts';

  interface Props {
    avatar: Avatar | null;
    name: string;
    size?: number;
  }

  let { avatar, name, size = 40 }: Props = $props();

  const url = $derived(
    avatar ? URL.createObjectURL(new Blob([avatar.bytes.slice()], { type: avatar.mime })) : null,
  );
  $effect(() => () => {
    if (url) URL.revokeObjectURL(url);
  });
  const initial = $derived(name.trim().charAt(0).toUpperCase() || '?');
</script>

{#if url}
  <img src={url} alt="" width={size} height={size} />
{:else}
  <span class="initial" style:width="{size}px" style:height="{size}px" aria-hidden="true"
    >{initial}</span
  >
{/if}

<style>
  img,
  .initial {
    border-radius: 50%;
    flex: none;
    object-fit: cover;
  }
  .initial {
    display: inline-grid;
    place-items: center;
    background: var(--brand-soft);
    color: var(--brand-strong);
    font-weight: 600;
  }
</style>
