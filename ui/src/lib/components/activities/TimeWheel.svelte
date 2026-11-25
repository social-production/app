<script lang="ts">
  import { locale } from '#lib/i18n/index.ts';
  import { snapTime, timeOptions } from '#lib/domain/activityWhen.ts';

  import '#lib/components/styles/ux/daytime.css';

  /**
   * A scroll wheel for a time (Activity creation mockup v2): five-minute steps that snap to the
   * middle row; scrolling or tapping a row picks it. A list box for keyboards and screen
   * readers, arrow keys step it.
   */
  interface Props {
    label: string;
    /** `HH:MM`. */
    value: string;
    onChange: (time: string) => void;
  }

  let { label, value, onChange }: Props = $props();

  const ROW = 36;
  const times = timeOptions(5);
  let list = $state<HTMLDivElement>();
  let settle: ReturnType<typeof setTimeout> | undefined;
  const current = $derived(snapTime(value || '18:00'));

  const shownTime = (time: string): string =>
    new Date(`2024-01-01T${time}`).toLocaleTimeString(locale(), {
      hour: 'numeric',
      minute: '2-digit',
    });

  // Only a person's scrolling picks a time; layout and snapping scroll it too.
  let touched = false;

  // Keep the chosen time in the middle row when it changes from outside (after layout: a
  // sheet still opening has no height to scroll yet).
  $effect(() => {
    const index = times.indexOf(current);
    const el = list;
    if (!el || index < 0) return;
    const place = (): void => {
      if (!touched && Math.round(el.scrollTop / ROW) !== index) el.scrollTop = index * ROW;
    };
    place();
    const frame = requestAnimationFrame(place);
    const later = setTimeout(place, 350);
    return () => {
      cancelAnimationFrame(frame);
      clearTimeout(later);
    };
  });

  function scrolled(): void {
    if (!touched) return;
    clearTimeout(settle);
    settle = setTimeout(() => {
      touched = false;
      if (!list) return;
      const picked = times[Math.round(list.scrollTop / ROW)];
      if (picked && picked !== current) onChange(picked);
    }, 120);
  }

  function key(event: KeyboardEvent): void {
    const by = event.key === 'ArrowDown' ? 1 : event.key === 'ArrowUp' ? -1 : 0;
    if (!by) return;
    event.preventDefault();
    const next = times[times.indexOf(current) + by];
    if (next) onChange(next);
  }
</script>

<div class="time-wheel">
  <small>{label}</small>
  <div
    class="time-wheel-list"
    bind:this={list}
    role="listbox"
    tabindex="0"
    aria-label={label}
    aria-activedescendant={`tw-${label}-${current}`}
    onscroll={scrolled}
    onkeydown={key}
    onpointerdown={() => (touched = true)}
    onwheel={() => (touched = true)}
    ontouchstart={() => (touched = true)}
  >
    {#each times as time (time)}
      <button
        id={`tw-${label}-${time}`}
        type="button"
        role="option"
        tabindex="-1"
        aria-selected={time === current}
        class:on={time === current}
        onclick={() => onChange(time)}>{shownTime(time)}</button
      >
    {/each}
  </div>
</div>
