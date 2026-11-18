<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { roleButton, roleLine, type Activity } from '#lib/domain/activities.ts';

  /** An opened activity's roles (Feed Headers and Activity Phase §3): one row each with who is
   * in, a Join or Leave pill (or Full), and Suggest to put someone forward. */
  interface Props {
    activity: Activity;
    names: Record<string, string>;
    me: string;
    readOnly: boolean;
    /** web's historyMode: who held each role, without suggestions. */
    past?: boolean;
    canSuggest: boolean;
    onCommit: (role: number | null) => void;
    onSuggest: (role: number, agent: string) => void;
    onDecline: (suggestion: string) => void;
    searchPeople: (q: string) => Promise<{ agent: string; name: string }[]>;
  }

  let p: Props = $props();

  let suggesting = $state<number | null>(null);
  let query = $state('');
  let results = $state<{ agent: string; name: string }[]>([]);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const nameOf = (agent: string): string => p.names[agent] ?? 'Unnamed';

  function search(value: string): void {
    query = value;
    clearTimeout(timer);
    if (!value.trim()) {
      results = [];
      return;
    }
    timer = setTimeout(() => {
      void p.searchPeople(value.trim()).then((found) => {
        results = found.filter((r) => r.agent !== p.me);
      });
    }, 200);
  }

  function pick(role: number, agent: string): void {
    p.onSuggest(role, agent);
    suggesting = null;
    query = '';
    results = [];
  }
</script>

<section class="sheet-section role-group" aria-label={t('Roles')}>
  <div class="role-list">
    {#each p.activity.roles as role (role.index)}
      {@const mine = p.activity.myRole === role.index}
      {@const label = roleButton(role, mine)}
      {@const suggestable =
        !p.past && !role.suggestion && p.canSuggest && p.activity.rolesOpen && !mine}
      <div class="role-bar">
        <div class="role-copy">
          <strong>{t(role.label)}</strong>
          <span>{roleLine(role, nameOf)}</span>
          {#if !p.past && role.suggestion}
            <span class="suggested-chip">Suggested · {nameOf(role.suggestion.person)}</span>
            {#if role.suggestion.person === p.me}
              {@const id = role.suggestion.id}
              <button class="text-button" type="button" onclick={() => p.onDecline(id)}>
                {t('Decline')}
              </button>
            {/if}
          {/if}
          {#if suggestable && suggesting === role.index}
            <input
              placeholder={t('Search username')}
              type="text"
              value={query}
              oninput={(e) => search(e.currentTarget.value)}
            />
            {#each results as person (person.agent)}
              <button
                class="text-button"
                type="button"
                onclick={() => pick(role.index, person.agent)}
              >
                {person.name}
              </button>
            {/each}
          {/if}
        </div>
        {#if !p.readOnly && !p.activity.rolesOpen}
          <span class="roles-locked-copy">{t('Locked')}</span>
        {:else if label === 'Full'}
          <span class="role-full">{t('Full')}</span>
        {:else if !p.readOnly}
          {#if suggestable}
            <button
              class="role-suggest"
              type="button"
              aria-expanded={suggesting === role.index}
              onclick={() => (suggesting = suggesting === role.index ? null : role.index)}
              >{t('Suggest')}</button
            >
          {/if}
          <!-- One button: Join, then the same spot becomes Leave. -->
          <button
            class="signup"
            class:selected={mine}
            aria-pressed={mine}
            type="button"
            onclick={() => p.onCommit(mine ? null : role.index)}>{t(label)}</button
          >
        {/if}
      </div>
    {/each}
  </div>
</section>
