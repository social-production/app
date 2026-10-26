<script lang="ts">
  import type { Activity } from '#lib/domain/activities.ts';
  import { dayHeading, dayKey } from '#lib/domain/activityTime.ts';

  import ActivityCard from './ActivityCard.svelte';

  import '#lib/components/styles/activities.css';

  /** web's ActivityAgendaList: upcoming activities as agenda rows grouped by day. */
  interface Props {
    activities: Activity[];
    names: Record<string, string>;
    me: string;
    planStageOf: (id: string | null) => string | null;
    highlighted: string | null;
    readOnly: boolean;
    canSuggest: boolean;
    onCommit: (activity: string, role: number | null) => void;
    onSuggest: (activity: string, role: number, agent: string) => void;
    onDecline: (suggestion: string) => void;
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
  }

  let p: Props = $props();

  const days = $derived.by(() => {
    const out: { key: string; label: string; items: Activity[] }[] = [];
    for (const a of [...p.activities].sort((x, y) => x.startsAt - y.startsAt)) {
      const key = dayKey(a.startsAt);
      let day = out.find((d) => d.key === key);
      if (!day) {
        day = { key, label: dayHeading(a.startsAt), items: [] };
        out.push(day);
      }
      day.items.push(a);
    }
    return out;
  });
</script>

<div class="agenda">
  {#each days as day (day.key)}
    <section class="agenda-day" aria-label={day.label}>
      <h4 class="agenda-day-label">{day.label}</h4>
      <div class="agenda-rows">
        {#each day.items as activity (activity.id)}
          <div id={`activity-card-${activity.id}`} class="rail-card">
            <ActivityCard
              {activity}
              names={p.names}
              me={p.me}
              planStage={p.planStageOf(activity.planStage)}
              highlighted={p.highlighted === activity.id}
              readOnly={p.readOnly}
              canSuggest={p.canSuggest}
              onCommit={(role) => p.onCommit(activity.id, role)}
              onSuggest={(role, agent) => p.onSuggest(activity.id, role, agent)}
              onDecline={p.onDecline}
              searchPeople={p.searchPeople}
            />
          </div>
        {/each}
      </div>
    </section>
  {/each}
</div>
