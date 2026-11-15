<script lang="ts">
  import SettingChoice from '#lib/components/settings/SettingChoice.svelte';
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
  const sizes: { value: TextSize; label: string }[] = [
    { value: 'small', label: 'Small' },
    { value: 'medium', label: 'Medium' },
    { value: 'large', label: 'Large' },
  ];
  const title = (s: string): string => s.charAt(0).toUpperCase() + s.slice(1);
</script>

<section class="settings-section" id="settings-appearance">
  <h2>Appearance</h2>
  <div class="card stack flush">
    <SettingChoice
      label="Theme"
      hint="System follows your device's light or dark setting."
      value={appearance.theme}
      options={themes}
      onChange={(theme) => setAppearance({ theme })}
    />
    <SettingChoice
      label="Type size"
      hint="Medium is the usual size. Small and Large change body text across the app."
      value={appearance.textSize}
      options={sizes}
      onChange={(textSize) => setAppearance({ textSize })}
    />
    <SettingChoice
      label="Accent"
      hint="Buttons, links and highlights. Each one stays readable in light and dark."
      value={appearance.accent}
      options={ACCENT_COLORS.map((a) => ({
        value: a,
        label: title(a),
        swatch: ACCENT_SWATCHES[a] ?? '',
      }))}
      onChange={(accent) => setAppearance({ accent })}
    />
    <SettingChoice
      label="Font"
      hint="Readable is Atkinson Hyperlegible, designed for low vision."
      value={appearance.font}
      options={fonts}
      onChange={(font) => setAppearance({ font })}
    />
  </div>
</section>
