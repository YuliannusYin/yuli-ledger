import type { TFunction } from "i18next";
import type { AccountDto, CategoryDto } from "./types";

export function categoryName(cat: CategoryDto | undefined, t: TFunction): string {
  if (!cat) return "";
  if (cat.name) return cat.name;
  if (cat.presetKey) return t(cat.presetKey);
  return cat.id;
}

export function accountName(acc: AccountDto | undefined, t: TFunction): string {
  if (!acc) return "";
  if (acc.name) return acc.name;
  if (acc.presetKey) return t(acc.presetKey);
  return acc.id;
}

export function mains(categories: CategoryDto[]): CategoryDto[] {
  return categories.filter((c) => !c.parentId).sort((a, b) => a.sortOrder - b.sortOrder);
}

export function subsOf(categories: CategoryDto[], parentId: string): CategoryDto[] {
  return categories
    .filter((c) => c.parentId === parentId)
    .sort((a, b) => a.sortOrder - b.sortOrder);
}

export function parentOf(categories: CategoryDto[], id: string): CategoryDto | undefined {
  const cat = categories.find((c) => c.id === id);
  if (!cat?.parentId) return cat;
  return categories.find((c) => c.id === cat.parentId);
}

export function liftHex(hex: string, dark: boolean): string {
  if (!dark) return hex;
  const n = hex.replace("#", "");
  if (n.length !== 6) return hex;
  const r = parseInt(n.slice(0, 2), 16);
  const g = parseInt(n.slice(2, 4), 16);
  const b = parseInt(n.slice(4, 6), 16);
  const lift = (c: number) => Math.min(255, Math.round(c + (255 - c) * 0.35));
  return `#${lift(r).toString(16).padStart(2, "0")}${lift(g).toString(16).padStart(2, "0")}${lift(b).toString(16).padStart(2, "0")}`;
}

export function subColor(parentHex: string, index: number, count: number, dark: boolean): string {
  const base = liftHex(parentHex, dark);
  const n = base.replace("#", "");
  const r = parseInt(n.slice(0, 2), 16);
  const g = parseInt(n.slice(2, 4), 16);
  const b = parseInt(n.slice(4, 6), 16);
  const t = count <= 1 ? 0 : index / Math.max(1, count - 1);
  const mix = dark ? 0.15 + t * 0.25 : 0.85 - t * 0.35;
  const ch = (c: number) => Math.round(c * (1 - mix) + (dark ? 255 : 255) * mix);
  const nr = Math.min(255, ch(r));
  const ng = Math.min(255, ch(g));
  const nb = Math.min(255, ch(b));
  return `#${nr.toString(16).padStart(2, "0")}${ng.toString(16).padStart(2, "0")}${nb.toString(16).padStart(2, "0")}`;
}
