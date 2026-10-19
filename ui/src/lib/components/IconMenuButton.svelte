<script lang="ts" generics="V extends string">
  import { tick, type Snippet } from 'svelte';

  import '#lib/components/styles/icon-menu.css';

  /**
   * web's IconMenuButton: an icon trigger opening a small radio menu. The menu is fixed-
   * positioned under the trigger (flipping above near the bottom edge). One open at a time.
   */
  interface Props {
    value: V;
    ariaLabel: string;
    options: { value: V; label: string }[];
    /** The "no filter" value; any other value marks the trigger active. */
    defaultValue?: V;
    showTriggerLabel?: boolean;
    onChange: (value: V) => void;
    children: Snippet;
  }

  let {
    value,
    ariaLabel,
    options,
    defaultValue,
    showTriggerLabel = false,
    onChange,
    children,
  }: Props = $props();

  let open = $state(false);
  let shell = $state<HTMLDivElement>();
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let menuStyle = $state('');

  const baseline = $derived(defaultValue ?? options[0]?.value);
  const triggerLabel = $derived(options.find((o) => o.value === value)?.label ?? ariaLabel);

  async function place(): Promise<void> {
    await tick();
    if (!trigger || !menu) return;
    const t = trigger.getBoundingClientRect();
    const m = menu.getBoundingClientRect();
    const pad = 8;
    let top = t.bottom + 4;
    if (top + m.height > window.innerHeight - pad) top = t.top - m.height - 4;
    const left = Math.max(pad, Math.min(t.right - m.width, window.innerWidth - m.width - pad));
    menuStyle = `top: ${Math.max(pad, top)}px; left: ${left}px;`;
  }

  async function toggle(event: MouseEvent): Promise<void> {
    event.stopPropagation();
    open = !open;
    if (open) await place();
  }

  function choose(next: V): void {
    open = false;
    onChange(next);
  }

  $effect(() => {
    if (!open) return;
    const outside = (e: MouseEvent): void => {
      const target = e.target as Node;
      if (!shell?.contains(target) && !menu?.contains(target)) open = false;
    };
    const escape = (e: KeyboardEvent): void => {
      if (e.key === 'Escape') open = false;
    };
    const reflow = (): void => void place();
    document.addEventListener('click', outside);
    window.addEventListener('keydown', escape);
    window.addEventListener('resize', reflow);
    window.addEventListener('scroll', reflow, true);
    return () => {
      document.removeEventListener('click', outside);
      window.removeEventListener('keydown', escape);
      window.removeEventListener('resize', reflow);
      window.removeEventListener('scroll', reflow, true);
    };
  });
</script>

<div bind:this={shell} class="icon-menu-shell">
  <button
    bind:this={trigger}
    type="button"
    class="icon-menu-trigger"
    class:has-label={showTriggerLabel}
    class:menu-open={open}
    class:menu-active={value !== baseline}
    aria-label={ariaLabel}
    title={ariaLabel}
    aria-expanded={open}
    aria-haspopup="menu"
    onclick={toggle}
  >
    {@render children()}
    {#if showTriggerLabel}<span class="trigger-label">{triggerLabel}</span>{/if}
  </button>

  {#if open}
    <div bind:this={menu} class="icon-menu" role="menu" style={menuStyle}>
      {#each options as option (option.value)}
        <button
          type="button"
          role="menuitemradio"
          aria-checked={option.value === value}
          class="icon-menu-item"
          class:selected={option.value === value}
          onclick={() => choose(option.value)}
        >
          <span class="item-label">{option.label}</span>
          {#if option.value === value}<span aria-hidden="true" class="check">✓</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</div>
