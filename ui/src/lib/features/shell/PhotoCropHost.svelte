<script lang="ts">
  import PhotoCropSheet from '#lib/components/photos/PhotoCropSheet.svelte';
  import { finishCrop, photoCrop } from '#lib/stores/photoCrop.svelte.ts';

  /** The one photo crop sheet; pages ask for it with `cropPhoto` (stores/photoCrop). */
</script>

<PhotoCropSheet file={photoCrop.file} shape={photoCrop.shape} onDone={finishCrop} />
