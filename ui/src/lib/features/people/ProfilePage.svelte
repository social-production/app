<script lang="ts">
  import { connection } from '#lib/stores/connection.svelte.ts';

  import DevicesPanel from './DevicesPanel.svelte';
  import ProfileEditor from './ProfileEditor.svelte';
</script>

<div class="stack">
  <ProfileEditor />
  {#if connection.current.status === 'connected'}
    <DevicesPanel me={connection.current.agent} />
  {/if}
</div>
