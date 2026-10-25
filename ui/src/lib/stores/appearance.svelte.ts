/**
 * web's appearance settings (theme, accent, font, type size), kept per device in browser
 * storage under web's keys; `app.html` applies them before first paint, as on web.
 */
export const ACCENT_COLORS = ['green', 'blue', 'purple', 'orange', 'pink', 'teal'] as const;
export type AccentColor = (typeof ACCENT_COLORS)[number];
export type ThemeMode = 'light' | 'dark' | 'system';
export type FontFamily = 'inter' | 'system' | 'readable';
export type TextSize = 'small' | 'medium' | 'large';

export interface Appearance {
  theme: ThemeMode;
  accent: AccentColor;
  font: FontFamily;
  textSize: TextSize;
}

const KEYS = { theme: 'sp_theme', accent: 'sp_accent', font: 'sp_font', textSize: 'sp_text_size' };

function read<T extends string>(key: string, allowed: readonly T[], fallback: T): T {
  try {
    const value = localStorage.getItem(key);
    return allowed.includes(value as T) ? (value as T) : fallback;
  } catch {
    return fallback;
  }
}

export const appearance = $state<Appearance>({
  theme: read(KEYS.theme, ['light', 'dark', 'system'], 'light'),
  accent: read(KEYS.accent, ACCENT_COLORS, 'green'),
  font: read(KEYS.font, ['inter', 'system', 'readable'], 'inter'),
  textSize: read(KEYS.textSize, ['small', 'medium', 'large'], 'medium'),
});

export function setAppearance(patch: Partial<Appearance>): void {
  Object.assign(appearance, patch);
  try {
    for (const [field, key] of Object.entries(KEYS)) {
      localStorage.setItem(key, appearance[field as keyof Appearance]);
    }
  } catch {
    // Private windows: the choice holds until the page closes.
  }
}

/** web's root layout: put the choices on <html>; "system" follows the device. */
export function applyAppearance(prefersDark: boolean): void {
  const root = document.documentElement;
  const theme = appearance.theme === 'system' ? (prefersDark ? 'dark' : 'light') : appearance.theme;
  root.dataset.theme = theme;
  document.body.dataset.theme = theme;
  root.dataset.textSize = appearance.textSize;
  root.dataset.accent = appearance.accent;
  root.dataset.font = appearance.font;
  document
    .querySelector('meta[name="theme-color"]')
    ?.setAttribute('content', theme === 'light' ? '#ffffff' : '#181a1b');
}
