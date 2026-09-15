export type ScreenId =
  | "record"
  | "pending"
  | "ledger"
  | "reports"
  | "accounts"
  | "categories"
  | "settings";

export const SCREENS: ScreenId[] = [
  "record",
  "pending",
  "ledger",
  "reports",
  "accounts",
  "categories",
  "settings",
];

export type KindDto = {
  id: string;
  labelKey: string;
  hintKey: string | null;
  payloadSchemaVersion: number;
  balanceEffect: string;
  reportBucket: string;
  feeReportBucket: string;
  categoryRequired: boolean;
  counterAccountRequired: boolean;
  counterAmountRequired: boolean;
  counterAccountsMustDiffer: boolean;
  primaryAccountLabelKey: string | null;
  counterAccountLabelKey: string | null;
  implemented: boolean;
};

export type AccountDto = {
  id: string;
  name: string | null;
  accountKind: string;
  openingBalanceMinor: number;
  openingDebtMinor: number;
  openingAt: string;
  note: string | null;
  sortOrder: number;
  presetKey: string | null;
  balanceMinor: number;
  debtMinor: number;
};

export type CategoryDto = {
  id: string;
  parentId: string | null;
  name: string | null;
  presetKey: string | null;
  sortOrder: number;
  colorHex: string | null;
};

export type TagDto = {
  id: string;
  name: string;
};

export type SettingsDto = {
  currencyCode: string;
  defaultAccountId: string;
  schemaVersion: number;
  uiLanguage: string | null;
  colorScheme: string | null;
  uiTheme: string | null;
  defaultFeeCategoryId: string | null;
  reportMode: string | null;
  reportSide: string | null;
  reportCustomFrom: string | null;
  reportCustomTo: string | null;
  lastKindId: string | null;
  lastAccountId: string | null;
  lastCounterAccountId: string | null;
  lastCategoryId: string | null;
  lastFeeCategoryId: string | null;
  lastOccurredAt: string | null;
};

export type EntryDto = {
  id: string;
  kindId: string;
  amountMinor: number;
  occurredAt: string;
  accountId: string;
  counterAccountId: string | null;
  counterAmountMinor: number | null;
  categoryId: string;
  feeCategoryId: string | null;
  note: string | null;
  kindPayload: string;
  createdAt: string;
  updatedAt: string;
  tagIds: string[];
};

export type EntryWrite = {
  kindId: string;
  amountMinor: number;
  occurredAt: string;
  accountId: string;
  counterAccountId: string | null;
  counterAmountMinor: number | null;
  categoryId: string;
  feeCategoryId: string | null;
  note: string | null;
  tagNames: string[];
};

export type PendingEntryDto = {
  id: string;
  amountMinor: number;
  occurredAt: string;
  kindId: string | null;
  accountId: string | null;
  counterAccountId: string | null;
  counterAmountMinor: number | null;
  categoryId: string | null;
  feeCategoryId: string | null;
  note: string | null;
  createdAt: string;
  updatedAt: string;
  tagIds: string[];
};

export type PendingEntryWrite = {
  amountMinor: number;
  occurredAt: string;
  kindId: string | null;
  accountId: string | null;
  counterAccountId: string | null;
  counterAmountMinor: number | null;
  categoryId: string | null;
  feeCategoryId: string | null;
  note: string | null;
  tagNames: string[];
};

export type ImportPendingResult = {
  imported: number;
  errors: { line: number; code: string }[];
};

export type AccountWrite = {
  name: string;
  accountKind: string;
  openingBalanceMinor: number;
  openingDebtMinor: number;
  openingAt: string;
  note: string | null;
};

export type LedgerFilter = {
  fromDate: string | null;
  toDate: string | null;
  kindIds: string[];
  accountIds: string[];
  categoryId: string | null;
  tagId: string | null;
  noteContains: string | null;
};

export type ReportQuery = {
  mode: string;
  side: string;
  anchorDate: string;
  customFrom: string | null;
  customTo: string | null;
};

export type NamedAmount = {
  id: string;
  amountMinor: number;
  colorHex: string | null;
  percentBp: number;
};

export type ReportDto = {
  rangeFrom: string;
  rangeTo: string;
  isoWeek: number | null;
  sideTotal: number;
  incomeTotal: number;
  expenseTotal: number;
  net: number;
  average: number;
  previousTotal: number | null;
  delta: number | null;
  secondaryRepayment: number;
  secondaryPrepayment: number;
  secondaryLoan: number;
  secondaryTransferVolume: number;
  secondaryTransferFees: number;
  trend: { key: string; amountMinor: number }[];
  compositionMain: NamedAmount[];
  compositionSub: NamedAmount[];
  byAccount: NamedAmount[];
  comparison: { key: string; amountMinor: number; isCurrent: boolean }[] | null;
  ranking: {
    entryId: string;
    occurredAt: string;
    kindId: string;
    categoryId: string;
    note: string | null;
    amountMinor: number;
  }[];
};

export type BootstrapDto = {
  settings: SettingsDto;
  accounts: AccountDto[];
  categories: CategoryDto[];
  tags: TagDto[];
  kinds: KindDto[];
  dbPath: string;
  resolvedLanguage: string;
  systemLanguage: string | null;
  categoryPalette: string[];
  pendingCount: number;
};

export type AppError = {
  code: string;
  params?: { count?: number };
};

export type LedgerJump = {
  entryId: string;
  fromDate: string;
  toDate: string;
  kindIds: string[];
};
