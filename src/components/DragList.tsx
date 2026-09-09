import { useState, type ReactNode } from "react";

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
  const [dragId, setDragId] = useState<string | null>(null);

  return (
    <div className="list surface">
      {items.map((item) => (
        <div
          key={item.id}
          draggable
          className={`list-item ${selectedId === item.id ? "active" : ""}`}
          onClick={() => onSelect(item.id)}
          onDragStart={() => setDragId(item.id)}
          onDragOver={(e) => e.preventDefault()}
          onDrop={() => {
            if (!dragId || dragId === item.id) return;
            const ids = items.map((i) => i.id);
            const from = ids.indexOf(dragId);
            const to = ids.indexOf(item.id);
            ids.splice(from, 1);
            ids.splice(to, 0, dragId);
            onReorder(ids);
            setDragId(null);
          }}
        >
          {render(item)}
        </div>
      ))}
    </div>
  );
}
