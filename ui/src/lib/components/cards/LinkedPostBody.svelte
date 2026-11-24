<script lang="ts">
  import { linkedSegments } from '#lib/domain/linkedBody.ts';
  import { quoteOf } from '#lib/domain/quote.ts';

  import QuotedCard from './QuotedCard.svelte';

  /** web's LinkedPostBody: a post's body with its URLs and app paths as links. A post ending
   * with a link to something in the app shows it as a quoted card (quote posts, §6). */
  let { body, variant = 'feed' }: { body: string; variant?: 'feed' | 'detail' } = $props();

  const quote = $derived(quoteOf(body, location.origin));
  const segments = $derived(linkedSegments(quote ? quote.text : body));
</script>

{#if !quote || quote.text}<p
    class:detail={variant === 'detail'}
    class:feed={variant === 'feed'}
    class="linked-body"
  >
    {#each segments as segment, i (i)}
      {#if segment.kind === 'link'}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- a link written in the post -->
        <a class="linked-subject" href={segment.href}>{segment.value}</a>
      {:else}{segment.value}{/if}
    {/each}
  </p>{/if}
{#if quote}<QuotedCard id={quote.id} href={quote.href} />{/if}
