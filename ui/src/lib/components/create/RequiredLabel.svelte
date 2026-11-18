<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { Snippet } from 'svelte';

  /** web's RequiredFieldLabel: the label text plus a "*" (read out as "required"). */
  let { children }: { children: Snippet } = $props();
</script>

<span class="field-label">
  {@render children()}
  <span class="required-marker" aria-hidden="true">*</span>
  <span class="sr-only"> {t('(required)')}</span>
</span>
