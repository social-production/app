import type { Heartbeat } from '#lib/domain/health.ts';
import { callZome, hashToString, type ActionHash, type Role } from '#lib/holo/index.ts';

export function ping(role: Role = 'directory'): Promise<string> {
  return callZome<string>(role, 'health', 'ping');
}

export async function postHeartbeat(note: string, role: Role = 'directory'): Promise<Heartbeat> {
  const hash = await callZome<ActionHash>(role, 'health', 'post_heartbeat', note);
  return { hash: hashToString(hash) };
}

export async function listHeartbeats(role: Role = 'directory'): Promise<Heartbeat[]> {
  const hashes = await callZome<ActionHash[]>(role, 'health', 'list_heartbeats');
  return hashes.map((hash) => ({ hash: hashToString(hash) }));
}
