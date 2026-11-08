<script lang="ts">
  import PhaseMenu, { type PhaseMenuItem } from '#lib/components/governance/PhaseMenu.svelte';

  /** web's RepositoryMenu: the rarely used repository votes, kept behind one button like Phase ▾. */
  interface Props {
    canReplace: boolean;
    canChangeMerge: boolean;
    onReplace: () => void;
    onChangeMerge: () => void;
  }

  let { canReplace, canChangeMerge, onReplace, onChangeMerge }: Props = $props();

  const items = $derived.by(() => {
    const list: PhaseMenuItem[] = [];
    if (canReplace) {
      list.push({
        id: 'replace-repository',
        glyph: '⇄',
        label: 'Replace repository',
        detail: 'Members vote to move the project to a different repository.',
        onSelect: onReplace,
      });
    }
    if (canChangeMerge) {
      list.push({
        id: 'merge-capability',
        glyph: '⑂',
        label: 'Grant or revoke merge capability',
        detail: 'Members vote on who can merge approved pull requests.',
        onSelect: onChangeMerge,
      });
    }
    return list;
  });
</script>

<PhaseMenu
  {items}
  label="Repository"
  sheetTitle="Repository"
  labelledById="repository-menu-title"
/>
