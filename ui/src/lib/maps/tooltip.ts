/** web's floating pin tooltip (`maps/maplibre.ts` `showTooltip`/`moveTooltip`/`hideTooltip`). */
import { ensureTooltipStyles } from './markerLook.ts';
import type { Tooltip } from './markerElement.ts';

export class FloatingTooltip implements Tooltip {
  private element: HTMLDivElement | null = null;

  private ensure(): HTMLDivElement {
    ensureTooltipStyles();
    if (!this.element) {
      this.element = document.createElement('div');
      this.element.className = 'sp-map-tooltip';
      this.element.hidden = true;
      document.body.appendChild(this.element);
    }
    return this.element;
  }

  show = (html: string, x: number, y: number): void => {
    const tip = this.ensure();
    tip.innerHTML = html;
    tip.hidden = false;
    tip.style.left = `${x + 14}px`;
    tip.style.top = `${y + 14}px`;
  };

  move = (x: number, y: number): void => {
    if (!this.element || this.element.hidden) return;
    this.element.style.left = `${x + 14}px`;
    this.element.style.top = `${y + 14}px`;
  };

  hide = (): void => {
    if (!this.element) return;
    this.element.hidden = true;
    this.element.innerHTML = '';
  };

  remove(): void {
    this.hide();
    this.element?.remove();
    this.element = null;
  }
}
