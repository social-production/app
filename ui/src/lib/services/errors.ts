/**
 * A message fit to show a person, from any error a flow throws. Zome errors arrive wrapped
 * ("…Guest(\"New members can start…\")…"); the sentence the zome wrote is what people see.
 */
export function explain(error: unknown): string {
  if (error instanceof DOMException && error.name === 'NotAllowedError') {
    return 'The passkey step was cancelled or timed out.';
  }
  const text = error instanceof Error ? error.message : String(error);
  const guest = /Guest\("((?:[^"\\]|\\.)*)"\)/.exec(text)?.[1];
  const invalid = /InvalidCommit\(\s*"?((?:[^"\\]|\\.)*)"?\s*\)/.exec(text)?.[1];
  return guest ?? (invalid ? `Not allowed: ${invalid}` : text);
}
