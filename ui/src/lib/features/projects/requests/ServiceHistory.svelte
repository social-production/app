<script lang="ts">
  import ActivityHistory from '#lib/components/activities/ActivityHistory.svelte';

  import type { ActivitiesState } from '../activitiesState.svelte.ts';

  /** web's personal-service "Past activity": Request history, then Self planned history. */
  let { activities, me }: { activities: ActivitiesState; me: string } = $props();

  const requests = $derived(activities.past.filter((a) => a.booking));
  const planned = $derived(activities.past.filter((a) => !a.booking));
</script>

{#if activities.past.length > 0}
  <details class="history-section">
    <summary class="history-summary">
      <span>Past activity</span>
      <svg class="history-chevron" aria-hidden="true" viewBox="0 0 24 24" width="18" height="18">
        <path
          d="M6 9l6 6 6-6"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </summary>
    <div class="history-stack activity-ui">
      <ActivityHistory
        items={requests}
        heading={{
          title: 'Request history',
          description: 'Past accepted requests and request-based activity.',
          empty: 'No request-based activity has moved into history yet.',
        }}
        names={activities.names}
        {me}
        planStageOf={() => null}
        onCheckIn={activities.checkIn}
        onRate={activities.rate}
      />
      <ActivityHistory
        items={planned}
        heading={{
          title: 'Self planned history',
          description: 'Past availability or directly created service activity.',
          empty: 'No self-planned activity has moved into history yet.',
        }}
        names={activities.names}
        {me}
        planStageOf={() => null}
        onCheckIn={activities.checkIn}
        onRate={activities.rate}
      />
    </div>
  </details>
{/if}
