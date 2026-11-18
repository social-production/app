<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import { language, LANGUAGES, setLanguage, t } from '#lib/i18n/index.ts';

  import './language-picker.css';

  /**
   * The language picker for people who may not be signed in (ADR 0114): a row (the More
   * menu) or a link (the sign-in page) with the current language, opening the same list as
   * Settings › Appearance › Language: every language in its own words, so it grows as
   * languages are added. Choosing one keeps it on this device and reloads the app in it.
   */
  let { variant = 'row', onOpen }: { variant?: 'row' | 'link'; onOpen?: () => void } = $props();

  let open = $state(false);
  const current = $derived(LANGUAGES.find((l) => l.value === language) ?? LANGUAGES[0]);

  function show(): void {
    onOpen?.();
    open = true;
  }
</script>

<button
  class={variant === 'row' ? 'language-row float-panel-item' : 'language-link'}
  type="button"
  aria-haspopup="dialog"
  onclick={show}
>
  <ToolbarIcon name="globe" />
  <span class="language-name">{t('Language')}</span>
  <span class="language-current" lang={current?.value}>{current?.label}</span>
</button>

<OverlaySheet
  {open}
  title={t('Language')}
  labelledById="language-picker-title"
  rootClass="setting-sheet"
  onClose={() => (open = false)}
>
  <p class="setting-hint">{t('The app’s own words. What people write stays as they wrote it.')}</p>
  <div class="setting-options" role="radiogroup" aria-label={t('Language')}>
    {#each LANGUAGES as option (option.value)}
      <button
        type="button"
        role="radio"
        lang={option.value}
        aria-checked={option.value === language}
        class="setting-option"
        class:chosen={option.value === language}
        onclick={() => setLanguage(option.value)}
      >
        <span>{option.label}</span>
        {#if option.value === language}
          <svg aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
            <path
              d="M5 12.5 10 17.5 19 7.5"
              fill="none"
              stroke="currentColor"
              stroke-width="2.2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        {/if}
      </button>
    {/each}
  </div>
</OverlaySheet>
