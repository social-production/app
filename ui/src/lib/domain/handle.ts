/**
 * Usernames, as on web (web/src/lib/utils/handles.ts); the directory DNA enforces the same
 * rule (sp_core::handle). Names aren't unique on a P2P network: the key tag tells people apart.
 */
export const HANDLE_MIN = 3;
export const HANDLE_MAX = 32;

const PATTERN = /^[A-Za-z0-9]+(?:[-_][A-Za-z0-9]+)*$/;
const GENERIC =
  'Use 3–32 characters: letters, numbers, hyphens, or underscores. No spaces, and no leading, trailing, or repeated separators.';

/** `null` when the username is fine, otherwise web's sentence saying what is wrong. */
export function handleProblem(value: string): string | null {
  const handle = value.trim();
  if (!handle) return 'Username is required';
  if (/\s/.test(handle)) return 'Username cannot contain spaces';
  if (handle.length < HANDLE_MIN || handle.length > HANDLE_MAX) {
    return `Username must be ${HANDLE_MIN}–${HANDLE_MAX} characters`;
  }
  return PATTERN.test(handle) ? null : GENERIC;
}
