<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SettingRow from '#lib/components/settings/SettingRow.svelte';
  import { allZones, deviceZone } from '#lib/domain/zone.ts';
  import { chooseDisplayZone, displayZoneChoice } from '#lib/stores/displayZone.svelte.ts';

  /** web's Display timezone: activity, event and request times show (and are typed) in it.
   * A row in Appearance; the sheet filters every zone the browser knows. */
  const SHOWN = 60;
  let open = $state(false);
  let query = $state('');
  const zones = allZones();
  const label = (zone: string): string => zone.replace(/_/g, ' ').replace(/\//g, ' / ');
  const matches = $derived.by(() => {
    const q = query.trim().toLowerCase().replace(/\s+/g, '_');
    return (q ? zones.filter((z) => z.toLowerCase().includes(q)) : zones).slice(0, SHOWN);
  });

  function offset(zone: string): string {
    const part = new Intl.DateTimeFormat('en-US', { timeZone: zone, timeZoneName: 'shortOffset' })
      .formatToParts(new Date())
      .find((p) => p.type === 'timeZoneName')?.value;
    return part === 'GMT' ? 'GMT+0' : (part ?? '');
  }

  function choose(zone: string | null): void {
    chooseDisplayZone(zone);
    open = false;
    query = '';
  }
</script>

<SettingRow
  label={t('Display timezone')}
  value={displayZoneChoice.zone ? label(displayZoneChoice.zone) : t('This device')}
  onOpen={() => (open = true)}
/>

<OverlaySheet
  {open}
  title={t('Display timezone')}
  labelledById="setting-timezone"
  rootClass="setting-sheet timezone-sheet"
  onClose={() => (open = false)}
>
  <p class="setting-hint">
    {t('Activity, event and request times show in it, and you type them in it.')}
  </p>
  <input
    class="timezone-search"
    bind:value={query}
    aria-label={t('Filter timezones')}
    placeholder={t('Filter timezones')}
    type="search"
  />
  <ul class="timezone-list" aria-label={t('Timezones')}>
    {#if !query.trim()}
      <li>
        <button
          class="timezone-row"
          class:chosen={!displayZoneChoice.zone}
          aria-pressed={!displayZoneChoice.zone}
          type="button"
          onclick={() => choose(null)}
        >
          <span>{t("This device's")}<small>{t(label(deviceZone()))}</small></span>
          <em>{offset(deviceZone())}</em>
        </button>
      </li>
    {/if}
    {#each matches as zone (zone)}
      <li>
        <button
          class="timezone-row"
          class:chosen={displayZoneChoice.zone === zone}
          aria-pressed={displayZoneChoice.zone === zone}
          type="button"
          onclick={() => choose(zone)}
        >
          <span>{t(label(zone))}</span>
          <em>{offset(zone)}</em>
        </button>
      </li>
    {:else}
      <li class="setting-hint">No timezone matches "{query}".</li>
    {/each}
  </ul>
</OverlaySheet>
