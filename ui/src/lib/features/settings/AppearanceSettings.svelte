<script lang="ts">
  import {
    ACCENT_COLORS,
    appearance,
    setAppearance,
    type FontFamily,
    type TextSize,
    type ThemeMode,
  } from '#lib/stores/appearance.svelte.ts';

  /** web's Appearance section: type size, theme, accent colour and font, kept on this device.
   * web's language (coming soon), feed split and timezone rows wait for those features. */
  const ACCENT_SWATCHES: Record<string, string> = {
    green: '#2f8a57',
    blue: '#2563eb',
    purple: '#7c3aed',
    orange: '#c2410c',
    pink: '#be185d',
    teal: '#0f766e',
  };
  const themes: { value: ThemeMode; label: string }[] = [
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
    { value: 'system', label: 'System' },
  ];
  const fonts: { value: FontFamily; label: string }[] = [
    { value: 'inter', label: 'Inter' },
    { value: 'system', label: 'System' },
    { value: 'readable', label: 'Readable' },
  ];
  const title = (s: string): string => s.charAt(0).toUpperCase() + s.slice(1);
</script>

<section class="settings-section" id="settings-appearance">
  <h2>Appearance</h2>
  <div class="card stack flush">
    <div class="setting-item">
      <div>
        <strong>Type size</strong>
        <p>Medium is the usual size. Small and Large change body text across the app.</p>
      </div>
      <label class="language-field">
        <span class="sr-only">Type size</span>
        <select
          class="language-select"
          value={appearance.textSize}
          onchange={(e) => setAppearance({ textSize: e.currentTarget.value as TextSize })}
        >
          <option value="small">Small</option>
          <option value="medium">Medium</option>
          <option value="large">Large</option>
        </select>
      </label>
    </div>
    <div class="setting-item">
      <div>
        <strong>Theme</strong>
        <p>System follows your device's light or dark setting.</p>
      </div>
      <div class="theme-choice" role="radiogroup" aria-label="Theme">
        {#each themes as option (option.value)}
          <button
            role="radio"
            aria-checked={appearance.theme === option.value}
            class:chosen={appearance.theme === option.value}
            type="button"
            onclick={() => setAppearance({ theme: option.value })}>{option.label}</button
          >
        {/each}
      </div>
    </div>
    <div class="setting-item">
      <div>
        <strong>Accent colour</strong>
        <p>Buttons, links and highlights. Each one stays readable in light and dark.</p>
      </div>
      <div class="accent-choice" role="radiogroup" aria-label="Accent colour">
        {#each ACCENT_COLORS as accent (accent)}
          <button
            role="radio"
            aria-checked={appearance.accent === accent}
            aria-label={title(accent)}
            title={title(accent)}
            class="swatch"
            class:chosen={appearance.accent === accent}
            style={`--swatch: ${ACCENT_SWATCHES[accent]}`}
            type="button"
            onclick={() => setAppearance({ accent })}
          ></button>
        {/each}
      </div>
    </div>
    <div class="setting-item">
      <div>
        <strong>Font</strong>
        <p>Readable is Atkinson Hyperlegible, designed for low vision.</p>
      </div>
      <div class="theme-choice" role="radiogroup" aria-label="Font">
        {#each fonts as option (option.value)}
          <button
            role="radio"
            aria-checked={appearance.font === option.value}
            class:chosen={appearance.font === option.value}
            type="button"
            onclick={() => setAppearance({ font: option.value })}>{option.label}</button
          >
        {/each}
      </div>
    </div>
  </div>
</section>
