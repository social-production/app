<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';

  import RepositoryMenu from './RepositoryMenu.svelte';
  import type { SoftwareState } from './softwareState.svelte.ts';

  /** web's Activity phase `dock-extra`: Pull request, then Repository ▾ (members only). */
  let { sw }: { sw: SoftwareState } = $props();

  const can = $derived(!!sw.governance?.canPropose);
</script>

{#if can}
  <RoundPlusButton
    label={t('Pull request')}
    ariaLabel={t('New pull request')}
    onclick={() => sw.open('create-pr')}
  />
  <RepositoryMenu
    canReplace={can}
    canChangeMerge={can && !sw.governance?.platform}
    onReplace={() => sw.open('repository-replacement')}
    onChangeMerge={() => sw.open('merge-capability')}
  />
{/if}
