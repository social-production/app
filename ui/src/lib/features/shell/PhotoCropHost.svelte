<script lang="ts">
  import ImageCropper from '#lib/components/overlay/ImageCropper.svelte';
  import { finishCrop, photoCrop, replaceCrop } from '#lib/stores/photoCrop.svelte.ts';

  /**
   * The one crop screen for a person's photo and cover; pages ask for it with `cropPhoto`
   * (stores/photoCrop). The same cropper as proposals and channels (owner, 2026-11-25: the
   * profile's picture and cover "didn't let me resize and adjust position").
   */
  let input = $state<HTMLInputElement>();
  let src = $state<string | null>(null);

  $effect(() => {
    const file = photoCrop.file;
    if (!file) return;
    const url = URL.createObjectURL(file);
    src = url;
    return () => {
      URL.revokeObjectURL(url);
      src = null;
    };
  });
</script>

{#if photoCrop.file && src}
  {#key src}
    <ImageCropper
      {src}
      slot={photoCrop.shape === 'wide' ? 'cover' : 'picture'}
      onDone={(file) => finishCrop(file)}
      onCancel={() => finishCrop(null)}
      onReplace={() => input?.click()}
    />
  {/key}
{/if}
<input
  bind:this={input}
  hidden
  type="file"
  accept="image/*"
  onchange={(e) => {
    const target = e.currentTarget;
    const file = target.files?.[0];
    target.value = '';
    if (file) replaceCrop(file);
  }}
/>
