<script lang="ts">
  import EditSheet from '#lib/components/overlay/EditSheet.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
  import type { PlaceSearch } from '#lib/domain/places.ts';
  import type { Charter, CharterDraft } from '#lib/domain/projects.ts';

  import { ScopeArt } from './scopes/scopeArt.svelte.ts';
  import ScopeSheetArt from './scopes/ScopeSheetArt.svelte';

  /**
   * Propose edit with the space's cover and picture on top (Owner Notes Mockups §2, ADR 0117):
   * a new one is staged until the proposal is filed, and members vote on it with the rest.
   * Help requests, threads and posts have none, so they get the plain sheet.
   */
  interface Props {
    open: boolean;
    sheetTitle: string;
    submitLabel: string;
    appliesDirectly: boolean;
    charter: Charter;
    pending: boolean;
    message: string | undefined;
    places: PlaceSearch;
    onSubmit: (draft: CharterDraft) => void;
    onClose: () => void;
  }

  let { open, charter, onSubmit, onClose, ...rest }: Props = $props();

  const art = new ScopeArt();
  const takesArt = $derived(!['help-request', 'thread', 'post'].includes(charter.mode));
  const icon = $derived<SurfaceIconId>(charter.mode === 'event' ? 'event' : 'project');

  $effect(() => {
    if (!open) return;
    art.clearStaged();
    void art.show(charter.picture, charter.cover);
  });

  function submit(draft: CharterDraft): void {
    onSubmit({
      ...draft,
      picture: art.newPicture?.hash ?? null,
      cover: art.newCover?.hash ?? null,
    });
  }
</script>

<EditSheet
  {open}
  {charter}
  {onClose}
  onSubmit={submit}
  {...rest}
  art={takesArt ? artBlock : undefined}
/>

{#snippet artBlock()}
  <ScopeSheetArt kind="space" {icon} {art} canEdit />
{/snippet}
