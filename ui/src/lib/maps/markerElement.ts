/** A map pin (Map Redesign, owner 2026-11-17): a round head in its kind's colour with the
 * kind's symbol (the same set as the feed cards' icons), a white ring and a short tail.
 * Proposals and activities are a size smaller, past events are faded, and the selected pin
 * grows and names itself. A tap selects the pin (the map opens its card); it never leaves
 * the map by itself. */
import { KIND_GLYPHS, kindOfMarker } from '#lib/domain/kindGlyphs.ts';
import { buildTooltipHtml, truncate } from './markerLook.ts';
import type { MapMarker } from './types.ts';

export interface Tooltip {
  show: (html: string, x: number, y: number) => void;
  move: (x: number, y: number) => void;
  hide: () => void;
}

export interface PinContext {
  tooltip: Tooltip;
  navigate: (href: string) => void;
  /** A tapped pin: the map selects it and shows its card. */
  select: (markerId: string) => void;
}

export function isPastMarker(item: MapMarker, now = Date.now()): boolean {
  const end = Date.parse(item.endsAt ?? item.scheduledAt ?? '');
  return Number.isFinite(end) && end < now;
}

export function buildMarkerElement(
  item: MapMarker,
  highlighted: boolean,
  ctx: PinContext,
  /** Other things at this exact spot, counted on the pin's badge. */
  sameSpot = 0,
): HTMLButtonElement {
  const kind = kindOfMarker(item);
  const small = kind === 'proposal' || item.entityType === 'activity';
  const element = document.createElement('button');
  element.type = 'button';
  element.className = [
    'sp-pin',
    `sp-pin--${kind}`,
    small && !highlighted ? 'sp-pin--small' : '',
    highlighted ? 'sp-pin--selected' : '',
    isPastMarker(item) ? 'sp-pin--past' : '',
  ]
    .filter(Boolean)
    .join(' ');
  element.setAttribute('aria-label', item.label);
  element.dataset.markerId = item.id;
  element.style.setProperty('--pin', `var(--kind-${kind})`);
  const children = (item.children?.length ?? 0) + sameSpot;
  element.innerHTML =
    (highlighted ? `<span class="sp-pin__name"></span>` : '') +
    `<span class="sp-pin__head"><svg viewBox="0 0 24 24" aria-hidden="true">${KIND_GLYPHS[kind]}</svg>` +
    (children > 0 ? `<span class="sp-pin__count">${children + 1}</span>` : '') +
    `</span><span class="sp-pin__tail"></span>`;
  const name = element.querySelector('.sp-pin__name');
  if (name) name.textContent = truncate(item.label, 32);
  const tooltipHtml = buildTooltipHtml(item);
  element.addEventListener('mouseenter', (event) => {
    if (!highlighted) ctx.tooltip.show(tooltipHtml, event.clientX, event.clientY);
  });
  element.addEventListener('mousemove', (event) => ctx.tooltip.move(event.clientX, event.clientY));
  element.addEventListener('mouseleave', () => ctx.tooltip.hide());
  element.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    ctx.tooltip.hide();
    ctx.select(item.id);
  });
  return element;
}

/** Pins too close to tell apart, as one round count; a tap zooms in on them. */
export function buildClusterElement(count: number, onTap: () => void): HTMLButtonElement {
  const element = document.createElement('button');
  element.type = 'button';
  element.className = 'sp-cluster';
  element.setAttribute('aria-label', `${count} places here. Zoom in`);
  element.textContent = String(count);
  element.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    onTap();
  });
  return element;
}
