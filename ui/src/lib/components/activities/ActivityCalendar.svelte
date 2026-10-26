<script lang="ts">
  import PhaseWorkToolbar from '#lib/components/governance/PhaseWorkToolbar.svelte';
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import type { Activity } from '#lib/domain/activities.ts';
  import { defaultMonth, isoOf, monthDays, monthStart } from '#lib/domain/activityCalendar.ts';
  import { zoneOf } from '#lib/domain/activityTime.ts';

  import '#lib/components/styles/activities.css';

  /** web's ProjectActivityCalendarCard: a month of days with activity bars, and Add activity. */
  interface Props {
    activities: Activity[];
    selectedDay: string;
    selectedActivity: string;
    canCreate: boolean;
    createActive: boolean;
    onDay: (iso: string) => void;
    onActivity: (id: string) => void;
    onCreate: () => void;
  }

  let p: Props = $props();

  // svelte-ignore state_referenced_locally
  let month = $state(defaultMonth(p.activities));
  let hoveredDay = $state('');
  let hoveredActivity = $state('');

  const days = $derived(monthDays(month, p.activities));
  const label = $derived(month.toLocaleDateString([], { month: 'long', year: 'numeric' }));
  const zone = zoneOf(new Date());
  const today = isoOf(new Date());
  const agendaDay = $derived(
    p.selectedDay ||
      (days.some((d) => d.iso === today) ? today : '') ||
      days.find((d) => d.bars.length > 0)?.iso ||
      '',
  );

  function shift(offset: number): void {
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- a throwaway copy
    const next = new Date(month);
    next.setMonth(next.getMonth() + offset);
    month = monthStart(next);
  }

  const past = (iso: string): boolean => iso < today;
</script>

<div class="calendar-shell surface-card">
  <div class="calendar-toolbar">
    <button
      aria-label="Previous month"
      title="Previous month"
      class="month-button"
      type="button"
      onclick={() => shift(-1)}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16">
        <path
          d="M14.5 6.5 9 12l5.5 5.5"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
        />
      </svg>
    </button>
    <strong class="month-label"
      >{label}{#if zone}
        <span class="zone-label" title="Times are shown in your Display timezone">· {zone}</span
        >{/if}</strong
    >
    <button
      aria-label="Next month"
      title="Next month"
      class="month-button"
      type="button"
      onclick={() => shift(1)}
    >
      <svg aria-hidden="true" viewBox="0 0 24 24" width="16" height="16">
        <path
          d="m9.5 6.5 5.5 5.5-5.5 5.5"
          fill="none"
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
        />
      </svg>
    </button>
  </div>

  <div class="calendar-grid calendar-header-row">
    {#each ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'] as weekday (weekday)}
      <span>
        <span class="weekday-full">{weekday}</span>
        <span class="weekday-short">{weekday.slice(0, 1)}</span>
      </span>
    {/each}
  </div>
  <div class="calendar-grid">
    {#each days as day (day.iso)}
      <div
        class="calendar-cell"
        class:muted-day={!day.inMonth}
        class:past-day={past(day.iso)}
        class:hovered-day={hoveredDay === day.iso}
        class:selected-day={agendaDay === day.iso}
        class:today-day={today === day.iso}
        role="button"
        tabindex="0"
        aria-label={`${day.day}${day.bars.length ? `, ${day.bars.length} ${day.bars.length === 1 ? 'activity' : 'activities'}` : ''}`}
        onmouseenter={() => (hoveredDay = day.iso)}
        onmouseleave={() => (hoveredDay = '')}
        onfocus={() => (hoveredDay = day.iso)}
        onblur={() => (hoveredDay = '')}
        ontouchstart={() => (hoveredDay = day.iso)}
        onclick={() => p.onDay(day.iso)}
        onkeydown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            p.onDay(day.iso);
          }
        }}
      >
        <span class="calendar-day-number">{day.day}</span>
        <div class="activity-dots" aria-hidden="true">
          {#each day.bars.slice(0, 3) as bar (bar.id)}
            <span class={`activity-dot tone-${bar.tone}`}></span>
          {/each}
        </div>
        <div class="timeline-track">
          {#each day.bars as bar (bar.id)}
            <button
              class={`timeline-item tone-${bar.tone}`}
              class:hovered-activity={hoveredActivity === bar.id}
              class:selected-activity={p.selectedActivity === bar.id}
              style={`top:${bar.top}%;height:${bar.height}%;left:${bar.left}%;width:${bar.width}%;`}
              type="button"
              aria-label={`${bar.title}, ${bar.label}`}
              title={`${bar.title}, ${bar.label}`}
              onmouseenter={() => (hoveredActivity = bar.id)}
              onmouseleave={() => (hoveredActivity = '')}
              onfocus={() => (hoveredActivity = bar.id)}
              onblur={() => (hoveredActivity = '')}
              onclick={(e) => {
                e.stopPropagation();
                p.onActivity(bar.id);
              }}
            ></button>
          {/each}
        </div>
      </div>
    {/each}
  </div>
</div>

{#if p.canCreate}
  <PhaseWorkToolbar>
    <RoundPlusButton
      standout
      active={p.createActive}
      label="Add activity"
      ariaLabel="Add activity"
      onclick={p.onCreate}
    />
  </PhaseWorkToolbar>
{/if}
