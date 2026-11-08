<script lang="ts">
  import { tick } from 'svelte';
  import { portal } from '#lib/components/actions/portal.ts';
  import '#lib/components/styles/map.css';

  /** web's `components/shared/RadiusCombobox.svelte`: a typed or picked radius ("25 km",
   * "Global"); the menu is fixed-positioned, flipping above near the bottom edge. */
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
  let input = $state<HTMLInputElement>();
  let menu = $state<HTMLDivElement>();
  let menuStyle = $state('');
  let draft = $state('');

  const display = $derived.by(() => {
    if (value === 'global') return 'Global';
    const parsed = Number(value);
    return Number.isFinite(parsed) ? `${Math.round(parsed)} km` : value;
  });

  async function positionMenu(): Promise<void> {
    await tick();
    if (!input || !menu) return;
    const trigger = input.getBoundingClientRect();
    const box = menu.getBoundingClientRect();
    const gap = 4;
    const pad = 8;
    let top = preferAbove ? trigger.top - box.height - gap : trigger.bottom + gap;
    if (!preferAbove && top + box.height > window.innerHeight - pad) {
      top = trigger.top - box.height - gap;
    }
    const left = Math.max(pad, Math.min(trigger.left, window.innerWidth - box.width - pad));
    top = Math.max(pad, Math.min(top, window.innerHeight - box.height - pad));
    menuStyle = `top: ${top}px; left: ${left}px; width: ${Math.max(trigger.width, 168)}px;`;
  }

  async function openMenu(): Promise<void> {
    if (open) return;
    open = true;
    draft = value === 'global' ? '' : value;
    await positionMenu();
    input?.select();
  }

  function select(next: string): void {
    open = false;
    if (next.trim() && next !== value) onChange(next);
  }

  /** web's `commitDraft`: "global", or a whole number of km (at least 1). */
  function commit(): void {
    const typed = draft.trim().toLowerCase();
    if (!typed) return void (open = false);
    if (typed === 'global') return select('global');
    const parsed = Number(typed.replace(/[^\d.]/g, ''));
    if (!Number.isFinite(parsed)) return void (open = false);
    select(String(Math.max(1, Math.round(parsed))));
  }

  function onKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      event.preventDefault();
      commit();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      open = false;
    }
  }

  $effect(() => {
    const outside = (event: MouseEvent): void => {
      const target = event.target as Node;
      if (open && !shell?.contains(target) && !menu?.contains(target)) open = false;
    };
    const reposition = (): void => {
      if (open) void positionMenu();
    };
    document.addEventListener('click', outside);
    window.addEventListener('resize', reposition);
    window.addEventListener('scroll', reposition, true);
    return () => {
      document.removeEventListener('click', outside);
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
        onmousedown={(e) => e.preventDefault()}
        onclick={() => select(option.value)}
      >
        <span>{option.label}</span>
        {#if option.value === value}
          <span aria-hidden="true" class="check">✓</span>
        {/if}
      </button>
    {/each}
  </div>
{/snippet}

<div bind:this={shell} class="radius-combobox" class:compact>
  <div class="input-shell" class:open>
    <input
      bind:this={input}
      class="radius-input"
      type="text"
      inputmode="numeric"
      aria-label={ariaLabel}
      aria-haspopup="listbox"
      value={open ? draft : display}
      onfocus={() => void openMenu()}
      oninput={(event) => {
        open = true;
        draft = event.currentTarget.value;
        void positionMenu();
      }}
      onkeydown={onKeydown}
      onblur={() => window.setTimeout(() => open && commit(), 120)}
    />
    <button
      class="toggle-button"
      type="button"
      aria-label="Open radius options"
      onmousedown={(e) => e.preventDefault()}
      onclick={(e) => {
        e.stopPropagation();
        if (open) open = false;
        else void openMenu();
      }}
    >
      ▾
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
