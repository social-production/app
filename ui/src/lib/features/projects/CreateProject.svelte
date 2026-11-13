<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import { searchFailure, searchPlaces } from '#lib/services/locations.ts';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import { tagCounts } from '#lib/domain/scopes.ts';
  import ScopeTagsStep from './scopes/ScopeTagsStep.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { CREATE_STEPS, CreateState } from './createState.svelte.ts';

  /** web's CreateProjectPage without its Type step (ADR 0059): basics → location → scope
   * (tags) → overview. */
  const s = new CreateState();
  const steps = CREATE_STEPS;

  $effect(() => {
    void s.checkAllowance();
  });

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/projects/[id]', { id }));
  }
</script>

<CreateSheet
  title="New proposal"
  description="A proposal is shared work people can find, join and shape. Its plan decides what it becomes."
  {steps}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel="Publish proposal"
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label>
        <RequiredLabel>Title</RequiredLabel>
        <input bind:value={s.title} maxlength="200" aria-required="true" />
      </label>
      <label>
        <RequiredLabel>Proposal description</RequiredLabel>
        <textarea bind:value={s.description} rows="5" aria-required="true"></textarea>
      </label>
      <p class="helper-text">
        No type yet: it becomes a project, a service or an event when its members approve a plan.
      </p>
      {#if s.allowance !== null}
        <p class="helper-text">
          New members can start 2 proposals a week ({s.allowance} left). Full members have no limit.
        </p>
      {/if}
      <PolicyNotice />
    </div>
  {:else if s.stepId === 'location'}
    <LocationIntent
      intent={s.intent}
      place={s.placeValue}
      search={searchPlaces}
      failure={searchFailure}
      onIntent={(i) => (s.intent = i)}
      onPlace={(p) => s.setPlace(p)}
    />
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <ScopeTagsStep tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else}
    <div class="form-stack overview">
      <button class="overview-row" type="button" onclick={() => (s.step = 0)}>
        <strong>Basics</strong><span>{s.title.trim() || 'Untitled proposal'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 1)}>
        <strong>Location</strong><span>{s.locationLabel}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 2)}>
        <strong>Scope</strong><span>{tagCounts(s.tags)}</span>
      </button>
      {#if s.message}<p class="status-note">{s.message}</p>{/if}
    </div>
  {/if}
</CreateSheet>
