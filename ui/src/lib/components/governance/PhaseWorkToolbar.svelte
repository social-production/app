<script lang="ts">
  import type { Snippet } from 'svelte';

  import '#lib/components/styles/governance.css';

  /**
   * web's PhaseWorkToolbar: a phase's work actions ("Add plan") and the Phase ▾ menu, moved into
   * the Take part dock (`#detail-participation-actions`) and packed into balanced rows. The
   * phase menu portals itself into `#phase-nav-end`.
   */
  let { children, governance }: { children?: Snippet; governance?: Snippet } = $props();

  const MIN_CELL_WIDTH = 112;
  let node = $state<HTMLDivElement>();

  function place(): void {
    const target = document.getElementById('detail-participation-actions');
    if (target && node && node.parentElement !== target) target.appendChild(node);
  }

  function hasVisibleControl(element: HTMLElement): boolean {
    const controls = element.matches('button, a')
      ? [element]
      : [...element.querySelectorAll<HTMLElement>('button, a')];
    return controls.some((control) => getComputedStyle(control).display !== 'none');
  }

  function collectCells(root: Element, cells: HTMLElement[]): void {
    for (const child of Array.from(root.children)) {
      if (!(child instanceof HTMLElement)) continue;
      const display = getComputedStyle(child).display;
      if (display === 'none') continue;
      if (display === 'contents') {
        collectCells(child, cells);
        continue;
      }
      if (hasVisibleControl(child)) cells.push(child);
    }
  }

  function pack(group: HTMLDivElement): void {
    const cells: HTMLElement[] = [];
    for (const selector of [
      ':scope > .adds',
      ':scope > .governance-row',
      ':scope > #phase-nav-vote',
      ':scope > .phase-nav-bar',
    ]) {
      const root = group.querySelector(selector);
      if (root) collectCells(root, cells);
    }
    group.querySelectorAll<HTMLElement>('.phase-pack-cell').forEach((cell) => {
      if (cells.includes(cell)) return;
      cell.classList.remove('phase-pack-cell', 'row-start', 'wrapped');
      cell.style.flex = '';
      cell.style.maxWidth = '';
      cell.style.order = '';
    });
    // Menus (Phase) go on the bottom row, actions above them.
    const isMenu = (cell: HTMLElement): boolean =>
      !!(cell.querySelector('.phase-menu-trigger') || cell.matches('.phase-menu-trigger'));
    cells.sort((a, b) => Number(isMenu(a)) - Number(isMenu(b)));
    const maxPerRow = Math.max(
      1,
      Math.floor((group.clientWidth || MIN_CELL_WIDTH) / MIN_CELL_WIDTH),
    );
    // Balance rows: four buttons become 2 + 2, not 3 + 1.
    const rows = Math.ceil(cells.length / maxPerRow);
    const perRow = Math.max(1, Math.ceil(cells.length / Math.max(rows, 1)));
    cells.forEach((cell, index) => {
      const rowStart = Math.floor(index / perRow) * perRow;
      const share = 100 / Math.min(perRow, cells.length - rowStart);
      cell.style.order = String(index);
      cell.classList.add('phase-pack-cell');
      cell.classList.toggle('row-start', index % perRow === 0);
      cell.classList.toggle('wrapped', index >= perRow);
      cell.style.flex = `1 1 ${share}%`;
      cell.style.maxWidth = `${share}%`;
    });
  }

  $effect(() => {
    const group = node;
    if (!group) return;
    place();
    let frame = 0;
    const schedule = (): void => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => pack(group));
    };
    const mutations = new MutationObserver(schedule);
    mutations.observe(group, { childList: true, subtree: true });
    const resize = new ResizeObserver(schedule);
    resize.observe(group);
    schedule();
    return () => {
      cancelAnimationFrame(frame);
      mutations.disconnect();
      resize.disconnect();
      group.remove();
    };
  });
</script>

<div class="phase-action-group" bind:this={node}>
  <div class="adds">{@render children?.()}</div>
  <div class="governance-row">{@render governance?.()}</div>
  <div id="phase-nav-vote" class="phase-action-slot"></div>
  <div class="phase-nav-bar">
    <div id="phase-nav-start" class="phase-action-slot"></div>
    <div id="phase-nav-end" class="phase-action-slot"></div>
  </div>
</div>
