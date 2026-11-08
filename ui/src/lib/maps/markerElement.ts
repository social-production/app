/** web's pin element (`maps/maplibre.ts` `buildMarkerElement`). Links go through `navigate`
 * (the app's router) instead of a page load, which would reconnect to Holochain. */
import { buildTooltipHtml, resolveMarkerColor, truncate } from './markerLook.ts';
import type { MapMarker } from './types.ts';

export interface Tooltip {
  show: (html: string, x: number, y: number) => void;
  move: (x: number, y: number) => void;
  hide: () => void;
}

export interface PinContext {
  tooltip: Tooltip;
  navigate: (href: string) => void;
}

const fromChild = (event: Event): boolean =>
  Boolean((event.target as HTMLElement | null)?.closest?.('.sp-map-marker__child'));

/** One activity row inside a parent's pin. */
function childRowElement(
  child: MapMarker,
  parent: MapMarker,
  childList: HTMLElement,
  highlightedChildId: string | null,
  parentTooltipHtml: string,
  ctx: PinContext,
): HTMLElement {
  const row = document.createElement('span');
  row.className = [
    'sp-map-marker__child',
    child.id === highlightedChildId ? 'sp-map-marker__child--highlighted' : '',
  ]
    .filter(Boolean)
    .join(' ');
  row.dataset.markerId = child.id;
  row.style.setProperty('--sp-map-marker-child-color', resolveMarkerColor(child));
  row.setAttribute('role', 'link');
  row.tabIndex = 0;
  row.textContent = truncate(child.label, 28);
  const html = buildTooltipHtml({ ...child, parentTitle: parent.label });
  row.addEventListener('mouseenter', (event) => {
    event.stopPropagation();
    for (const other of childList.querySelectorAll<HTMLElement>('.sp-map-marker__child')) {
      other.classList.toggle(
        'sp-map-marker__child--highlighted',
        other === row || other.dataset.markerId === highlightedChildId,
      );
    }
    ctx.tooltip.show(html, event.clientX, event.clientY);
  });
  row.addEventListener('mousemove', (event) => {
    event.stopPropagation();
    ctx.tooltip.move(event.clientX, event.clientY);
  });
  row.addEventListener('mouseleave', (event) => {
    event.stopPropagation();
    if (child.id !== highlightedChildId) row.classList.remove('sp-map-marker__child--highlighted');
    ctx.tooltip.show(parentTooltipHtml, event.clientX, event.clientY);
  });
  row.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    if (child.href) ctx.navigate(child.href);
  });
  return row;
}

/** The title, status and activity rows of an expanded pin. */
function fillExpanded(
  element: HTMLElement,
  item: MapMarker,
  highlightedChildId: string | null,
  tooltipHtml: string,
  ctx: PinContext,
): void {
  const title = document.createElement('span');
  title.className = 'sp-map-marker__title sp-map-marker__title--solo';
  title.textContent = truncate(item.label, 32);
  element.appendChild(title);
  if (item.statusText?.trim()) {
    const status = document.createElement('span');
    status.className = 'sp-map-marker__status';
    status.textContent = truncate(item.statusText, 28);
    element.appendChild(status);
  }
  const children = item.children ?? [];
  if (children.length === 0) return;
  const list = document.createElement('span');
  list.className = 'sp-map-marker__children';
  for (const child of children) {
    list.appendChild(childRowElement(child, item, list, highlightedChildId, tooltipHtml, ctx));
  }
  element.appendChild(list);
}

export function buildMarkerElement(
  item: MapMarker,
  compact: boolean,
  highlighted: boolean,
  highlightedChildId: string | null,
  ctx: PinContext,
): HTMLButtonElement {
  const entityType = item.entityType ?? 'event';
  const hasChildren = (item.children ?? []).length > 0;
  const showExpanded = highlighted || !compact || hasChildren;
  const element = document.createElement('button');
  element.type = 'button';
  element.className = [
    'sp-map-marker',
    showExpanded ? '' : 'sp-map-marker--dot',
    highlighted ? 'sp-map-marker--highlighted' : '',
    hasChildren ? 'sp-map-marker--grouped' : '',
    `sp-map-marker--${entityType}`,
  ]
    .filter(Boolean)
    .join(' ');
  element.setAttribute('aria-label', item.label);
  element.dataset.markerId = item.id;
  element.style.setProperty('--sp-map-marker-color', resolveMarkerColor(item));
  const tooltipHtml = buildTooltipHtml(item);
  if (showExpanded) fillExpanded(element, item, highlightedChildId, tooltipHtml, ctx);
  element.addEventListener('mouseenter', (event) => {
    if (!fromChild(event)) ctx.tooltip.show(tooltipHtml, event.clientX, event.clientY);
  });
  element.addEventListener('mousemove', (event) => {
    if (!fromChild(event)) ctx.tooltip.move(event.clientX, event.clientY);
  });
  element.addEventListener('mouseleave', () => ctx.tooltip.hide());
  if (item.href) {
    const href = item.href;
    element.addEventListener('click', (event) => {
      if (fromChild(event)) return;
      event.preventDefault();
      ctx.navigate(href);
    });
  }
  return element;
}
