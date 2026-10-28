import {
  decodeHashFromBase64,
  encodeHashToBase64,
  type ActionHash,
  type CellId,
} from '@holochain/client';

/**
 * Spaces in the commons (ADR 0065): every public project, event and help request is a
 * record in one shared network, the project role's base cell. A directory listing names one
 * with the seed `<creator>.space.<charter hash>`, and every call about it names the space.
 */

/** The word in a seed that marks a space in the commons (project_integrity::SPACE_SEED_WORD). */
const SPACE = 'space';

/** The space a seed names (`<creator>.space.<charter hash>`), if it names one. */
export function spaceInSeed(networkSeed: string): ActionHash | undefined {
  const [, word, id, ...rest] = networkSeed.split('.');
  if (word !== SPACE || !id || rest.length) return undefined;
  try {
    return decodeHashFromBase64(id);
  } catch {
    return undefined;
  }
}

/** The seed naming a space founded by `creator` in the commons (directory listings use it). */
export function spaceSeed(creator: string, space: ActionHash): string {
  return `${creator}.${SPACE}.${encodeHashToBase64(space)}`;
}

/** Stand-in cell ids for spaces: the commons cell's id, tagged with a space. */
const spaces = new WeakMap<CellId, ActionHash>();

export function tagSpace(cell: CellId, space: ActionHash): void {
  spaces.set(cell, space);
}

/** The space a call through `cell` is about, when it is a space in the commons. */
export function spaceOf(cell: CellId): ActionHash | undefined {
  return spaces.get(cell);
}
