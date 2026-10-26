<script lang="ts">
  import type { ActivityRating } from '#lib/domain/activities.ts';

  /** web's HistoryRatingComposer (flat): your saved stars and comment, or the 1–5 editor. */
  interface Props {
    saved: ActivityRating | null;
    onSave: (stars: number, comment: string) => Promise<void>;
    onRemove: () => Promise<void>;
  }

  let { saved, onSave, onRemove }: Props = $props();

  let stars = $state(0);
  let comment = $state('');
  let saving = $state(false);
  let removing = $state(false);

  async function save(): Promise<void> {
    if (stars < 1 || stars > 5) return;
    saving = true;
    try {
      await onSave(stars, comment.trim());
      stars = 0;
      comment = '';
    } finally {
      saving = false;
    }
  }

  async function remove(): Promise<void> {
    removing = true;
    try {
      await onRemove();
    } finally {
      removing = false;
    }
  }
</script>

{#if saved}
  <div class="rating-saved rating-flat">
    <div class="saved-stars" aria-label={`Your rating: ${saved.stars} out of 5 stars`}>
      {#each [1, 2, 3, 4, 5] as star (star)}
        <span class:selected={star <= saved.stars} class="saved-star">{star}★</span>
      {/each}
    </div>
    {#if saved.comment}<p class="saved-comment">{saved.comment}</p>{/if}
    <button class="text-button" disabled={removing} type="button" onclick={remove}>
      {removing ? 'Removing…' : 'Remove rating'}
    </button>
  </div>
{:else}
  <div class="rating-editor rating-flat">
    <p class="rating-note">Ratings are separate from completion check-in.</p>
    <div class="star-row" role="group" aria-label="Rate this activity from 1 to 5 stars">
      {#each [1, 2, 3, 4, 5] as star (star)}
        <button
          class:selected={stars === star}
          class="star-button"
          type="button"
          onclick={() => (stars = star)}
        >
          {star}★
        </button>
      {/each}
    </div>
    <textarea bind:value={comment} maxlength="2000" placeholder="Optional comment" rows="3"
    ></textarea>
    <div class="rating-actions">
      <button class="secondary-button" disabled={stars < 1 || saving} type="button" onclick={save}>
        {saving ? 'Saving…' : 'Save rating'}
      </button>
    </div>
  </div>
{/if}
