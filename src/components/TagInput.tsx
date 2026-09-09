import { useState, type KeyboardEvent } from "react";
import { useTranslation } from "react-i18next";

export default function TagInput({
  value,
  onChange,
}: {
  value: string[];
  onChange: (tags: string[]) => void;
}) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState("");

  function add(raw: string) {
    const name = raw.trim();
    if (!name) return;
    if (value.some((v) => v.toLowerCase() === name.toLowerCase())) {
      setDraft("");
      return;
    }
    onChange([...value, name]);
    setDraft("");
  }

  function onKey(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      add(draft);
    } else if (e.key === "Backspace" && !draft && value.length) {
      onChange(value.slice(0, -1));
    }
  }

  return (
    <div className="tags">
      {value.map((tag) => (
        <span key={tag} className="chip">
          {tag}
          <button
            type="button"
            style={{ border: 0, background: "transparent", marginLeft: 4 }}
            onClick={() => onChange(value.filter((x) => x !== tag))}
          >
            ×
          </button>
        </span>
      ))}
      <input
        value={draft}
        placeholder={t("field.tags")}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={onKey}
        onBlur={() => add(draft)}
      />
    </div>
  );
}
