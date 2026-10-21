<script module lang="ts">
  /** One entry of web's chat menu: a link or an action; `danger` ones sit below a divider. */
  export interface ChatMenuItem {
    label: string;
    href?: string;
    danger?: boolean;
    onSelect?: () => void;
  }
</script>

<script lang="ts">
  import { tick } from 'svelte';

  /**
   * web's chat-row-menu: opened from a list row (right-click, long press) or the chat
   * header, at the pointer, kept on screen (196 × 220 as on web); a press outside closes it.
   */
  interface Props {
    x: number;
    y: number;
    items: ChatMenuItem[];
    onClose: () => void;
  }

  let { x, y, items, onClose }: Props = $props();

  const left = $derived(Math.max(8, Math.min(x, window.innerWidth - 196)));
  const top = $derived(Math.max(8, Math.min(y, window.innerHeight - 220)));
  const plain = $derived(items.filter((i) => !i.danger));
  const danger = $derived(items.filter((i) => i.danger));

  $effect(() => {
    const outside = (event: PointerEvent): void => {
      if (event.target instanceof Element && event.target.closest('.chat-row-menu')) return;
      onClose();
    };
    // Wait a tick: the press that opened the menu must not close it.
    let armed = true;
    void tick().then(() => armed && window.addEventListener('pointerdown', outside));
    return () => {
      armed = false;
      window.removeEventListener('pointerdown', outside);
    };
  });

  function select(item: ChatMenuItem): void {
    onClose();
    item.onSelect?.();
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if (event.key === 'Escape') onClose();
  }}
/>

{#snippet entry(item: ChatMenuItem)}
  {#if item.href}
    <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- callers resolve it -->
    <a role="menuitem" href={item.href} onclick={onClose}>{item.label}</a>
  {:else}
    <button
      class:menu-danger={item.danger}
      type="button"
      role="menuitem"
      onclick={() => select(item)}>{item.label}</button
    >
  {/if}
{/snippet}

<div class="chat-row-menu" role="menu" style="left: {left}px; top: {top}px">
  {#each plain as item (item.label)}{@render entry(item)}{/each}
  {#if danger.length > 0}
    <!-- Kept apart: leaving can't be undone by you. -->
    <hr class="menu-divider" />
    {#each danger as item (item.label)}{@render entry(item)}{/each}
  {/if}
</div>
