/**
 * Move a node into `target` (default <body>) so fixed overlays escape card filters and
 * transforms, or so a control can sit in another component's slot (web/src/lib/utils/portal.ts).
 * `false` leaves the node where it is. Svelte may re-insert it on update, so update re-appends.
 */
export function portal(
  node: HTMLElement,
  target: HTMLElement | false = document.body,
): { update: (next?: HTMLElement | false) => void; destroy: () => void } {
  let current = target;
  const mount = (): void => {
    if (current && node.parentElement !== current) current.appendChild(node);
  };
  mount();
  return {
    update: (next = document.body) => {
      current = next;
      mount();
    },
    destroy: () => {
      if (current) node.remove();
    },
  };
}
