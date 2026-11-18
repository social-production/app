<script lang="ts">
  import { language, LANGUAGES, setLanguage, t } from '#lib/i18n/index.ts';
  import SettingChoice from '#lib/components/settings/SettingChoice.svelte';
  import SettingSwitch from '#lib/components/settings/SettingSwitch.svelte';
  import {
    ACCENT_COLORS,
    appearance,
    setAppearance,
    type FontFamily,
    type TextSize,
    type ThemeMode,
  } from '#lib/stores/appearance.svelte.ts';
  import { backButtons, setBackButtons } from '#lib/stores/backButtons.svelte.ts';

  import TimezoneSetting from './TimezoneSetting.svelte';

  /** web's Appearance section: language, type size, theme, accent colour and font, kept on this
   * device, and the Display timezone (ADR 0114 for the language). */
  const ACCENT_SWATCHES: Record<string, string> = {
    green: '#2f8a57',
    blue: '#2563eb',
    purple: '#7c3aed',
    orange: '#c2410c',
    pink: '#be185d',
    teal: '#0f766e',
  };
  const themes: { value: ThemeMode; label: string }[] = [
    { value: 'light', label: t('Light') },
    { value: 'dark', label: t('Dark') },
    { value: 'system', label: t('System') },
  ];
  const fonts: { value: FontFamily; label: string }[] = [
    { value: 'inter', label: t('Inter') },
    { value: 'system', label: t('System') },
    { value: 'readable', label: t('Readable') },
  ];
  const sizes: { value: TextSize; label: string }[] = [
    { value: 'small', label: t('Small') },
    { value: 'medium', label: t('Medium') },
    { value: 'large', label: t('Large') },
  ];
  const title = (s: string): string => s.charAt(0).toUpperCase() + s.slice(1);
</script>

<section class="settings-section" id="settings-appearance">
  <h2>{t('Appearance')}</h2>
  <div class="card stack flush">
    <SettingChoice
      label={t('Language')}
      hint={t('The app’s own words. What people write stays as they wrote it.')}
      value={language}
      options={LANGUAGES}
      onChange={setLanguage}
    />
    <SettingChoice
      label={t('Theme')}
      hint={t("System follows your device's light or dark setting.")}
      value={appearance.theme}
      options={themes}
      onChange={(theme) => setAppearance({ theme })}
    />
    <SettingChoice
      label={t('Type size')}
      hint={t('Medium is the usual size. Small and Large change body text across the app.')}
      value={appearance.textSize}
      options={sizes}
      onChange={(textSize) => setAppearance({ textSize })}
    />
    <SettingChoice
      label={t('Accent')}
      hint={t('Buttons, links and highlights. Each one stays readable in light and dark.')}
      value={appearance.accent}
      options={ACCENT_COLORS.map((a) => ({
        value: a,
        label: title(a),
        swatch: ACCENT_SWATCHES[a] ?? '',
      }))}
      onChange={(accent) => setAppearance({ accent })}
    />
    <SettingChoice
      label={t('Font')}
      hint={t('Readable is Atkinson Hyperlegible, designed for low vision.')}
      value={appearance.font}
      options={fonts}
      onChange={(font) => setAppearance({ font })}
    />
    <SettingSwitch
      label={t('Show back buttons')}
      note={t('On this device. Turn off if your phone has its own back gesture.')}
      checked={backButtons.shown}
      onToggle={() => setBackButtons(!backButtons.shown)}
    />
    <TimezoneSetting />
  </div>
</section>
