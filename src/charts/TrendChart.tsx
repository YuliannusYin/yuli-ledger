import { scaleLinear, scalePoint } from "@visx/scale";
import { LinePath } from "@visx/shape";
import { ParentSize } from "@visx/responsive";
import { formatMinor } from "../lib/money";

export default function TrendChart({
  points,
  color,
  locale,
}: {
  points: { key: string; amountMinor: number }[];
  color: string;
  locale: string;
}) {
  return (
    <div className="chart">
      <ParentSize>
        {({ width, height }) => {
          if (width < 10 || height < 10 || points.length === 0) return null;
          const x = scalePoint({
            domain: points.map((p) => p.key),
            range: [24, width - 12],
          });
          const max = Math.max(...points.map((p) => p.amountMinor), 1);
          const y = scaleLinear({ domain: [0, max], range: [height - 20, 12] });
          return (
            <svg width={width} height={height}>
              <LinePath
                data={points}
                x={(d) => x(d.key) ?? 0}
                y={(d) => y(d.amountMinor)}
                stroke={color}
                strokeWidth={1.75}
                fill="transparent"
              />
              {points.map((p) => (
                <circle
                  key={p.key}
                  cx={x(p.key) ?? 0}
                  cy={y(p.amountMinor)}
                  r={3}
                  fill={color}
                >
                  <title>
                    {p.key} {formatMinor(p.amountMinor, locale)}
                  </title>
                </circle>
              ))}
            </svg>
          );
        }}
      </ParentSize>
    </div>
  );
}
