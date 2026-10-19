/**
 * Move a node to <body> so fixed overlays escape card filters and transforms
 * (web/src/lib/utils/portal.ts). Svelte may re-insert it on update, so update re-appends.
 */
export function portal(node: HTMLElement): { update: () => void; destroy: () => void } {
  const mount = (): void => {
    if (node.parentElement !== document.body) document.body.appendChild(node);
  };
  mount();
  return { update: mount, destroy: () => node.remove() };
}
