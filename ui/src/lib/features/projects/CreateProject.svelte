<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import TagSelector from '#lib/components/create/TagSelector.svelte';
  import TypeSelector from '#lib/components/create/TypeSelector.svelte';
  import { PROJECT_TYPES } from '#lib/domain/projects.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { CREATE_STEPS, CreateState } from './createState.svelte.ts';

  /** web's CreateProjectPage: type → basics → location → scope (tags) → overview. */
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
  title="Create project"
  description="A project is shared work people can find, join, and take part in."
  {steps}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel="Publish project"
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'type'}
    <div class="form-stack">
      <TypeSelector
        label="Project type"
        name="project-type"
        selected={s.mode}
        options={PROJECT_TYPES}
        onSelect={(mode) => (s.mode = mode)}
      />
      {#if s.allowance !== null}
        <p class="helper-text">
          New members can start 2 projects a week ({s.allowance} left). Full members have no limit.
        </p>
      {/if}
    </div>
  {:else if s.stepId === 'basics'}
    <div class="form-stack">
      <label>
        <RequiredLabel>Title</RequiredLabel>
        <input bind:value={s.title} maxlength="200" aria-required="true" />
      </label>
      <label>
        <RequiredLabel>Proposal description</RequiredLabel>
        <textarea bind:value={s.description} rows="5" aria-required="true"></textarea>
      </label>
      <PolicyNotice />
    </div>
  {:else if s.stepId === 'location'}
    <LocationIntent
      intent={s.intent}
      place={s.place}
      onIntent={(i) => (s.intent = i)}
      onPlace={(p) => (s.place = p)}
    />
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <TagSelector label="Tags" tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else}
    <div class="form-stack overview">
      <button class="overview-row" type="button" onclick={() => (s.step = 0)}>
        <strong>Type</strong><span>{s.typeLabel}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 1)}>
        <strong>Basics</strong><span>{s.title.trim() || 'Untitled project'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 2)}>
        <strong>Location</strong><span>{s.locationLabel}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 3)}>
        <strong>Scope</strong><span>{s.tags.length} {s.tags.length === 1 ? 'tag' : 'tags'}</span>
      </button>
      {#if s.message}<p class="status-note">{s.message}</p>{/if}
    </div>
  {/if}
</CreateSheet>
