<script lang="ts">
  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import { roleButton, roleCapacity, type Activity } from '#lib/domain/activities.ts';

  import { resolve } from '$app/paths';

  /** The Roles section of web's activity sheet: who holds each role, take/leave, suggestions. */
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

<section class="sheet-section">
  <h3>Roles</h3>
  <div class="role-list">
    {#each p.activity.roles as role (role.index)}
      {@const mine = p.activity.myRole === role.index}
      <div class="role-bar">
        <div class="role-copy">
          <strong>{role.label}</strong>
          <span>{roleCapacity(role)}</span>
          {#if role.people.length > 0}
            <div class="assignee-list">
              {#each role.people as agent (agent)}
                <a class="assignee-row" href={resolve('/profile/[agent]', { agent })}>
                  <AvatarBadge size="sm" name={nameOf(agent)} />
                  <span>{nameOf(agent)}</span>
                </a>
              {/each}
            </div>
          {/if}
          {#if p.past}
            <!-- web shows no suggestions on past activities -->
          {:else if role.suggestion}
            <span class="suggested-chip">Suggested · {nameOf(role.suggestion.person)}</span>
            {#if role.suggestion.person === p.me}
              {@const id = role.suggestion.id}
              <button class="text-button" type="button" onclick={() => p.onDecline(id)}>
                Decline
              </button>
            {/if}
          {:else if p.canSuggest && p.activity.rolesOpen && !mine}
            <button class="text-button" type="button" onclick={() => (suggesting = role.index)}>
              Suggest someone
            </button>
            {#if suggesting === role.index}
              <input
                placeholder="Search username"
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
          {/if}
        </div>
        {#if !p.readOnly && !p.activity.rolesOpen}
          <span class="roles-locked-copy">Locked</span>
        {:else if !p.readOnly}
          {@const label = roleButton(role, mine)}
          <!-- One button: Take role, then the same spot becomes Leave role. -->
          <button
            class="signup"
            class:selected={mine}
            aria-pressed={mine}
            disabled={label === 'Role full'}
            type="button"
            onclick={() => p.onCommit(mine ? null : role.index)}>{label}</button
          >
        {/if}
      </div>
    {/each}
  </div>
</section>
