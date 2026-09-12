export default function ColorGrid({
  colors,
  value,
  onSelect,
}: {
  colors: string[];
  value: string | null;
  onSelect: (hex: string) => void;
}) {
  return (
    <div className="color-grid">
      {colors.map((hex) => (
        <button
          key={hex}
          type="button"
          className={value && value.toLowerCase() === hex.toLowerCase() ? "active" : ""}
          style={{ background: hex }}
          title={hex}
          aria-label={hex}
          onClick={() => onSelect(hex)}
        />
      ))}
    </div>
  );
}
