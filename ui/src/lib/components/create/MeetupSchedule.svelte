<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { WEEKDAYS, weekdayName } from '#lib/domain/meetupSchedule.ts';
  import type { ScheduleForm } from '#lib/domain/eventSchedule.ts';

  import '#lib/components/styles/ux/meetup.css';

  /**
   * A meetup's "How do you meet?" (Edit, Plan and Meetup mockups §C, ADR 0119): on a regular
   * day, with an optional usual time, or whenever members set a session. Either way members
   * add each session as an activity.
   */
  let { form = $bindable() }: { form: ScheduleForm } = $props();
</script>

<div class="meetup-how" role="radiogroup" aria-label={t('How do you meet?')}>
  <label class="meetup-opt" class:on={form.scheduleMode === 'weekly'}>
    <input type="radio" bind:group={form.scheduleMode} value="weekly" />
    <b>{t('On a regular day')}</b>
    <small>{t('Shown as a hint; members still add each session')}</small>
  </label>
  {#if form.scheduleMode === 'weekly'}
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
  <label class="meetup-opt" class:on={form.scheduleMode === 'any-day'}>
    <input type="radio" bind:group={form.scheduleMode} value="any-day" />
    <b>{t('Whenever members set one')}</b>
    <small>{t('No usual time; sessions go on the calendar')}</small>
  </label>
</div>
