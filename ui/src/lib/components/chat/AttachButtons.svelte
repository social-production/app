<script lang="ts">
  import type { ComposerFiles } from './composerFiles.svelte.ts';

  /** web LiveChatPanel's "Add a photo" and "Add a file" buttons and their file inputs. The
   * inputs sit outside the composer row in web's markup; `inputs` renders them there. */
  interface Props {
    files: ComposerFiles;
    part: 'buttons' | 'inputs';
  }

  let { files, part }: Props = $props();
  // One picker pair per composer: the buttons and the inputs are two renders of it.
  const ids = $derived(`attach-${files.key}`);
</script>

{#if part === 'buttons'}
  <button
    aria-label="Add a photo"
    title="Add a photo"
    class="attach-button"
    type="button"
    onclick={() => document.getElementById(`${ids}-photo`)?.click()}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24" width="20" height="20">
      <rect
        x="3"
        y="5"
        width="18"
        height="14"
        rx="2"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      />
      <circle cx="8.5" cy="10" r="1.5" fill="currentColor" />
      <path d="M21 16l-5-5-8 8" fill="none" stroke="currentColor" stroke-width="2" />
    </svg>
  </button>
  <button
    aria-label="Add a file"
    title="Add a file"
    class="attach-button"
    type="button"
    onclick={() => document.getElementById(`${ids}-file`)?.click()}
  >
    <svg aria-hidden="true" viewBox="0 0 24 24" width="20" height="20">
      <path
        d="M21.44 11.05l-9.19 9.19a6 6 0 01-8.49-8.49l9.19-9.19a4 4 0 015.66 5.66l-9.2 9.19a2 2 0 01-2.82-2.83l8.48-8.48"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      />
    </svg>
  </button>
{:else}
  <input
    id={`${ids}-photo`}
    accept="image/jpeg,image/png,image/webp"
    aria-hidden="true"
    class="attach-input"
    multiple
    tabindex="-1"
    type="file"
    onchange={(event) => void files.choosePhotos(event.currentTarget)}
  />
  <input
    id={`${ids}-file`}
    aria-hidden="true"
    class="attach-input"
    multiple
    tabindex="-1"
    type="file"
    onchange={(event) => files.chooseFiles(event.currentTarget)}
  />
{/if}
