<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { WEEKDAYS, weekdayName } from '#lib/domain/meetupSchedule.ts';
  import type { ScheduleForm } from '#lib/domain/eventSchedule.ts';

  import ChoiceRows from './ChoiceRows.svelte';

  import '#lib/components/styles/ux/meetup.css';

  /**
   * A meetup's "How do you meet?" (Edit, Plan and Meetup mockups §C, ADR 0119): on a regular
   * day, with an optional usual time, or whenever members set a session. Either way members
   * add each session as an activity.
   */
  let { form = $bindable() }: { form: ScheduleForm } = $props();
</script>

<div class="meetup-how">
  <ChoiceRows
    value={form.scheduleMode === 'any-day' ? 'any-day' : 'weekly'}
    ariaLabel={t('How do you meet?')}
    options={[
      {
        value: 'weekly',
        label: t('On a regular day'),
        hint: t('Shown as a hint; members still add each session'),
        icon: 'M5 5h14v15H5zM5 10h14M9 3v4M15 3v4M9 15h2',
      },
      {
        value: 'any-day',
        label: t('Whenever members set one'),
        hint: t('No usual time; sessions go on the calendar'),
        icon: 'M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM12 7v5l3 2',
      },
    ]}
    onChange={(value) => (form.scheduleMode = value)}
  >
    {#snippet under(chosen)}
      {#if chosen === 'weekly'}
        <div class="meetup-when">
          <label>
            <span>{t('Day')}</span>
            <select bind:value={form.weeklyDay}>
              {#each WEEKDAYS as day (day)}<option value={day}>{weekdayName(day)}</option>{/each}
            </select>
          </label>
          <label>
            <span>{t('From')}</span>
            <input type="time" bind:value={form.startTime} />
          </label>
          <label>
            <span>{t('Until')}</span>
            <input type="time" bind:value={form.finishTime} />
          </label>
        </div>
      {/if}
    {/snippet}
  </ChoiceRows>
</div>
