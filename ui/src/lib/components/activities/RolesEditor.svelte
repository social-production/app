<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
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

<!-- Roles as cards with steppers (Activity creation mockup §3, owner 2026-11-25). -->
<div class="role-cards">
  {#each roles as role, index (index)}
    <div class="role-card">
      <div class="role-card-top">
        <input
          class="role-card-name"
          maxlength="80"
          aria-label={t('Role')}
          placeholder={t('For example: Intake desk')}
          type="text"
          value={role.label}
          oninput={(e) => patch(index, { label: e.currentTarget.value })}
        />
        {#if roles.length > 1}
          <button class="role-card-remove" type="button" onclick={() => remove(index)}
            >{t('Remove')}</button
          >
        {/if}
      </div>
      <div class="role-step">
        <span>{t('Needed')}</span>
        <span class="stepper">
          <button
            type="button"
            aria-label={t('Fewer needed')}
            disabled={role.required <= 1}
            onclick={() => setRequired(index, role.required - 1)}>−</button
          >
          <b aria-live="polite">{role.required}</b>
          <button
            type="button"
            aria-label={t('More needed')}
            onclick={() => setRequired(index, role.required + 1)}>+</button
          >
        </span>
      </div>
      <div class="role-step">
        <span>{t('Up to')}</span>
        <span class="stepper">
          <button
            type="button"
            aria-label={t('Lower the most')}
            disabled={role.maximum === null}
            onclick={() =>
              setMaximum(
                index,
                role.maximum === null || role.maximum <= role.required
                  ? ''
                  : String(role.maximum - 1),
              )}>−</button
          >
          <b aria-live="polite">{role.maximum ?? t('Any')}</b>
          <button
            type="button"
            aria-label={t('Raise the most')}
            onclick={() => setMaximum(index, String((role.maximum ?? role.required) + 1))}>+</button
          >
        </span>
      </div>
      <div class="role-ask">
        {#if role.suggest}
          <span>{t('Asked')} · <b>{role.suggest.name}</b></span>
          <button type="button" onclick={() => patch(index, { suggest: null })}>{t('Clear')}</button
          >
        {:else}
          <input
            aria-label={t('Ask someone (optional)')}
            placeholder={t('Ask someone (optional)')}
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
      </div>
    </div>
  {/each}
  <button class="role-add" type="button" onclick={() => (roles = [...roles, emptyRole()])}
    >+ {t('Add a role')}</button
  >
</div>
