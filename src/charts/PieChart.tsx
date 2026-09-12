import { Group } from "@visx/group";
import { Pie } from "@visx/shape";
import { ParentSize } from "@visx/responsive";
import { formatMinor } from "../lib/money";

export type Slice = { id: string; label: string; amountMinor: number; color: string };

export default function PieChart({
  slices,
  locale,
  centerLabel,
  otherLabel,
}: {
  slices: Slice[];
  locale: string;
  centerLabel: string;
  otherLabel: string;
}) {
  const pieData =
    slices.length > 10
      ? [
          ...slices.slice(0, 9),
          {
            id: "other",
            label: otherLabel,
            amountMinor: slices.slice(9).reduce((s, x) => s + x.amountMinor, 0),
            color: "#6b7280",
          },
        ]
      : slices;

  return (
    <div className="chart">
      <ParentSize>
        {({ width, height }) => {
          const size = Math.min(width, height);
          const r = size / 2 - 4;
          if (r < 10) return null;
          return (
            <svg width={width} height={height}>
              <Group top={height / 2} left={width / 2}>
                <Pie
                  data={pieData}
                  pieValue={(d) => d.amountMinor}
                  outerRadius={r}
                  innerRadius={r * 0.58}
                  padAngle={0.01}
                >
                  {({ arcs, path }) =>
                    arcs.map((arc) => (
                      <g key={arc.data.id}>
                        <path d={path(arc) ?? undefined} fill={arc.data.color} stroke="var(--surface)" strokeWidth={1} />
                        <title>
                          {arc.data.label} {formatMinor(arc.data.amountMinor, locale)}
                        </title>
                      </g>
                    ))
                  }
                </Pie>
                <text
                  textAnchor="middle"
                  dy="0.35em"
                  fill="var(--text)"
                  fontFamily="var(--font-mono)"
                  fontSize={12}
                >
                  {centerLabel}
                </text>
              </Group>
            </svg>
          );
        }}
      </ParentSize>
    </div>
  );
}
