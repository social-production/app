export { restore, signIn, signUp, type AccountStep } from './account.ts';
export {
  callZome,
  connect,
  onConnectionChange,
  signOut,
  type ConnectionState,
  type Role,
} from './connection.ts';
export { GatewayError } from './gateway.ts';
export { passkeysSupported } from './passkeys.ts';
export { confirmationPositions, newPhrase, phraseIsValid, splitWords } from './phrase.ts';
export {
  decodeHashFromBase64 as stringToHash,
  encodeHashToBase64 as hashToString,
  type ActionHash,
  type AgentPubKey,
} from '@holochain/client';
