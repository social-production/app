<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';

  /** The end of a feed or list (Scope Filters mockups §2c). */
</script>

<p class="feed-end">
  <span class="feed-end-tick" aria-hidden="true"><ToolbarIcon name="check" /></span>{t(
    "You're all caught up",
  )}
</p>
