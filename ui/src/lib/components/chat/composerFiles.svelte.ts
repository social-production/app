import { compressChatPhoto, rejectOutgoingAttachment } from './attachmentLimits.ts';

interface PendingFile {
  id: string;
  file: File;
  /** Photos show a thumbnail before sending. */
  previewUrl: string;
}

/** web LiveChatPanel's `pendingAttachments`: the photos and files picked for the next message. */
export class ComposerFiles {
  items = $state<PendingFile[]>([]);
  error = $state('');
  /** Tells this composer's file inputs apart from another chat's on the page. */
  readonly key = crypto.randomUUID();
  /** web's maxPendingAttachments (web-backend MAX_ATTACHMENTS). */
  static readonly MAX = 10;

  get files(): File[] {
    return this.items.map((i) => i.file);
  }

  private remember(file: File, previewUrl = ''): void {
    if (this.items.length >= ComposerFiles.MAX) {
      if (previewUrl) URL.revokeObjectURL(previewUrl);
      this.error = `You can attach up to ${ComposerFiles.MAX} files`;
      return;
    }
    this.items = [...this.items, { id: crypto.randomUUID(), file, previewUrl }];
    this.error = '';
  }

  /** web's choosePhoto: resized to fit 1 MB as a JPEG. */
  async choosePhotos(input: HTMLInputElement): Promise<void> {
    const files = [...(input.files ?? [])];
    input.value = '';
    for (const file of files) {
      const rejection = rejectOutgoingAttachment(file);
      if (rejection) {
        this.error = rejection;
        continue;
      }
      try {
        const compressed = await compressChatPhoto(file);
        this.remember(compressed, URL.createObjectURL(compressed));
      } catch (err) {
        this.error = err instanceof Error ? err.message : 'Could not process image.';
      }
    }
  }

  chooseFiles(input: HTMLInputElement): void {
    const files = [...(input.files ?? [])];
    input.value = '';
    for (const file of files) {
      const rejection = rejectOutgoingAttachment(file);
      if (rejection) this.error = rejection;
      else this.remember(file);
    }
  }

  /** One item, or all of them once sent. */
  release(id?: string): void {
    for (const item of this.items) {
      if ((id === undefined || item.id === id) && item.previewUrl) {
        URL.revokeObjectURL(item.previewUrl);
      }
    }
    this.items = id === undefined ? [] : this.items.filter((i) => i.id !== id);
  }
}
