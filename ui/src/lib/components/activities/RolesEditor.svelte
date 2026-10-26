<script lang="ts">
  import RoundPlusButton from '#lib/components/RoundPlusButton.svelte';
  import { emptyRole, type RoleDraft } from '#lib/domain/activityForm.ts';

  /** web's ProjectActivityRolesEditor: one row per role, with an optional suggested person. */
  interface Props {
    roles: RoleDraft[];
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
  }

  let { roles = $bindable(), searchPeople }: Props = $props();

  let queries = $state<Record<number, string>>({});
  let results = $state<Record<number, { agent: string; name: string }[]>>({});
  const timers: Record<number, ReturnType<typeof setTimeout>> = {};

  function patch(index: number, change: Partial<RoleDraft>): void {
    roles = roles.map((r, i) => (i === index ? { ...r, ...change } : r));
  }

  function setRequired(index: number, value: number): void {
    const required = Number.isFinite(value) ? Math.max(1, Math.floor(value)) : 1;
    const max = roles[index]?.maximum ?? null;
    patch(index, { required, maximum: max === null ? null : Math.max(max, required) });
  }

  function setMaximum(index: number, value: string): void {
    const parsed = Number(value.trim());
    const required = roles[index]?.required ?? 1;
    const maximum =
      value.trim() && Number.isFinite(parsed) ? Math.max(required, Math.floor(parsed)) : null;
    patch(index, { maximum });
  }

  function remove(index: number): void {
    const rest = roles.filter((_, i) => i !== index);
    roles = rest.length > 0 ? rest : [emptyRole()];
  }

  function search(index: number, value: string): void {
    queries = { ...queries, [index]: value };
    clearTimeout(timers[index]);
    if (!value.trim()) {
      results = { ...results, [index]: [] };
      return;
    }
    timers[index] = setTimeout(() => {
      void searchPeople(value.trim()).then((found) => (results = { ...results, [index]: found }));
    }, 200);
  }

  function pick(index: number, person: { agent: string; name: string }): void {
    patch(index, { suggest: person });
    results = { ...results, [index]: [] };
  }
</script>

<div class="role-editor-shell">
  <div class="role-editor-header">
    <div>
      <span class="field-inline-label">Roles needed</span>
      <p class="role-editor-note">
        Add one row per role. Set the minimum needed, and leave max blank if that role has no cap.
      </p>
    </div>
  </div>

  <div class="role-row-stack">
    {#each roles as role, index (index)}
      <div class="role-row">
        <label class="role-field role-name-field">
          <span class="field-inline-label">Role</span>
          <input
            maxlength="80"
            placeholder="For example: Intake desk"
            type="text"
            value={role.label}
            oninput={(e) => patch(index, { label: e.currentTarget.value })}
          />
        </label>
        <label class="role-field role-count-field">
          <span class="field-inline-label">Minimum</span>
          <input
            min="1"
            type="number"
            value={role.required}
            oninput={(e) => setRequired(index, e.currentTarget.valueAsNumber)}
          />
        </label>
        <label class="role-field role-count-field">
          <span class="field-inline-label">Maximum</span>
          <input
            min={role.required}
            placeholder="No cap"
            type="number"
            value={role.maximum ?? ''}
            oninput={(e) => setMaximum(index, e.currentTarget.value)}
          />
        </label>
        <div class="role-remove-cell">
          {#if roles.length > 1}
            <button class="secondary-button" type="button" onclick={() => remove(index)}
              >Remove</button
            >
          {/if}
        </div>
      </div>
      <label class="role-field role-suggest-field">
        <span class="field-inline-label">Suggest someone (optional)</span>
        {#if role.suggest}
          <div class="suggested-chip-row">
            <span class="suggested-chip">Suggested · {role.suggest.name}</span>
            <button
              class="text-button"
              type="button"
              onclick={() => patch(index, { suggest: null })}>Clear</button
            >
          </div>
        {:else}
          <input
            placeholder="Search username"
            type="text"
            value={queries[index] ?? ''}
            oninput={(e) => search(index, e.currentTarget.value)}
          />
          {#if (results[index] ?? []).length > 0}
            <div class="suggestion-list">
              {#each results[index] ?? [] as person (person.agent)}
                <button class="suggestion-item" type="button" onclick={() => pick(index, person)}
                  >{person.name}</button
                >
              {/each}
            </div>
          {/if}
        {/if}
      </label>
    {/each}
  </div>
  <div class="role-add-row">
    <RoundPlusButton label="Add role" onclick={() => (roles = [...roles, emptyRole()])} />
  </div>
</div>
