import { useEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

export default function DragList<T extends { id: string }>({
  items,
  selectedId,
  onSelect,
  onReorder,
  render,
}: {
  items: T[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onReorder: (ids: string[]) => void;
  render: (item: T) => ReactNode;
}) {
  const { t } = useTranslation();
  const [menu, setMenu] = useState<{ id: string; left: number; top: number } | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!menu) return;
    function onDown(e: MouseEvent) {
      if (menuRef.current?.contains(e.target as Node)) return;
      setMenu(null);
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") setMenu(null);
    }
    function onScroll() {
      setMenu(null);
    }
    window.addEventListener("mousedown", onDown);
    window.addEventListener("keydown", onKey);
    window.addEventListener("scroll", onScroll, true);
    return () => {
      window.removeEventListener("mousedown", onDown);
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("scroll", onScroll, true);
    };
  }, [menu]);

  function move(id: string, dir: -1 | 1) {
    const ids = items.map((i) => i.id);
    const from = ids.indexOf(id);
    const to = from + dir;
    if (from < 0 || to < 0 || to >= ids.length) return;
    const next = [...ids];
    const [moved] = next.splice(from, 1);
    next.splice(to, 0, moved);
    onReorder(next);
    setMenu(null);
  }

  const menuIndex = menu ? items.findIndex((i) => i.id === menu.id) : -1;

  return (
    <div className="list surface">
      {items.map((item) => (
        <div
          key={item.id}
          className={`list-item ${selectedId === item.id ? "active" : ""}`}
          onClick={() => onSelect(item.id)}
          onContextMenu={(e) => {
            e.preventDefault();
            onSelect(item.id);
            const width = 128;
            const height = 64;
            setMenu({
              id: item.id,
              left: Math.min(Math.max(8, e.clientX), window.innerWidth - width - 8),
              top: Math.min(Math.max(8, e.clientY), window.innerHeight - height - 8),
            });
          }}
        >
          <div className="list-item-body">{render(item)}</div>
        </div>
      ))}
      {menu && (
        <div
          ref={menuRef}
          className="ctx-menu surface"
          style={{ left: menu.left, top: menu.top }}
          role="menu"
        >
          <button
            type="button"
            role="menuitem"
            disabled={menuIndex <= 0}
            onClick={() => move(menu.id, -1)}
          >
            {t("action.moveUp")}
          </button>
          <button
            type="button"
            role="menuitem"
            disabled={menuIndex < 0 || menuIndex >= items.length - 1}
            onClick={() => move(menu.id, 1)}
          >
            {t("action.moveDown")}
          </button>
        </div>
      )}
    </div>
  );
}
