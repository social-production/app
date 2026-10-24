import type { ChatLine } from '#lib/domain/chat.ts';

import { menuPlacement } from './chatViewport.ts';

/** web LiveChatPanel's bubble menu: where it is open, and the tap that closes it (which must
 * not open another one). */
export class BubbleMenuState {
  open = $state<{ message: ChatLine; x: number; y: number } | null>(null);
  private block = false;

  show = (message: ChatLine, event: MouseEvent): void => {
    const target = event.target;
    if (target instanceof Element && target.closest('a, button, input, textarea, label')) return;
    if (this.block || this.open) {
      this.block = false;
      this.open = null;
      return;
    }
    this.open = { message, ...menuPlacement(event) };
  };

  close = (): void => {
    this.open = null;
  };

  /** A press outside the menu closes it. */
  pointerDown = (event: PointerEvent): void => {
    if (!this.open) return;
    if (event.target instanceof Element && event.target.closest('.bubble-menu')) return;
    this.open = null;
    this.block = true;
  };

  clicked = (): void => {
    this.block = false;
  };
}
