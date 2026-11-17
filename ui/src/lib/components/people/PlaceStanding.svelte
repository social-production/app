<script lang="ts">
  import type { Person } from '#lib/domain/people.ts';
  import { placeAndStanding } from '#lib/domain/personLine.ts';

  /** A picker row's second line: where they live and their standing (Detail Pages Cleanup §5).
   * `load` reads the person (services' `cachedPerson`). */
  let { agent, load }: { agent: string; load: (agent: string) => Promise<Person> } = $props();
</script>

{#await load(agent) then person}{placeAndStanding(person)}{/await}
