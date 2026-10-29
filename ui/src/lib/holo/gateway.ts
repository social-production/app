/** The seed node's HTTP API (services/gateway/src/routes). */

export interface Ceremony {
  ceremony: string;
  options: { publicKey: unknown };
}

export interface GatewaySession {
  agent: string;
  appWsUrl: string;
  token: number[];
  capSecret: number[];
  deviceSecret: number[];
  cells: { dna: string; agent: string }[];
}

export interface Resumed {
  agent: string;
  appWsUrl: string;
  token: number[];
}

/** An error the gateway explained in a sentence meant for people. */
export class GatewayError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

async function post<T>(gateway: string, path: string, body: unknown = {}): Promise<T> {
  const response = await fetch(new URL(path, gateway), {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  const json: unknown = await response.json().catch(() => ({}));
  if (!response.ok) {
    // A bare 404 means this page is newer than the node it talks to (mid-deploy).
    const fallback =
      response.status === 404
        ? 'The site is updating. Please try again in a minute.'
        : `The server answered ${response.status}.`;
    const message = (json as { error?: string }).error ?? fallback;
    throw new GatewayError(message, response.status);
  }
  return json as T;
}

export type OpenMode = 'signup' | 'restore' | 'signin';

export interface OpenBody {
  mode: OpenMode;
  challenge?: string;
  nonce?: string;
  seed: number[];
  signingKey: number[];
}

async function get<T>(gateway: string, path: string, query: Record<string, string>): Promise<T> {
  const url = new URL(path, gateway);
  for (const [key, value] of Object.entries(query)) url.searchParams.set(key, value);
  const response = await fetch(url);
  if (!response.ok)
    throw new GatewayError(`The server answered ${response.status}.`, response.status);
  return (await response.json()) as T;
}

export interface CellsBody {
  signingKey: number[];
  deviceSecret: number[];
  capSecret: number[];
  dnaHashes: number[][];
}

export const gatewayApi = {
  pow: (g: string) => post<{ challenge: string; bits: number }>(g, '/account/pow'),
  open: (g: string, body: OpenBody) => post<GatewaySession>(g, '/account/open', body),
  loginStart: (g: string) => post<Ceremony>(g, '/account/login/start'),
  loginFinish: (g: string, body: { ceremony: string; credential: unknown; signingKey: number[] }) =>
    post<GatewaySession>(g, '/account/login/finish', body),
  passkeyStart: (g: string, body: { signingKey: number[]; deviceSecret: number[]; name: string }) =>
    post<Ceremony>(g, '/account/passkey/start', body),
  passkeyFinish: (g: string, body: { ceremony: string; credential: unknown }) =>
    post<{ saved: boolean }>(g, '/account/passkey/finish', body),
  resume: (g: string, signingKey: number[]) => post<Resumed>(g, '/session/resume', { signingKey }),
  cells: (g: string, body: CellsBody) => post<{ granted: number }>(g, '/session/cells', body),
  projects: <T>(g: string, query: Record<string, string>) => get<T>(g, '/index/projects', query),
  people: <T>(g: string, query: Record<string, string>) => get<T>(g, '/index/people', query),
  scopes: <T>(g: string, query: Record<string, string>) => get<T>(g, '/index/scopes', query),
};
