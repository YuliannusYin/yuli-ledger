export function yTickValues(maxMinor: number): number[] {
  const max = Math.max(maxMinor, 0);
  if (max === 0) return [0];
  const raw = [0, Math.round(max / 3), Math.round((2 * max) / 3), max];
  return [...new Set(raw)];
}

export function pickIndices(n: number, maxTicks: number): number[] {
  if (n <= 0) return [];
  if (n <= maxTicks) return Array.from({ length: n }, (_, i) => i);
  const out = new Set<number>();
  for (let i = 0; i < maxTicks; i++) {
    out.add(Math.round((i * (n - 1)) / (maxTicks - 1)));
  }
  return [...out].sort((a, b) => a - b);
}

export function shortAxisLabel(key: string, mode: string, series: "trend" | "comparison", monthly = false): string {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(key)) return key;
  if (series === "comparison") {
    if (mode === "year") return key.slice(0, 4);
    if (mode === "month") return key.slice(0, 7);
    return key.slice(5);
  }
  if (mode === "year" || monthly) return key.slice(0, 7);
  return key.slice(5);
}

export const CHART_MARGIN = { top: 12, right: 12, bottom: 28, left: 52 };
