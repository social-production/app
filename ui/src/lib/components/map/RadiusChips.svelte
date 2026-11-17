<script lang="ts">
  /** The distance as one-tap chips, for sheets with room for them (the Region filter sheet;
   * owner, 2026-11-17), and a "km" box under them for any other distance. */
  interface Props {
    value: string;
    onChange: (value: string) => void;
    options: { value: string; label: string }[];
    ariaLabel?: string;
  }

  let { value, onChange, options, ariaLabel = 'Distance' }: Props = $props();

  const custom = $derived(!options.some((o) => o.value === value));
  let draft = $state('');
  let editing = $state(false);
  const shown = $derived(editing ? draft : custom ? value : '');

  function commit(): void {
    editing = false;
    const parsed = Number(draft.replace(/[^\d.]/g, ''));
    if (!draft.trim() || !Number.isFinite(parsed)) return;
    const next = String(Math.max(1, Math.round(parsed)));
    if (next !== value) onChange(next);
  }
</script>

<div class="radius-chips" role="radiogroup" aria-label={ariaLabel}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      class:on={option.value === value}
      onclick={() => option.value !== value && onChange(option.value)}>{option.label}</button
    >
  {/each}
</div>
<form
  class="radius-other"
  class:on={custom}
  onsubmit={(e) => {
    e.preventDefault();
    commit();
    (e.currentTarget.querySelector('input') as HTMLInputElement | null)?.blur();
  }}
>
  <span>Other distance</span>
  <input
    type="text"
    inputmode="numeric"
    enterkeyhint="done"
    aria-label="Other distance in km"
    placeholder="—"
    value={shown}
    onfocus={() => {
      draft = custom ? value : '';
      editing = true;
    }}
    oninput={(e) => (draft = e.currentTarget.value)}
    onblur={commit}
  />
  <span>km</span>
</form>
