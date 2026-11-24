<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
  import ImageCropper from '#lib/components/overlay/ImageCropper.svelte';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import type { ScopeArt } from './scopeArt.svelte.ts';

  /**
   * The top of a scope's Settings (Channel Pictures mockups §2): the cover with "Change cover"
   * and the picture with a camera, as in Edit profile. A new one shows "New" until it is saved
   * or voted on (ADR 0109). A chosen image opens the crop screen first, and Adjust re-crops the
   * one shown (owner, 2026-11-24).
   */
  interface Props {
    kind: ScopeKind | 'space';
    /** A project's or event's tile while it has no picture (ADR 0117). */
    icon?: SurfaceIconId;
    art: ScopeArt;
    canEdit: boolean;
  }

  let { kind, icon = 'project', art, canEdit }: Props = $props();

  let coverInput = $state<HTMLInputElement>();
  let pictureInput = $state<HTMLInputElement>();

  const cover = $derived(art.newCover?.url ?? art.coverUrl);
  const picture = $derived(art.newPicture?.url ?? art.pictureUrl);

  let cropping = $state<{ slot: 'picture' | 'cover'; src: string; owned: boolean } | null>(null);

  function pick(slot: 'picture' | 'cover', input: HTMLInputElement | undefined): void {
    const file = input?.files?.[0];
    if (file) open(slot, URL.createObjectURL(file), true);
    if (input) input.value = '';
  }

  function open(slot: 'picture' | 'cover', src: string, owned = false): void {
    close();
    cropping = { slot, src, owned };
  }

  function close(): void {
    if (cropping?.owned) URL.revokeObjectURL(cropping.src);
    cropping = null;
  }

  function cropped(file: File): void {
    const slot = cropping?.slot;
    close();
    if (slot) void art.stage(slot, file);
  }
</script>

<section class="scope-sheet-art">
  <div class="scope-cover" style={cover ? `background-image: url(${cover})` : undefined}>
    {#if art.newCover}<span class="art-new">{t('New')}</span>{/if}
    {#if canEdit}
      <button
        class="art-change"
        type="button"
        disabled={art.busy}
        onclick={() => coverInput?.click()}
      >
        <ToolbarIcon name="image" />
        {t('Change cover')}
      </button>
      {#if cover}
        <button
          class="art-change art-adjust"
          type="button"
          disabled={art.busy}
          onclick={() => open('cover', cover)}>{t('Adjust')}</button
        >
      {/if}
    {/if}
  </div>
  <span class="scope-avatar">
    {#if picture && canEdit}<button
        class="art-adjust-picture"
        type="button"
        aria-label={t('Adjust picture')}
        onclick={() => open('picture', picture)}><img src={picture} alt="" /></button
      >{:else if picture}<img
        src={picture}
        alt=""
      />{:else if kind === 'channel'}#{:else}<SurfaceIcon
        icon={kind === 'space' ? icon : kind}
        size="md"
      />{/if}
    {#if art.newPicture}<span class="art-new">{t('New')}</span>{/if}
  </span>
  {#if canEdit}
    <button
      class="art-camera"
      type="button"
      aria-label={t('Change picture')}
      disabled={art.busy}
      onclick={() => pictureInput?.click()}
    >
      <ToolbarIcon name="image" />
    </button>
  {/if}
  <input
    bind:this={coverInput}
    class="art-file"
    type="file"
    accept="image/jpeg,image/png,image/webp"
    aria-label={t('Cover image')}
    onchange={() => pick('cover', coverInput)}
  />
  <input
    bind:this={pictureInput}
    class="art-file"
    type="file"
    accept="image/jpeg,image/png,image/webp"
    aria-label={t('Picture image')}
    onchange={() => pick('picture', pictureInput)}
  />
</section>
{#if cropping}
  <ImageCropper
    src={cropping.src}
    slot={cropping.slot}
    onDone={cropped}
    onCancel={close}
    onReplace={() => (cropping?.slot === 'cover' ? coverInput : pictureInput)?.click()}
  />
{/if}
