/**
 * A call someone asked for from a direct chat's header (web's `placeDirectCall`). Messages
 * sets it; the call screen (features/calls, composed by the root layout) places the call
 * and clears it, so neither feature imports the other.
 */
export const callRequest = $state<{
  next: { seed: string; name: string; peer: string | null } | null;
}>({
  next: null,
});
