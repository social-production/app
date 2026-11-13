/**
 * A message fit to show a person, from any error a flow throws. Zome errors arrive wrapped
 * ("…Guest(\"New members can start…\")…"); the sentence the zome wrote is what people see.
 */
export function explain(error: unknown): string {
  if (error instanceof DOMException && error.name === 'NotAllowedError') {
    return 'The passkey step was cancelled or timed out.';
  }
  const text = error instanceof Error ? error.message : String(error);
  // A deactivated account's directory chain is closed (ADR 0088): signing in, which grants
  // this device a capability there, is refused.
  if (text.includes('No more actions are allowed after a chain close')) return DEACTIVATED;
  const guest = /Guest\("((?:[^"\\]|\\.)*)"\)/.exec(text)?.[1];
  const invalid = /InvalidCommit\(\s*"?((?:[^"\\]|\\.)*)"?\s*\)/.exec(text)?.[1];
  return guest ?? (invalid ? `Not allowed: ${invalid}` : text);
}

export const DEACTIVATED = 'This account was deactivated, so it can no longer sign in.';
