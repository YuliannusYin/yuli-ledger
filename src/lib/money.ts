export function parseCny(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed || trimmed.startsWith("-") || trimmed.startsWith("+")) return null;
  const parts = trimmed.split(".");
  if (parts.length > 2) return null;
  const whole = parts[0];
  const frac = parts[1] ?? "";
  if (!whole || !/^\d+$/.test(whole)) return null;
  if (frac.length > 2 || (frac.length > 0 && !/^\d+$/.test(frac))) return null;
  const wholeVal = Number(whole);
  const fracVal = Number(frac.padEnd(2, "0") || "0");
  if (!Number.isSafeInteger(wholeVal)) return null;
  return wholeVal * 100 + fracVal;
}

export function formatMinor(minor: number, locale: string, signed = false): string {
  const abs = Math.abs(minor);
  const n = abs / 100;
  const body = new Intl.NumberFormat(locale === "zh-Hans" ? "zh-CN" : "en-US", {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(n);
  if (!signed) return body;
  if (minor === 0) return body;
  const sign = minor < 0 ? "−" : "+";
  return `${sign}${body}`;
}

export function formatInput(minor: number): string {
  const abs = Math.abs(minor);
  return `${Math.floor(abs / 100)}.${String(abs % 100).padStart(2, "0")}`;
}
