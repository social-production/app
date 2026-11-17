<script lang="ts">
  import { tick } from 'svelte';
  import { portal } from '#lib/components/actions/portal.ts';
  import '#lib/components/styles/map.css';

  /** web's `components/shared/RadiusCombobox.svelte`: a picked or typed radius ("25 km",
   * "Global"); the menu is fixed-positioned, flipping above near the bottom edge. Tapping the
   * box only opens the list (no keyboard on phones); typing lives in the list's last row
   * (owner, 2026-11-17: "always wants me to type"). */
  interface Props {
    value: string;
    onChange: (value: string) => void;
    ariaLabel?: string;
    options?: { value: string; label: string }[];
    portaled?: boolean;
    preferAbove?: boolean;
    compact?: boolean;
  }

  let {
    value,
    onChange,
    ariaLabel = 'Search radius',
    options = [],
    portaled = true,
    preferAbove = false,
    compact = false,
  }: Props = $props();

  let open = $state(false);
  let shell = $state<HTMLDivElement>();
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let menuStyle = $state('');
  let draft = $state('');

  const display = $derived.by(() => {
    if (value === 'global') return 'Global';
    const parsed = Number(value);
    return Number.isFinite(parsed) ? `${Math.round(parsed)} km` : value;
  });
  const custom = $derived(!options.some((o) => o.value === value));

  async function positionMenu(): Promise<void> {
    await tick();
    if (!trigger || !menu) return;
    const box = trigger.getBoundingClientRect();
    const size = menu.getBoundingClientRect();
    const gap = 4;
    const pad = 8;
    let top = preferAbove ? box.top - size.height - gap : box.bottom + gap;
    if (!preferAbove && top + size.height > window.innerHeight - pad) {
      top = box.top - size.height - gap;
    }
    const left = Math.max(pad, Math.min(box.left, window.innerWidth - size.width - pad));
    top = Math.max(pad, Math.min(top, window.innerHeight - size.height - pad));
    menuStyle = `top: ${top}px; left: ${left}px; width: ${Math.max(box.width, 180)}px;`;
  }

  function toggle(): void {
    open = !open;
    draft = custom && value !== 'global' ? value : '';
    if (open) void positionMenu();
  }

  function select(next: string): void {
    open = false;
    if (next.trim() && next !== value) onChange(next);
  }

  /** web's `commitDraft`: a whole number of km (at least 1). */
  function commit(): void {
    const parsed = Number(draft.replace(/[^\d.]/g, ''));
    if (!draft.trim() || !Number.isFinite(parsed)) return;
    select(String(Math.max(1, Math.round(parsed))));
  }

  $effect(() => {
    const outside = (event: PointerEvent): void => {
      const target = event.target as Node;
      if (open && !shell?.contains(target) && !menu?.contains(target)) open = false;
    };
    const reposition = (): void => {
      if (open) void positionMenu();
    };
    const escape = (event: KeyboardEvent): void => {
      if (open && event.key === 'Escape') open = false;
    };
    document.addEventListener('pointerdown', outside);
    document.addEventListener('keydown', escape);
    window.addEventListener('resize', reposition);
    window.addEventListener('scroll', reposition, true);
    return () => {
      document.removeEventListener('pointerdown', outside);
      document.removeEventListener('keydown', escape);
      window.removeEventListener('resize', reposition);
      window.removeEventListener('scroll', reposition, true);
    };
  });
</script>

{#snippet menuList()}
  <div bind:this={menu} class="radius-menu" class:portaled role="listbox" style={menuStyle}>
    {#each options as option (option.value)}
      <button
        type="button"
        role="option"
        aria-selected={option.value === value}
        class="radius-option"
        class:selected={option.value === value}
        onclick={() => select(option.value)}
      >
        <span>{option.label}</span>
        {#if option.value === value}
          <span aria-hidden="true" class="check">✓</span>
        {/if}
      </button>
    {/each}
    <form
      class="radius-custom"
      class:selected={custom}
      onsubmit={(e) => {
        e.preventDefault();
        commit();
      }}
    >
      <input
        type="text"
        inputmode="numeric"
        enterkeyhint="done"
        aria-label="Custom distance in km"
        placeholder="Custom"
        bind:value={draft}
      />
      <span>km</span>
      <button type="submit" disabled={!draft.trim()}>Set</button>
    </form>
  </div>
{/snippet}

<div bind:this={shell} class="radius-combobox" class:compact>
  <div class="input-shell" class:open>
    <button
      bind:this={trigger}
      class="radius-input radius-trigger"
      type="button"
      aria-label={ariaLabel}
      aria-haspopup="listbox"
      aria-expanded={open}
      onclick={toggle}
    >
      <span>{display}</span>
      <svg class="chevron" aria-hidden="true" viewBox="0 0 24 24" width="14" height="14">
        <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" stroke-width="2.4" />
      </svg>
    </button>
  </div>

  {#if open}
    {#if portaled}
      <!-- Portaled menus keep the root class so the ported styles still reach them. -->
      <div class="radius-combobox radius-portal" use:portal>{@render menuList()}</div>
    {:else}
      {@render menuList()}
    {/if}
  {/if}
</div>
