export type LocalParts = {
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
};

export function pad(n: number): string {
  return String(n).padStart(2, "0");
}

export function localParts(d = new Date()): LocalParts {
  return {
    year: d.getFullYear(),
    month: d.getMonth() + 1,
    day: d.getDate(),
    hour: d.getHours(),
    minute: d.getMinutes(),
  };
}

export function toUtcIso(parts: LocalParts): string {
  const local = new Date(
    parts.year,
    parts.month - 1,
    parts.day,
    parts.hour,
    parts.minute,
    0,
    0,
  );
  return `${local.getUTCFullYear()}-${pad(local.getUTCMonth() + 1)}-${pad(local.getUTCDate())}T${pad(local.getUTCHours())}:${pad(local.getUTCMinutes())}:00Z`;
}

export function fromUtcIso(iso: string): LocalParts {
  return localParts(new Date(iso));
}

export function partsToDateInput(parts: LocalParts): string {
  return `${parts.year}-${pad(parts.month)}-${pad(parts.day)}`;
}

export function partsToTimeInput(parts: LocalParts): string {
  return `${pad(parts.hour)}:${pad(parts.minute)}`;
}

export function applyDateInput(parts: LocalParts, value: string): LocalParts {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return parts;
  return {
    ...parts,
    year: Number(match[1]),
    month: Number(match[2]),
    day: Number(match[3]),
  };
}

export function applyTimeInput(parts: LocalParts, value: string): LocalParts {
  const match = /^(\d{1,2}):(\d{2})/.exec(value);
  if (!match) return parts;
  const hour = Number(match[1]);
  const minute = Number(match[2]);
  if (hour > 23 || minute > 59) return parts;
  return { ...parts, hour, minute };
}

export function localDateString(d = new Date()): string {
  const p = localParts(d);
  return `${p.year}-${pad(p.month)}-${pad(p.day)}`;
}

export function monthRange(anchor = new Date()): { from: string; to: string } {
  const y = anchor.getFullYear();
  const m = anchor.getMonth();
  const from = new Date(y, m, 1);
  const to = new Date(y, m + 1, 0);
  return { from: localDateString(from), to: localDateString(to) };
}

export function formatLocalDateTime(iso: string, locale: string): string {
  const d = new Date(iso);
  return new Intl.DateTimeFormat(locale === "zh-Hans" ? "zh-CN" : "en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(d);
}

export function formatLocalDate(isoOrDate: string, locale: string): string {
  const d = isoOrDate.length === 10 ? new Date(`${isoOrDate}T00:00:00`) : new Date(isoOrDate);
  return new Intl.DateTimeFormat(locale === "zh-Hans" ? "zh-CN" : "en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(d);
}

export function shiftDate(dateStr: string, days: number): string {
  const d = new Date(`${dateStr}T00:00:00`);
  d.setDate(d.getDate() + days);
  return localDateString(d);
}

export function shiftMonth(dateStr: string, months: number): string {
  const d = new Date(`${dateStr}T00:00:00`);
  d.setMonth(d.getMonth() + months);
  return localDateString(d);
}

export function shiftYear(dateStr: string, years: number): string {
  const d = new Date(`${dateStr}T00:00:00`);
  d.setFullYear(d.getFullYear() + years);
  return localDateString(d);
}
