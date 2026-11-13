<script lang="ts">
  /** web's PlatformBoardExplainerCard: how moderator roles work. */
  let { open = $bindable(false), showToggle = true }: { open?: boolean; showToggle?: boolean } =
    $props();
</script>

<div class="role-rules">
  {#if showToggle}
    <button class="rules-toggle" type="button" onclick={() => (open = !open)}>
      {open ? 'Hide how moderator roles work' : 'How moderator roles work'}
    </button>
  {/if}
  {#if open}
    <ul>
      <li>
        Moderators execute platform outcomes only after users approve them. They do not gate
        creation or decide platform direction on their own.
      </li>
      <li>
        Moderator standing follows the weekly active platform vote context rather than raw member
        counts.
      </li>
      <li>
        Roles that keep approval but temporarily fall short on required votes enter a one-week grace
        period before removal.
      </li>
    </ul>
  {/if}
</div>
