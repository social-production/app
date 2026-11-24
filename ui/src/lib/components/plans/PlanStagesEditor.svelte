<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { PlanForm } from '#lib/domain/planForm.ts';
  import { emptyStage, moveStage, stageLine } from '#lib/domain/planStages.ts';

  import '#lib/components/styles/ux/planstages.css';

  /**
   * The plan wizard's stages step as cards (Edit, Plan and Meetup mockups §B, owner
   * 2026-11-24): a number, the title and a small line; tapping one opens its title and
   * details (and dates, for an event plan only: other plans are timed by their activities,
   * owner 2026-11-24); hold ⠿ to drag it; a dashed "+ Add stage" at the end.
   */
  let { stages = $bindable(), dated = false }: { stages: PlanForm['stages']; dated?: boolean } =
    $props();

  // The first stage starts open while it is empty.
  let opened = $state<number | null>(stages.length === 1 && !stages[0]?.title ? 0 : null);
  let dragging = $state<number | null>(null);
  let list = $state<HTMLOListElement>();

  function add(): void {
    stages = [...stages, emptyStage()];
    opened = stages.length - 1;
  }

  function remove(index: number): void {
    if (stages.length <= 1) return;
    stages = stages.filter((_, i) => i !== index);
    opened = null;
  }

  function move(from: number, to: number): void {
    stages = moveStage(stages, from, to);
    if (opened === from) opened = to;
  }

  function grab(event: PointerEvent, index: number): void {
    event.preventDefault();
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
    dragging = index;
    opened = null;
  }

  // The card under the pointer takes the dragged stage's place.
  function drag(event: PointerEvent): void {
    if (dragging === null || !list) return;
    const cards = [...list.querySelectorAll<HTMLElement>(':scope > li')];
    const over = cards.findIndex((card) => {
      const r = card.getBoundingClientRect();
      return event.clientY >= r.top && event.clientY <= r.bottom;
    });
    if (over >= 0 && over !== dragging) {
      move(dragging, over);
      dragging = over;
    }
  }
</script>

<ol class="stage-cards" bind:this={list}>
  {#each stages as stage, index (index)}
    <li class:open={opened === index} class:dragging={dragging === index}>
      <div class="stage-card">
        <button
          type="button"
          class="stage-face"
          aria-expanded={opened === index}
          onclick={() => (opened = opened === index ? null : index)}
        >
          <span class="stage-n">{index + 1}</span>
          <b>{stage.title.trim() || t('New stage')}</b>
          <small>{stageLine(stage) || t('Tap to add what happens')}</small>
        </button>
        <span
          class="stage-handle"
          role="button"
          tabindex="-1"
          aria-label={t('Drag to reorder')}
          onpointerdown={(e) => grab(e, index)}
          onpointermove={drag}
          onpointerup={() => (dragging = null)}
          onpointercancel={() => (dragging = null)}>⠿</span
        >
      </div>
      {#if opened === index}
        <div class="stage-fields">
          <label>
            <span class="stage-lab">{t('Title')}</span>
            <input
              bind:value={stage.title}
              maxlength="120"
              placeholder={t('Harvest Friday')}
              aria-required="true"
            />
          </label>
          <label>
            <span class="stage-lab">{t('What happens')}</span>
            <textarea
              bind:value={stage.details}
              rows="3"
              placeholder={t('What happens in this stage?')}
              aria-required="true"></textarea>
          </label>
          {#if dated}<div class="stage-dates">
              <label>
                <span class="stage-lab">{t('Starts')}</span>
                <input bind:value={stage.startDate} type="date" />
              </label>
              <label>
                <span class="stage-lab">{t('Ends')}</span>
                <input bind:value={stage.endDate} min={stage.startDate || undefined} type="date" />
              </label>
            </div>{/if}
          <div class="stage-tools">
            <button
              type="button"
              aria-label={t('Move up')}
              disabled={index === 0}
              onclick={() => move(index, index - 1)}>↑</button
            >
            <button
              type="button"
              aria-label={t('Move down')}
              disabled={index === stages.length - 1}
              onclick={() => move(index, index + 1)}>↓</button
            >
            {#if stages.length > 1}
              <button type="button" class="stage-remove" onclick={() => remove(index)}
                >{t('Remove')}</button
              >
            {/if}
            <button type="button" class="stage-done" onclick={() => (opened = null)}
              >{t('Done')}</button
            >
          </div>
        </div>
      {/if}
    </li>
  {/each}
</ol>
<button class="stage-add" type="button" onclick={add}>+ {t('Add stage')}</button>
