import { scaleBand, scaleLinear } from "@visx/scale";
import { ParentSize } from "@visx/responsive";
import { formatMinor } from "../lib/money";

export default function BarChart({
  bars,
  color,
  locale,
}: {
  bars: { key: string; amountMinor: number; isCurrent: boolean }[];
  color: string;
  locale: string;
}) {
  return (
    <div className="chart">
      <ParentSize>
        {({ width, height }) => {
          if (width < 10 || height < 10) return null;
          const x = scaleBand({
            domain: bars.map((b) => b.key),
            range: [8, width - 8],
            padding: 0.25,
          });
          const max = Math.max(...bars.map((b) => b.amountMinor), 1);
          const y = scaleLinear({ domain: [0, max], range: [height - 18, 8] });
          return (
            <svg width={width} height={height}>
              {bars.map((b) => {
                const bw = x.bandwidth();
                const bh = Math.max(1, y(0) - y(b.amountMinor));
                return (
                  <g key={b.key}>
                    <rect
                      x={x(b.key)}
                      y={y(b.amountMinor)}
                      width={bw}
                      height={bh}
                      fill={color}
                      opacity={b.isCurrent ? 1 : 0.7}
                      stroke={b.isCurrent ? "var(--text)" : "none"}
                      strokeWidth={b.isCurrent ? 1 : 0}
                    />
                    <title>
                      {b.key} {formatMinor(b.amountMinor, locale)}
                    </title>
                  </g>
                );
              })}
            </svg>
          );
        }}
      </ParentSize>
    </div>
  );
}
