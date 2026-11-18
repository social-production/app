<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import type { FollowRow } from '#lib/domain/follows.ts';
  import type { Person } from '#lib/domain/people.ts';
  import { displayName } from '#lib/domain/people.ts';

  import { resolve } from '$app/paths';

  /** web's "Follow requests" card on your own profile (Accept / Decline). */
  interface Props {
    requests: FollowRow[];
    people: Record<string, Person>;
    pending: string | null;
    onDecide: (agent: string, accept: boolean) => void;
  }

  let { requests, people, pending, onDecide }: Props = $props();
</script>

{#if requests.length > 0}
  <section id="follow-requests" class="requests-card">
    <h2>{t('Follow requests')}</h2>
    <p class="requests-copy">
      {t('Approve followers who need your permission before they can see personal content.')}
    </p>
    <div class="people-list">
      {#each requests as request (request.agent)}
        {@const person = people[request.agent]}
        <div class="person-row">
          <a class="person-link" href={resolve('/profile/[agent]', { agent: request.agent })}>
            <strong>{person ? displayName(person) : 'Unnamed'}</strong>
            {#if person?.profile?.bio}<span>{person.profile.bio}</span>{/if}
          </a>
          <div class="request-actions">
            <button
              class="accept-button"
              disabled={pending === request.agent}
              type="button"
              onclick={() => onDecide(request.agent, true)}
            >
              {t('Accept')}
            </button>
            <button
              class="reject-button"
              disabled={pending === request.agent}
              type="button"
              onclick={() => onDecide(request.agent, false)}
            >
              {t('Decline')}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </section>
{/if}
