<script lang="ts">
  import type { PlanForm } from '#lib/domain/planForm.ts';

  /** web's plan wizard stages step: add, remove and reorder stages, each with optional dates. */
  let { stages = $bindable() }: { stages: PlanForm['stages'] } = $props();

  function add(): void {
    stages = [...stages, { title: '', details: '', startDate: '', endDate: '' }];
  }

  function remove(index: number): void {
    if (stages.length > 1) stages = stages.filter((_, i) => i !== index);
  }

  function move(index: number, direction: -1 | 1): void {
    const target = index + direction;
    const moved = stages[index];
    if (!moved || target < 0 || target >= stages.length) return;
    const next = stages.filter((_, i) => i !== index);
    next.splice(target, 0, moved);
    stages = next;
  }
</script>

<div class="stages-stack">
  {#each stages as stage, index (index)}
    <div class="stage-editor">
      <div class="stage-editor-head">
        <strong>Stage {index + 1}</strong>
        <div class="stage-editor-actions">
          <button
            aria-label={`Move stage ${index + 1} up`}
            class="icon-button"
            disabled={index === 0}
            type="button"
            onclick={() => move(index, -1)}>↑</button
          >
          <button
            aria-label={`Move stage ${index + 1} down`}
            class="icon-button"
            disabled={index === stages.length - 1}
            type="button"
            onclick={() => move(index, 1)}>↓</button
          >
          <button
            class="text-button danger"
            disabled={stages.length <= 1}
            type="button"
            onclick={() => remove(index)}>Remove</button
          >
        </div>
      </div>
      <input bind:value={stage.title} maxlength="120" placeholder="Stage title" />
      <textarea bind:value={stage.details} rows="3" placeholder="What happens in this stage?"
      ></textarea>
      <div class="stage-dates">
        <label>
          <span>Start date</span>
          <input bind:value={stage.startDate} type="date" />
        </label>
        <label>
          <span>End date</span>
          <input bind:value={stage.endDate} min={stage.startDate || undefined} type="date" />
        </label>
      </div>
    </div>
  {/each}
  <button class="secondary-button" type="button" onclick={add}>Add stage</button>
</div>
