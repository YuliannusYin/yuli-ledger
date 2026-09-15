import { invoke } from "@tauri-apps/api/core";
import type {
  AccountDto,
  AccountWrite,
  AppError,
  BootstrapDto,
  CategoryDto,
  EntryDto,
  EntryWrite,
  LedgerFilter,
  ReportDto,
  ReportQuery,
  SettingsDto,
  TagDto,
  PendingEntryDto,
  PendingEntryWrite,
  ImportPendingResult,
} from "./types";

export async function api<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    if (err && typeof err === "object" && "code" in err) {
      throw err as AppError;
    }
    if (typeof err === "string") {
      try {
        const parsed = JSON.parse(err) as AppError;
        if (parsed.code) throw parsed;
      } catch {
        /* fall through */
      }
    }
    throw { code: "error.db" } satisfies AppError;
  }
}

export const getBootstrap = () => api<BootstrapDto>("get_bootstrap");
export const updateSettings = (settings: SettingsDto) =>
  api<SettingsDto>("update_settings", { settings });
export const createAccount = (write: AccountWrite) =>
  api<AccountDto>("create_account", { write });
export const updateAccount = (id: string, write: AccountWrite) =>
  api<AccountDto>("update_account", { id, write });
export const deleteAccount = (id: string) => api<void>("delete_account", { id });
export const reorderAccounts = (orderedIds: string[]) =>
  api<void>("reorder_accounts", { orderedIds });
export const accountUsage = (id: string) => api<number>("account_usage", { id });
export const createMainCategory = (name: string, otherLabel: string) =>
  api<CategoryDto[]>("create_main_category", { name, otherLabel });
export const createSubCategory = (parentId: string, name: string) =>
  api<CategoryDto[]>("create_sub_category", { parentId, name });
export const renameCategory = (id: string, name: string) =>
  api<CategoryDto[]>("rename_category", { id, name });
export const updateCategoryColor = (id: string, colorHex: string) =>
  api<CategoryDto[]>("update_category_color", { id, colorHex });
export const deleteCategory = (id: string) => api<void>("delete_category", { id });
export const reorderCategories = (parentId: string | null, orderedIds: string[]) =>
  api<void>("reorder_categories", { parentId, orderedIds });
export const categoryUsage = (id: string) => api<number>("category_usage", { id });
export const deleteTag = (id: string) => api<void>("delete_tag", { id });
export const createEntry = (write: EntryWrite) => api<EntryDto>("create_entry", { write });
export const updateEntry = (id: string, write: EntryWrite) =>
  api<EntryDto>("update_entry", { id, write });
export const deleteEntry = (id: string) => api<void>("delete_entry", { id });
export const listEntries = (filter: LedgerFilter) =>
  api<EntryDto[]>("list_entries", { filter });
export const getReport = (query: ReportQuery) => api<ReportDto>("get_report", { query });
export const listTags = () => api<TagDto[]>("list_tags");
export const exportEntriesCsv = (args: {
  path: string;
  fromDate: string | null;
  toDate: string | null;
  labels: Record<string, string>;
}) => api<void>("export_entries_csv", args);
export const exportEntriesTxt = (args: {
  path: string;
  fromDate: string | null;
  toDate: string | null;
  labels: Record<string, string>;
}) => api<void>("export_entries_txt", args);
export const exportBackupJson = (path: string) => api<void>("export_backup_json", { path });
export const listPendingEntries = () => api<PendingEntryDto[]>("list_pending_entries");
export const updatePendingEntry = (id: string, write: PendingEntryWrite) =>
  api<PendingEntryDto>("update_pending_entry", { id, write });
export const deletePendingEntry = (id: string) => api<void>("delete_pending_entry", { id });
export const postPendingEntry = (id: string) => api<EntryDto>("post_pending_entry", { id });
export const importPendingCsv = (path: string) =>
  api<ImportPendingResult>("import_pending_csv", { path });
export const writePendingCsvTemplate = (path: string) =>
  api<void>("write_pending_csv_template", { path });
