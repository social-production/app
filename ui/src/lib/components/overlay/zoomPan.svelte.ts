/** PhotoViewer's zoom (1–4×) and pan, from web's components/shared/PhotoViewer. */
export class ZoomPan {
  scale = $state(1);
  x = $state(0);
  y = $state(0);
  image: HTMLImageElement | null = null;
  private pinch = { distance: 0, scale: 1, midX: 0, midY: 0, x: 0, y: 0 };
  private drag: { x: number; y: number; tx: number; ty: number } | null = null;

  reset(): void {
    this.scale = 1;
    this.x = 0;
    this.y = 0;
    this.pinch.distance = 0;
    this.drag = null;
  }

  private clampPan(): void {
    const img = this.image;
    if (!img || this.scale <= 1) {
      this.x = 0;
      this.y = 0;
      return;
    }
    const maxX = Math.max(0, (img.offsetWidth * (this.scale - 1)) / 2);
    const maxY = Math.max(0, (img.offsetHeight * (this.scale - 1)) / 2);
    this.x = Math.min(maxX, Math.max(-maxX, this.x));
    this.y = Math.min(maxY, Math.max(-maxY, this.y));
  }

  private zoomTo(scale: number): void {
    this.scale = Math.min(4, Math.max(1, scale));
  }

  wheel(event: WheelEvent): void {
    event.preventDefault();
    this.zoomTo(this.scale + (event.deltaY < 0 ? 0.2 : -0.2));
    this.clampPan();
  }

  pointerDown(event: PointerEvent): void {
    if (event.pointerType === 'touch' || this.scale <= 1 || event.button !== 0) return;
    this.drag = { x: event.clientX, y: event.clientY, tx: this.x, ty: this.y };
    (event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
  }

  pointerMove(event: PointerEvent): void {
    if (!this.drag) return;
    this.x = this.drag.tx + (event.clientX - this.drag.x);
    this.y = this.drag.ty + (event.clientY - this.drag.y);
    this.clampPan();
  }

  pointerUp(): void {
    this.drag = null;
  }

  touchStart(event: TouchEvent): void {
    const [first, second] = [event.touches[0], event.touches[1]];
    if (first && second) {
      this.drag = null;
      this.pinch = {
        distance: Math.hypot(first.clientX - second.clientX, first.clientY - second.clientY),
        scale: this.scale,
        midX: (first.clientX + second.clientX) / 2,
        midY: (first.clientY + second.clientY) / 2,
        x: this.x,
        y: this.y,
      };
      return;
    }
    if (first && this.scale > 1) {
      this.drag = { x: first.clientX, y: first.clientY, tx: this.x, ty: this.y };
    }
  }

  touchMove(event: TouchEvent): void {
    const [first, second] = [event.touches[0], event.touches[1]];
    if (first && second && this.pinch.distance > 0) {
      event.preventDefault();
      const distance = Math.hypot(first.clientX - second.clientX, first.clientY - second.clientY);
      this.zoomTo(this.pinch.scale * (distance / this.pinch.distance));
      this.x = this.pinch.x + ((first.clientX + second.clientX) / 2 - this.pinch.midX);
      this.y = this.pinch.y + ((first.clientY + second.clientY) / 2 - this.pinch.midY);
      this.clampPan();
      return;
    }
    if (first && !second && this.drag && this.scale > 1) {
      event.preventDefault();
      this.x = this.drag.tx + (first.clientX - this.drag.x);
      this.y = this.drag.ty + (first.clientY - this.drag.y);
      this.clampPan();
    }
  }

  touchEnd(event: TouchEvent): void {
    if (event.touches.length < 2) this.pinch.distance = 0;
    if (event.touches.length === 0) this.drag = null;
  }
}
