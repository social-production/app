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
    const message = (json as { error?: string }).error ?? `The server answered ${response.status}.`;
    throw new GatewayError(message, response.status);
  }
  return json as T;
}

export const gatewayApi = {
  pow: (g: string) => post<{ challenge: string; bits: number }>(g, '/account/pow'),
  signup: (g: string, body: SeedBody) => post<Ceremony>(g, '/account/signup', body),
  restore: (g: string, body: SeedBody) => post<Ceremony>(g, '/account/restore', body),
  register: (g: string, body: RegisterBody) => post<GatewaySession>(g, '/account/register', body),
  loginStart: (g: string) => post<Ceremony>(g, '/account/login/start'),
  loginFinish: (g: string, body: LoginBody) =>
    post<GatewaySession>(g, '/account/login/finish', body),
  resume: (g: string, signingKey: number[]) => post<Resumed>(g, '/session/resume', { signingKey }),
};

export interface SeedBody {
  challenge: string;
  nonce: string;
  seed: number[];
  name: string;
}

interface RegisterBody {
  ceremony: string;
  credential: unknown;
  signingKey: number[];
  restore: boolean;
}

interface LoginBody {
  ceremony: string;
  credential: unknown;
  signingKey: number[];
}
