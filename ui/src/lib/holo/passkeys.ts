/**
 * Browser side of passkeys. The gateway sends standard WebAuthn JSON options; modern
 * browsers parse and serialise them natively (`parse*OptionsFromJSON`, `toJSON`).
 */
type JsonOptions = { publicKey: unknown };

export function passkeysSupported(): boolean {
  return (
    typeof PublicKeyCredential !== 'undefined' &&
    typeof PublicKeyCredential.parseCreationOptionsFromJSON === 'function'
  );
}

/** True when this device has its own passkey store (Windows Hello, Touch ID, Android…). */
export async function devicePasskeysAvailable(): Promise<boolean> {
  if (!passkeysSupported()) return false;
  try {
    return await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
  } catch {
    return false;
  }
}

export async function createPasskey(options: JsonOptions): Promise<unknown> {
  const publicKey = PublicKeyCredential.parseCreationOptionsFromJSON(
    options.publicKey as PublicKeyCredentialCreationOptionsJSON,
  );
  const credential = await navigator.credentials.create({ publicKey });
  if (!(credential instanceof PublicKeyCredential)) throw new Error('No passkey was created.');
  return credential.toJSON();
}

export async function usePasskey(options: JsonOptions): Promise<unknown> {
  const publicKey = PublicKeyCredential.parseRequestOptionsFromJSON(
    options.publicKey as PublicKeyCredentialRequestOptionsJSON,
  );
  const credential = await navigator.credentials.get({ publicKey });
  if (!(credential instanceof PublicKeyCredential)) throw new Error('No passkey was chosen.');
  return credential.toJSON();
}
