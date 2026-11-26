import './graphicsRecovery.css';

export class GraphicsRecovery {
  private notice?: HTMLElement;
  private restoreFocus = false;

  constructor(private canvas: HTMLCanvasElement, private online: boolean, private change: (lost: boolean) => void) {
    canvas.addEventListener('webglcontextlost', this.lost);
    canvas.addEventListener('webglcontextrestored', this.restored);
  }

  get interrupted() { return !!this.notice; }

  private lost = (event: Event) => {
    event.preventDefault();
    if (this.notice) return;
    this.restoreFocus = document.activeElement === this.canvas;
    // Blur sends the existing neutral input immediately, including to a live private room.
    this.canvas.blur();
    this.canvas.inert = true;
    const notice = document.createElement('div');
    notice.className = 'graphics-recovery';
    notice.tabIndex = -1;
    notice.setAttribute('role', 'alert');
    const title = document.createElement('strong');
    title.textContent = 'Graphics interrupted';
    const detail = document.createElement('p');
    detail.textContent = this.online
      ? 'The round is still live. Waiting for your browser to restore the arena.'
      : 'Play is suspended until your browser restores graphics. Reloading starts over.';
    const reload = document.createElement('button');
    reload.type = 'button';
    reload.textContent = this.online ? 'Reload and reconnect' : 'Reload and restart';
    reload.onclick = () => location.reload();
    notice.append(title, detail, reload);
    this.notice = notice;
    this.canvas.after(notice);
    if (this.restoreFocus) notice.focus({preventScroll:true});
    this.change(true);
  };

  private restored = () => {
    if (!this.notice) return;
    const refocus = this.restoreFocus && this.notice.contains(document.activeElement);
    this.canvas.inert = false;
    this.notice.remove();
    this.notice = undefined;
    this.change(false);
    if (refocus) this.canvas.focus({preventScroll:true});
  };

  dispose() {
    this.canvas.removeEventListener('webglcontextlost', this.lost);
    this.canvas.removeEventListener('webglcontextrestored', this.restored);
    this.notice?.remove();
    this.notice = undefined;
  }
}
