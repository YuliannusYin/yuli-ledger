import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import type { CategoryDto } from "../lib/types";
import {
  categoryUsage,
  createMainCategory,
  createSubCategory,
  deleteCategory,
  renameCategory,
  reorderCategories,
  updateCategoryColor,
} from "../lib/api";
import { categoryName, mains, subsOf } from "../lib/names";
import ConfirmDialog from "../components/ConfirmDialog";
import ColorGrid from "../components/ColorGrid";
import DragList from "../components/DragList";

export default function CategoriesScreen({
  categories,
  palette,
  locale,
  onChanged,
}: {
  categories: CategoryDto[];
  palette: string[];
  locale: string;
  onChanged: () => Promise<void>;
}) {
  const { t } = useTranslation();
  const mainList = mains(categories);
  const [mainId, setMainId] = useState<string | null>(mainList[0]?.id ?? null);
  const main = categories.find((c) => c.id === mainId);
  const children = mainId ? subsOf(categories, mainId) : [];
  const [subId, setSubId] = useState<string | null>(null);
  const sub = categories.find((c) => c.id === subId);
  const [mainName, setMainName] = useState("");
  const [subName, setSubName] = useState("");
  const [mainUsage, setMainUsage] = useState(0);
  const [subUsage, setSubUsage] = useState(0);
  const [confirm, setConfirm] = useState<"main" | "sub" | null>(null);
  const [picker, setPicker] = useState<{ id: string; left: number; top: number } | null>(null);
  const pickerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (main) setMainName(categoryName(main, t));
    if (mainId) {
      void Promise.all(subsOf(categories, mainId).map((s) => categoryUsage(s.id))).then((ns) =>
        setMainUsage(ns.reduce((a, b) => a + b, 0)),
      );
    }
  }, [main, mainId, categories, t]);

  useEffect(() => {
    if (sub) {
      setSubName(categoryName(sub, t));
      void categoryUsage(sub.id).then(setSubUsage);
    }
  }, [sub, t]);

  useEffect(() => {
    if (!picker) return;
    function onDown(e: MouseEvent) {
      const target = e.target as HTMLElement | null;
      if (pickerRef.current?.contains(target)) return;
      if (target?.closest("button.swatch")) return;
      setPicker(null);
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setPicker(null);
    }
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
    };
  }, [picker]);

  const pickerMain = picker ? categories.find((c) => c.id === picker.id) : undefined;

  function openColorPicker(id: string, el: HTMLElement) {
    const r = el.getBoundingClientRect();
    const width = 196;
    const height = 300;
    const left = Math.min(Math.max(8, r.left), window.innerWidth - width - 8);
    const top =
      r.bottom + 4 + height > window.innerHeight ? Math.max(8, r.top - height - 4) : r.bottom + 4;
    setMainId(id);
    setSubId(null);
    setPicker({ id, left, top });
  }

  return (
    <div>
      <div className="toolbar">
        <h1 style={{ margin: 0 }}>{t("nav.categories")}</h1>
        <button
          type="button"
          className="btn"
          onClick={() =>
            void createMainCategory(t("categories.newMain"), t("category.other")).then(onChanged)
          }
        >
          {t("action.addMain")}
        </button>
      </div>
      <div className="split">
        <DragList
          items={mainList}
          selectedId={mainId}
          onSelect={(id) => {
            setMainId(id);
            setSubId(null);
            setPicker(null);
          }}
          onReorder={(ids) => void reorderCategories(null, ids).then(onChanged)}
          render={(m) => (
            <span className="list-item-main">
              <button
                type="button"
                className="swatch"
                style={{ background: m.colorHex ?? "#52525b" }}
                aria-label={t("categories.pickColor")}
                title={t("categories.pickColor")}
                onClick={(e) => {
                  e.stopPropagation();
                  openColorPicker(m.id, e.currentTarget);
                }}
              />
              <span className="list-item-name">{categoryName(m, t)}</span>
            </span>
          )}
        />
        <div className="surface" style={{ padding: 16 }}>
          {main && (
            <>
              <div className="field">
                <label>{t("field.name")}</label>
                <input value={mainName} onChange={(e) => setMainName(e.target.value)} />
              </div>
              <div className="row">
                <button
                  type="button"
                  className="btn primary"
                  onClick={() => void renameCategory(main.id, mainName).then(onChanged)}
                >
                  {t("action.save")}
                </button>
                <button
                  type="button"
                  className="btn"
                  onClick={() =>
                    void createSubCategory(main.id, t("categories.newSub")).then(onChanged)
                  }
                >
                  {t("action.addSub")}
                </button>
                <button
                  type="button"
                  className="btn danger"
                  disabled={mainUsage > 0}
                  onClick={() => setConfirm("main")}
                >
                  {t("action.delete")}
                </button>
              </div>
              {mainUsage > 0 && (
                <p className="reason">{t("error.categoryInUse", { count: mainUsage })}</p>
              )}
              <h2 style={{ marginTop: 16 }}>{t("field.subCategory")}</h2>
              <DragList
                items={children}
                selectedId={subId}
                onSelect={setSubId}
                onReorder={(ids) => void reorderCategories(main.id, ids).then(onChanged)}
                render={(s) => <span className="list-item-name">{categoryName(s, t)}</span>}
              />
            </>
          )}
          {sub && (
            <div style={{ marginTop: 16 }}>
              <div className="field">
                <label>{t("field.name")}</label>
                <input value={subName} onChange={(e) => setSubName(e.target.value)} />
              </div>
              <div className="row">
                <button
                  type="button"
                  className="btn primary"
                  onClick={() => void renameCategory(sub.id, subName).then(onChanged)}
                >
                  {t("action.save")}
                </button>
                <button
                  type="button"
                  className="btn danger"
                  disabled={subUsage > 0}
                  onClick={() => setConfirm("sub")}
                >
                  {t("action.delete")}
                </button>
              </div>
              {subUsage > 0 && (
                <p className="reason">{t("error.categoryInUse", { count: subUsage })}</p>
              )}
            </div>
          )}
        </div>
      </div>
      {picker && pickerMain && (
        <div
          ref={pickerRef}
          className="color-popover surface"
          style={{ left: picker.left, top: picker.top }}
        >
          <ColorGrid
            colors={palette}
            value={pickerMain.colorHex}
            onSelect={(hex) => {
              void updateCategoryColor(picker.id, hex).then(onChanged);
              setPicker(null);
            }}
          />
        </div>
      )}
      {confirm && (
        <ConfirmDialog
          message={t("confirm.deleteCategory")}
          onCancel={() => setConfirm(null)}
          onConfirm={() => {
            const id = confirm === "main" ? mainId : subId;
            if (!id) return;
            void deleteCategory(id).then(async () => {
              setConfirm(null);
              if (confirm === "main") setMainId(mainList.find((m) => m.id !== id)?.id ?? null);
              setSubId(null);
              await onChanged();
            });
          }}
        />
      )}
      <span hidden>{locale}</span>
    </div>
  );
}
