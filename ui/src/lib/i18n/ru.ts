/** Russian (ADR 0114): English text → Russian. A plural entry has one, few and many forms. */
export interface Forms {
  one: string;
  few: string;
  many: string;
  other?: string;
}

export const ru: Record<string, string | Forms> = {};
