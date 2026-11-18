<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  /** A long text that folds after about six lines, with "Show more" (Plan, Phase and
   * Feedback Screens, 1). Short texts show whole, with no link. */
  let { text, className = '' }: { text: string; className?: string } = $props();

  let open = $state(false);
  let el = $state<HTMLParagraphElement>();
  let long = $state(false);

  $effect(() => {
    void text;
    if (el && !open) long = el.scrollHeight > el.clientHeight + 2;
  });
</script>

<p bind:this={el} class={`folded-text ${className}`} class:folded={!open}>{text}</p>
{#if long}
  <button class="fold-toggle" type="button" onclick={() => (open = !open)}
    >{open ? t('Show less') : t('Show more')}</button
  >
{/if}
