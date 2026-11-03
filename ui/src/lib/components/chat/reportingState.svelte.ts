import type { ChatLine } from '#lib/domain/chat.ts';

import type { ChatReports } from './chatReports.ts';

/** The bubble menu's Report (the composer's message) and votes in a space's chat (ADR 0071). */
export class ReportingState {
  target = $state<ChatLine | null>(null);
  /** Why the last vote was refused (web toasts it; here it shows above the composer). */
  note = $state('');

  constructor(
    private readonly reports: () => ChatReports | undefined,
    private readonly closeMenu: () => void,
  ) {}

  open = (m: ChatLine): void => {
    this.closeMenu();
    if (this.reports()?.signedIn()) this.target = m;
  };

  dismiss = (): void => {
    this.target = null;
  };

  vote = (m: ChatLine, report: string, yes: boolean): void => {
    this.closeMenu();
    this.note = '';
    const reports = this.reports();
    void reports?.vote(m.id, report, yes).catch((e: unknown) => {
      this.note = reports.explain(e);
    });
  };
}
