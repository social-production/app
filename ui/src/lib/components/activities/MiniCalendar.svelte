<script lang="ts">
  import { locale, t } from '#lib/i18n/index.ts';
  import { localDayOf } from '#lib/domain/activityWhen.ts';

  import '#lib/components/styles/ux/daytime.css';

  /**
   * A small month calendar (Activity creation mockup v2): Monday first, the chosen day filled,
   * a meetup's usual weekday ringed, days before `min` or outside `allowed` greyed out.
   */
  interface Props {
    /** The chosen day, `YYYY-MM-DD`. */
    value: string;
    min: string;
    /** An event's plan days; every day from `min` when empty. */
    allowed?: string[];
    /** 0 Sunday … 6 Saturday. */
    usualWeekday?: number | null;
    onPick: (day: string) => void;
  }

  let { value, min, allowed = [], usualWeekday = null, onPick }: Props = $props();

  // The month shown; it follows the chosen day until you page.
  let shown = $derived(new Date(`${value || min}T12:00`));

  const title = $derived(shown.toLocaleDateString(locale(), { month: 'long', year: 'numeric' }));
  const weekdays = $derived(
    // 2024-01-01 was a Monday.
    Array.from({ length: 7 }, (_, i) =>
      new Date(2024, 0, 1 + i).toLocaleDateString(locale(), { weekday: 'narrow' }),
    ),
  );
  const cells = $derived.by(() => {
    const first = new Date(shown.getFullYear(), shown.getMonth(), 1, 12);
    const lead = (first.getDay() + 6) % 7;
    const count = new Date(shown.getFullYear(), shown.getMonth() + 1, 0).getDate();
    return [
      ...Array.from({ length: lead }, () => null),
      ...Array.from({ length: count }, (_, i) => {
        const d = new Date(shown.getFullYear(), shown.getMonth(), i + 1, 12);
        return { day: localDayOf(d), n: i + 1, weekday: d.getDay() };
      }),
    ];
  });
  const open = (day: string): boolean =>
    day >= min && (allowed.length === 0 || allowed.includes(day));

  function page(by: number): void {
    shown = new Date(shown.getFullYear(), shown.getMonth() + by, 1, 12);
  }
</script>

<div class="mini-cal">
  <div class="mini-cal-head">
    <b>{title}</b>
    <span>
      <button type="button" aria-label={t('Previous month')} onclick={() => page(-1)}>‹</button>
      <button type="button" aria-label={t('Next month')} onclick={() => page(1)}>›</button>
    </span>
  </div>
  <div class="mini-cal-grid" role="grid" aria-label={title}>
    {#each weekdays as w, i (i)}<span class="mini-cal-dow" aria-hidden="true">{w}</span>{/each}
    {#each cells as cell, i (cell?.day ?? `blank-${i}`)}
      {#if cell}
        <button
          type="button"
          class:sel={cell.day === value}
          class:usual={usualWeekday === cell.weekday && cell.day !== value && open(cell.day)}
          disabled={!open(cell.day)}
          aria-pressed={cell.day === value}
          aria-label={new Date(`${cell.day}T12:00`).toLocaleDateString(locale(), {
            weekday: 'long',
            day: 'numeric',
            month: 'long',
          })}
          onclick={() => onPick(cell.day)}>{cell.n}</button
        >
      {:else}
        <span></span>
      {/if}
    {/each}
  </div>
</div>
