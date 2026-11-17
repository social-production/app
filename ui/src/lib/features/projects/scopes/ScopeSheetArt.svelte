<script lang="ts">
  import SurfaceIcon from '#lib/components/icons/SurfaceIcon.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import type { ScopeKind } from '#lib/domain/scopes.ts';

  import type { ScopeArt } from './scopeArt.svelte.ts';

  /**
   * The top of a scope's Settings (Channel Pictures mockups §2): the cover with "Change cover"
   * and the picture with a camera, as in Edit profile. A new one shows "New" until it is saved
   * or voted on (ADR 0109).
   */
  interface Props {
    kind: ScopeKind;
    art: ScopeArt;
    canEdit: boolean;
  }

  let { kind, art, canEdit }: Props = $props();

  let coverInput = $state<HTMLInputElement>();
  let pictureInput = $state<HTMLInputElement>();

  const cover = $derived(art.newCover?.url ?? art.coverUrl);
  const picture = $derived(art.newPicture?.url ?? art.pictureUrl);

  function pick(slot: 'picture' | 'cover', input: HTMLInputElement | undefined): void {
    const file = input?.files?.[0];
    if (file) void art.stage(slot, file);
    if (input) input.value = '';
  }
</script>

<section class="scope-sheet-art">
  <div class="scope-cover" style={cover ? `background-image: url(${cover})` : undefined}>
    {#if art.newCover}<span class="art-new">New</span>{/if}
    {#if canEdit}
      <button
        class="art-change"
        type="button"
        disabled={art.busy}
        onclick={() => coverInput?.click()}
      >
        <ToolbarIcon name="image" /> Change cover
      </button>
    {/if}
  </div>
  <span class="scope-avatar">
    {#if picture}<img src={picture} alt="" />{:else if kind === 'channel'}#{:else}<SurfaceIcon
        icon={kind}
        size="md"
      />{/if}
    {#if art.newPicture}<span class="art-new">New</span>{/if}
  </span>
  {#if canEdit}
    <button
      class="art-camera"
      type="button"
      aria-label="Change picture"
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
    aria-label="Cover image"
    onchange={() => pick('cover', coverInput)}
  />
  <input
    bind:this={pictureInput}
    class="art-file"
    type="file"
    accept="image/jpeg,image/png,image/webp"
    aria-label="Picture image"
    onchange={() => pick('picture', pictureInput)}
  />
</section>
