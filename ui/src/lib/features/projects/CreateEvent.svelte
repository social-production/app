<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import TagSelector from '#lib/components/create/TagSelector.svelte';
  import { CreateState } from './createState.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  /**
   * web's CreateEventPage, public path: basics → tags → location → overview. A public event
   * starts as a proposal; its dates come from the plan members approve (ADR 0057).
   */
  const s = new CreateState('event');

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/events/[id]', { id }));
  }
</script>

<CreateSheet
  title="Create event"
  description="An event gathers people for a planned occasion. A public event starts as a proposal, and its dates are set when members approve its plan."
  steps={s.steps}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel="Publish event"
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label>
        <RequiredLabel>Event title</RequiredLabel>
        <input bind:value={s.title} maxlength="200" aria-required="true" />
      </label>
      <label>
        <RequiredLabel>Description</RequiredLabel>
        <textarea bind:value={s.description} rows="4" aria-required="true"></textarea>
      </label>
      <PolicyNotice event />
    </div>
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <TagSelector label="Tags" tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else if s.stepId === 'location'}
    <LocationIntent
      intent={s.intent}
      place={s.place}
      onIntent={(i) => (s.intent = i)}
      onPlace={(p) => (s.place = p)}
    />
  {:else}
    <div class="form-stack overview">
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('basics'))}>
        <strong>Basics</strong><span>{s.title.trim() || 'Untitled event'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('scope'))}>
        <strong>Tags</strong><span>{s.tags.length} {s.tags.length === 1 ? 'tag' : 'tags'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = s.stepOf('location'))}>
        <strong>Location</strong><span>{s.locationLabel}</span>
      </button>
      <div class="overview-row static">
        <strong>Dates</strong>
        <span>Set when members approve the event's plan</span>
      </div>
      {#if s.message}<p class="status-note">{s.message}</p>{/if}
    </div>
  {/if}
</CreateSheet>
