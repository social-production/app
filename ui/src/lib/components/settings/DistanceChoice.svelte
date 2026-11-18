<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import '#lib/components/styles/ux/regionsettings.css';

  /** Owner's fix 3d (2026-11-17): preset distances you tap, and a box below them you can type
   * any distance into (km). `value` is a radius as the Region feed keeps it ('25', 'global'). */
  interface Props {
    value: string;
    presets: { value: string; label: string }[];
    onChange: (value: string) => void;
  }

  let { value, presets, onChange }: Props = $props();

  const typed = $derived(/^\d+$/.test(value) ? value : '');
</script>

<div class="distance-choice">
  <p class="distance-label">{t("Show what's within")}</p>
  <div class="distance-chips">
    {#each presets as preset (preset.value)}
      <button
        type="button"
        class:active={preset.value === value}
        aria-pressed={preset.value === value}
        onclick={() => onChange(preset.value)}>{preset.label}</button
      >
    {/each}
  </div>
  <label class="distance-typed">
    <span>{t('Or type a distance')}</span>
    <span class="distance-box">
      <input
        aria-label={t('Distance in km')}
        inputmode="numeric"
        min="1"
        type="number"
        value={typed}
        oninput={(e) => {
          const n = Math.round(Number(e.currentTarget.value));
          if (n >= 1) onChange(String(n));
        }}
      />
      <em>{t('km')}</em>
    </span>
  </label>
</div>
