<script lang="ts">
  import { linkedSegments } from '#lib/domain/linkedBody.ts';

  /** web's LinkedPostBody: a post's body with its URLs and app paths as links. */
  let { body, variant = 'feed' }: { body: string; variant?: 'feed' | 'detail' } = $props();

  const segments = $derived(linkedSegments(body));
</script>

<p class:detail={variant === 'detail'} class:feed={variant === 'feed'} class="linked-body">
  {#each segments as segment, i (i)}
    {#if segment.kind === 'link'}
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a link written in the post -->
      <a class="linked-subject" href={segment.href}>{segment.value}</a>
    {:else}{segment.value}{/if}
  {/each}
</p>
