<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { scheduleProblem, type ActivityForm } from '#lib/domain/activityForm.ts';
  import { dayOf, onDay, timeOf, withEnd, withStart } from '#lib/domain/activityWhen.ts';

  import MiniCalendar from './MiniCalendar.svelte';
  import TimeWheel from './TimeWheel.svelte';

  /** Step 1 (Activity creation mockup v2): the name, a month calendar and two time wheels. */
  interface Props {
    form: ActivityForm;
    /** Today, or an event's first plan day. */
    minDay: string;
    allowedDays: string[];
    usualWeekday: number | null;
    onChange: (form: ActivityForm) => void;
  }

  let { form, minDay, allowedDays, usualWeekday, onChange }: Props = $props();

  const problem = $derived(scheduleProblem(form));
  const set = (times: { scheduledAt: string; endsAt: string }): void =>
    onChange({ ...form, ...times });
</script>

<div class="activity-when">
  <label class="activity-field">
    <span>{t('Name')}</span>
    <input
      value={form.title}
      maxlength="120"
      placeholder={t('Activity title')}
      oninput={(e) => onChange({ ...form, title: e.currentTarget.value })}
    />
  </label>
  <MiniCalendar
    value={dayOf(form.scheduledAt)}
    min={minDay}
    allowed={allowedDays}
    {usualWeekday}
    onPick={(day) => set(onDay(form, day))}
  />
  <div class="time-wheels">
    <TimeWheel
      label={t('Starts')}
      value={timeOf(form.scheduledAt)}
      onChange={(time) => set(withStart(form, time))}
    />
    <TimeWheel
      label={t('Ends')}
      value={timeOf(form.endsAt)}
      onChange={(time) => set(withEnd(form, time))}
    />
  </div>
  {#if problem}<p class="validation-copy">{problem}</p>{/if}
</div>
