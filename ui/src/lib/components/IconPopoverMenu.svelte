<script lang="ts">
  import { tick, type Snippet } from 'svelte';

  /** web's IconPopoverMenu: an icon button that opens a small panel beside it. */
  interface Props {
    open: boolean;
    ariaLabel: string;
    active?: boolean;
    disabled?: boolean;
    menuLabel?: string;
    minWidth?: number;
    chip?: boolean;
    icon: Snippet;
    children: Snippet;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open,
    ariaLabel,
    active = false,
    disabled = false,
    menuLabel = '',
    minWidth = 168,
    chip = false,
    icon,
    children,
    onOpenChange,
  }: Props = $props();

  let shell = $state<HTMLDivElement>();
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let menuStyle = $state('');

  async function position(): Promise<void> {
    await tick();
    if (!trigger || !menu) return;
    const t = trigger.getBoundingClientRect();
    const m = menu.getBoundingClientRect();
    const gap = 4;
    const pad = 8;
    // Place it by the width it will have once min-width applies (capped to the screen).
    const min = Math.min(minWidth, window.innerWidth - pad * 2);
    const width = Math.max(m.width, min);
    let top = t.bottom + gap;
    let left = t.right - width;
    if (top + m.height > window.innerHeight - pad) top = t.top - m.height - gap;
    left = Math.max(pad, Math.min(left, window.innerWidth - width - pad));
    top = Math.max(pad, Math.min(top, window.innerHeight - m.height - pad));
    menuStyle = `top: ${top}px; left: ${left}px; min-width: ${min}px;`;
  }

  $effect(() => {
    if (open && menu) void position();
  });

  function toggle(event: MouseEvent): void {
    event.stopPropagation();
    if (!disabled) onOpenChange(!open);
  }

  function onDocumentClick(event: MouseEvent): void {
    const target = event.target as Node | null;
    // A click that removed its own element (e.g. picking a suggestion) isn't an outside click.
    if (target && !target.isConnected) return;
    if (open && shell && !shell.contains(target)) onOpenChange(false);
  }

  function onViewport(): void {
    if (open) void position();
  }
</script>

<svelte:window
  onkeydown={(e) => open && e.key === 'Escape' && onOpenChange(false)}
  onresize={onViewport}
/>
<svelte:document onclick={onDocumentClick} onscrollcapture={onViewport} />

<div bind:this={shell} class="icon-popover-shell">
  <button
    bind:this={trigger}
    type="button"
    class="icon-popover-trigger"
    class:chip
    class:menu-open={open}
    class:menu-active={active}
    aria-label={ariaLabel}
    title={ariaLabel}
    aria-expanded={open}
    aria-haspopup="menu"
    {disabled}
    onclick={toggle}
  >
    {@render icon()}
  </button>

  {#if open}
    <div
      bind:this={menu}
      class="icon-popover-menu"
      role="menu"
      aria-label={menuLabel || ariaLabel}
      style={menuStyle}
    >
      {@render children()}
    </div>
  {/if}
</div>
