<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import type { Charter } from '#lib/domain/projects.ts';

  import { cardData, saveFile } from './cardData.ts';
  import type { CardData } from './cardParts.ts';
  import { FORMATS, cardFile, drawFormat, type CardFormat } from './formats.ts';

  import '#lib/components/styles/ux/qrcard.css';

  /**
   * Download a QR card (QR Share Cards mockup, owner 2026-11-24): pick a format, see it as it
   * will be saved, then download it (on phones, the share sheet's Save Image). QR code only
   * also saves as SVG.
   */
  let { id, charter, onClose }: { id: string; charter: Charter; onClose: () => void } = $props();

  let format = $state<CardFormat>('square');
  let data = $state<CardData | null>(null);
  let preview = $state('');
  let busy = $state(false);
  let error = $state('');

  $effect(() => {
    void cardData(id, charter).then(
      (d) => (data = d),
      () => (error = t('The card could not be drawn.')),
    );
  });

  $effect(() => {
    if (!data) return;
    preview = drawFormat(format, data).toDataURL('image/png');
  });

  const info = $derived(FORMATS.find((f) => f.id === format) ?? FORMATS[0]);

  async function download(svg = false): Promise<void> {
    if (!data || busy) return;
    busy = true;
    error = '';
    try {
      const { blob, name } = await cardFile(format, data, svg);
      await saveFile(blob, name);
    } catch {
      error = t('The card could not be saved. Try again.');
    } finally {
      busy = false;
    }
  }
</script>

<OverlaySheet open title={t('Download a QR card')} rootClass="qr-card-sheet" {onClose}>
  <div class="qr-preview" class:tall={format === 'story' || format === 'flyer'}>
    {#if preview}
      <img
        src={preview}
        alt={t('{format} for {title}', { format: t(info?.label ?? ''), title: charter.title })}
      />
    {:else}
      <span class="qr-preview-wait">{error || t('Drawing the card…')}</span>
    {/if}
  </div>
  <div class="qr-formats" role="group" aria-label={t('Format')}>
    {#each FORMATS as f (f.id)}
      <button type="button" aria-pressed={format === f.id} onclick={() => (format = f.id)}>
        <strong>{t(f.label)}</strong>
        <span>{t(f.detail)}</span>
      </button>
    {/each}
  </div>
  {#if format === 'qr'}
    <button
      class="qr-svg-link"
      type="button"
      disabled={busy || !data}
      onclick={() => void download(true)}>{t('Download as SVG instead')}</button
    >
  {/if}
  {#if error && preview}<p class="qr-error" role="alert">{error}</p>{/if}
  {#snippet footer()}
    <div class="sheet-actions">
      <button
        class="sheet-submit"
        type="button"
        disabled={busy || !data}
        onclick={() => void download()}
      >
        {busy ? t('Saving…') : t('Download {file}', { file: info?.file ?? 'PNG' })}
      </button>
    </div>
  {/snippet}
</OverlaySheet>
