<script lang="ts">
  import CreateSheet from '#lib/components/create/CreateSheet.svelte';
  import LocationIntent from '#lib/components/create/LocationIntent.svelte';
  import PolicyNotice from '#lib/components/create/PolicyNotice.svelte';
  import RequiredLabel from '#lib/components/create/RequiredLabel.svelte';
  import TagSelector from '#lib/components/create/TagSelector.svelte';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { HELP_STEPS, HelpCreateState } from './helpCreateState.svelte.ts';

  /**
   * web's CreateHelpRequestPage: basics → location → timing → scope → roles → overview
   * (ADR 0067). Scope is the app's tags until channels and communities exist (M5d).
   */
  const s = new HelpCreateState();
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;

  async function publish(): Promise<void> {
    const id = await s.submit();
    if (id) await goto(resolve('/help-requests/[id]', { id }));
  }

  function setRole(index: number, patch: Partial<(typeof s.roles)[number]>): void {
    s.roles = s.roles.map((r, i) => (i === index ? { ...r, ...patch } : r));
  }
</script>

<CreateSheet
  title="Ask for help"
  description="A help request asks people to take a role for a specific need, with a time and a place."
  steps={[...HELP_STEPS]}
  stepIndex={s.step}
  missing={s.missing}
  submitLabel="Publish help request"
  submitting={s.submitting}
  onStep={(i) => (s.step = i)}
  onSubmit={() => void publish()}
  onClose={() => void goto(resolve('/'))}
>
  {#if s.stepId === 'basics'}
    <div class="form-stack">
      <label class="field">
        <RequiredLabel>Title</RequiredLabel>
        <input
          bind:value={s.title}
          maxlength="200"
          aria-required="true"
          placeholder="e.g. Need a lift to the station"
        />
      </label>
      <label class="field">
        <RequiredLabel>What do you need?</RequiredLabel>
        <textarea
          bind:value={s.body}
          rows="4"
          aria-required="true"
          placeholder="Describe the one-off help needed."></textarea>
      </label>
      <PolicyNotice request />
    </div>
  {:else if s.stepId === 'location'}
    <div class="form-stack">
      <LocationIntent
        intent={s.intent}
        place={s.place}
        allowLater={false}
        helperText="Help requests need a physical or online location so people know where to show up."
        onIntent={(i) => (s.intent = i)}
        onPlace={(p) => (s.place = p)}
      />
    </div>
  {:else if s.stepId === 'timing'}
    <div class="form-stack">
      <div class="field-grid">
        <label class="field">
          <RequiredLabel>Start time</RequiredLabel>
          <input
            value={s.neededAt}
            oninput={(e) => s.setNeededAt(e.currentTarget.value)}
            aria-required="true"
            type="datetime-local"
          />
        </label>
        <label class="field">
          <RequiredLabel>End time</RequiredLabel>
          <input
            bind:value={s.endsAt}
            min={s.neededAt || undefined}
            aria-required="true"
            type="datetime-local"
          />
        </label>
      </div>
      <p class="zone-note">
        Times are in {zone}, your display time zone. Everyone sees them in their own.
      </p>
    </div>
  {:else if s.stepId === 'scope'}
    <div class="form-stack">
      <TagSelector label="Tags" tags={s.tags} onChange={(t) => (s.tags = t)} />
    </div>
  {:else if s.stepId === 'roles'}
    <section class="roles-block">
      <div class="roles-header">
        <RequiredLabel>Roles needed</RequiredLabel>
        <button class="ghost-button" type="button" onclick={() => s.addRole()}>Add role</button>
      </div>
      {#each s.roles as role, index (index)}
        <div class="role-card">
          <label class="field">
            <RequiredLabel>Role title</RequiredLabel>
            <input
              value={role.label}
              oninput={(e) => setRole(index, { label: e.currentTarget.value })}
              maxlength="100"
              aria-required="true"
              placeholder="e.g. Driver"
            />
          </label>
          <label class="field">
            <span class="field-label">Role details</span>
            <textarea
              value={role.description}
              oninput={(e) => setRole(index, { description: e.currentTarget.value })}
              rows="2"></textarea>
          </label>
          <label class="field">
            <RequiredLabel>Slots needed</RequiredLabel>
            <input
              type="number"
              min="0"
              value={role.slots}
              oninput={(e) => setRole(index, { slots: Number(e.currentTarget.value) || 0 })}
              aria-required="true"
            />
          </label>
          {#if s.roles.length > 1}
            <button class="ghost-button danger" type="button" onclick={() => s.removeRole(index)}
              >Remove role</button
            >
          {/if}
        </div>
      {/each}
    </section>
  {:else}
    <div class="form-stack overview">
      <button class="overview-row" type="button" onclick={() => (s.step = 0)}>
        <strong>Basics</strong><span>{s.title.trim() || 'Untitled help request'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 1)}>
        <strong>Location</strong><span>{s.locationLabel}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 2)}>
        <strong>Timing</strong><span>{s.schedule}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 3)}>
        <strong>Scope</strong><span>{s.tags.length} {s.tags.length === 1 ? 'tag' : 'tags'}</span>
      </button>
      <button class="overview-row" type="button" onclick={() => (s.step = 4)}>
        <strong>Roles</strong>
        <span>{s.namedRoles.length} {s.namedRoles.length === 1 ? 'role' : 'roles'}</span>
      </button>
      {#if s.allProblems.length > 0}
        <ul class="status-message-list">
          {#each s.allProblems as problem (problem)}<li>{problem}</li>{/each}
        </ul>
      {:else if s.message}
        <p class="status-message">{s.message}</p>
      {/if}
    </div>
  {/if}
</CreateSheet>
