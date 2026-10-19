export {
  addPasskey,
  signInWithPasskey,
  signInWithPhrase,
  signUp,
  type AccountStep,
} from './account.ts';
export { newProjectSeed, projectCell } from './cells.ts';
export { projectFeed } from './feed.ts';
export {
  callCell,
  callZome,
  connect,
  onConnectionChange,
  signOut,
  type ConnectionState,
  type Role,
} from './connection.ts';
export { GatewayError } from './gateway.ts';
export { devicePasskeysAvailable, passkeysSupported } from './passkeys.ts';
export {
  confirmationPositions,
  firstInvalidWord,
  newPhrase,
  phraseIsValid,
  splitWords,
} from './phrase.ts';
export {
  decodeHashFromBase64 as stringToHash,
  encodeHashToBase64 as hashToString,
  type ActionHash,
  type AgentPubKey,
  type CellId,
} from '@holochain/client';
