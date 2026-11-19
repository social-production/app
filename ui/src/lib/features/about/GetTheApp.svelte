<script lang="ts">
  import { t } from '#lib/i18n/index.ts';

  import { appOptions, detectPlatform, ownFirst } from './getTheApp.ts';

  import '#lib/components/styles/ux/getapp.css';

  /** "Get the app" on the About page (ADR 0115): desktop downloads and how to install
   * on a phone, the visitor's own device first. */
  const own =
    typeof navigator === 'undefined'
      ? null
      : detectPlatform(navigator.userAgent, navigator.maxTouchPoints);
  const options = ownFirst(appOptions(), own);
  const installed =
    typeof window !== 'undefined' && window.matchMedia?.('(display-mode: standalone)').matches;
</script>

<section class="get-app" aria-labelledby="get-app-title">
  <h3 id="get-app-title">{t('Get the app')}</h3>
  {#if installed}
    <p class="get-app-lead">{t("You're using the installed app.")}</p>
  {/if}
  <ul>
    {#each options as option (option.id)}
      <li class:own={option.id === own}>
        <div>
          <strong>{option.name}</strong>
          {#if option.id === own}<span class="get-app-tag">{t('This device')}</span>{/if}
          <span class="get-app-note">{option.note}</span>
        </div>
        {#if option.href}
          <a class="get-app-download" href={option.href} rel="external noreferrer" download>
            {t('Download')}
          </a>
        {/if}
      </li>
    {/each}
  </ul>
  <p class="get-app-fine">
    {t('Desktop apps are unsigned previews, so your computer may warn before opening them.')}
  </p>
</section>
