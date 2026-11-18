<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { HelpRoleDraft } from '#lib/domain/helpRequests.ts';

  /**
   * A request's roles (Polish Pass mockups, 2026-11-16): each role a soft card with its title,
   * details and how many people, edited in place; "Add a role" under them.
   */
  interface Props {
    roles: HelpRoleDraft[];
    onSet: (index: number, patch: Partial<HelpRoleDraft>) => void;
    onAdd: () => void;
    onRemove: (index: number) => void;
  }

  let { roles, onSet, onAdd, onRemove }: Props = $props();

  const people = (n: number) => (n === 1 ? 'person' : 'people');
</script>

<div class="roles-step">
  {#each roles as role, index (index)}
    <section class="role-card" aria-label={role.label || `Role ${index + 1}`}>
      <input
        class="role-title"
        value={role.label}
        oninput={(e) => onSet(index, { label: e.currentTarget.value })}
        maxlength="100"
        aria-label={t('Role title')}
        aria-required="true"
        placeholder={t('Role, e.g. Driver')}
      />
      <textarea
        class="role-details"
        value={role.description}
        oninput={(e) => onSet(index, { description: e.currentTarget.value })}
        rows="2"
        aria-label={t('Role details')}
        placeholder={t('What this person does')}></textarea>
      <div class="role-foot">
        <div class="role-count" role="group" aria-label={t('How many people')}>
          <button
            type="button"
            aria-label={t('One fewer')}
            disabled={role.slots <= 0}
            onclick={() => onSet(index, { slots: Math.max(0, role.slots - 1) })}>−</button
          >
          <input
            type="number"
            min="0"
            value={role.slots}
            oninput={(e) => onSet(index, { slots: Number(e.currentTarget.value) || 0 })}
            aria-label={t('People needed')}
          />
          <button
            type="button"
            aria-label={t('One more')}
            onclick={() => onSet(index, { slots: role.slots + 1 })}>+</button
          >
          <span>{people(role.slots)}</span>
        </div>
        {#if roles.length > 1}
          <button class="role-remove" type="button" onclick={() => onRemove(index)}
            >{t('Remove')}</button
          >
        {/if}
      </div>
    </section>
  {/each}
  <button class="role-add" type="button" onclick={onAdd}>{t('+ Add a role')}</button>
</div>
