<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { APPROVAL_PERCENT } from '#lib/domain/governance.ts';
  import type { Snippet } from 'svelte';

  /** One thin bar towards the 66% mark and one line under it (owner, 2026-11-18). */
  interface Props {
    yes: number;
    no: number;
    population: number;
    failed?: boolean;
    children?: Snippet;
  }

  let { yes, no, population, failed = false, children }: Props = $props();

  const total = $derived(yes + no);
  const percent = $derived(total > 0 ? Math.round((yes / total) * 100) : 0);
</script>

<section class="vote-tally" aria-label={t('Votes')}>
  <div class="vote-tally-bar" class:failed>
    <i style={`width: ${percent}%`}></i>
    <span style={`left: ${APPROVAL_PERCENT}%`}></span>
  </div>
  <div class="vote-tally-line">
    <span
      ><b>{t('{yes} yes · {no} no', { yes, no })}</b> · {t('{n} of {total} voted', {
        n: total,
        total: population,
      })}</span
    >
    <span>{t('needs {percent}%', { percent: APPROVAL_PERCENT })}</span>
  </div>
  {@render children?.()}
</section>
