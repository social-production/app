/** web's utils/mentions.ts: the "@na" being typed before the caret, and choosing a name. */

export function activeMention(value: string, cursor: number) {
  const upto = value.slice(0, Math.max(0, cursor));
  const match = upto.match(/(^|[^\w])@([A-Za-z0-9_-]{1,32})$/);
  const query = match?.[2];
  if (!query) return null;
  return { query, start: cursor - query.length - 1 };
}

export function insertMention(value: string, start: number, cursor: number, username: string) {
  const next = `${value.slice(0, start)}@${username} ${value.slice(cursor)}`;
  return { value: next, caret: start + username.length + 2 };
}
