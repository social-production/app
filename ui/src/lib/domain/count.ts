/** web's formatCompactCount (utils/compactCount.ts): 999, 1.2k, 12k, 1.5m. */
export function formatCompactCount(value: number): string {
  const count = Math.max(0, Math.round(value));
  if (count < 1000) return String(count);
  if (count < 10_000) {
    const thousands = (Math.round(count / 100) * 100) / 1000;
    return thousands % 1 === 0 ? `${thousands.toFixed(0)}k` : `${thousands.toFixed(1)}k`;
  }
  if (count < 1_000_000) {
    const thousands = Math.round(count / 1000);
    return thousands >= 1000 ? '1m' : `${thousands}k`;
  }
  const millions = (Math.round(count / 100_000) * 100_000) / 1_000_000;
  return millions % 1 === 0 ? `${millions.toFixed(0)}m` : `${millions.toFixed(1)}m`;
}
