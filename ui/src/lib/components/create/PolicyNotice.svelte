<script lang="ts">
  import {
    DIRECT_USE_CORE,
    DIRECT_USE_CREATE,
    DIRECT_USE_EVENT_CREATE,
  } from '#lib/domain/policy.ts';

  /** web's DirectUsePolicyNotice (`context="event"` on the create-event page). */
  let { event = false }: { event?: boolean } = $props();
</script>

<aside class="policy-notice" aria-label="Direct-use policy">
  <strong>Direct use only — no commercial activity</strong>
  <p>{event ? DIRECT_USE_EVENT_CREATE : DIRECT_USE_CREATE}</p>
  <p class="policy-core">{DIRECT_USE_CORE}</p>
</aside>
