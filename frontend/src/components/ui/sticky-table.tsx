import type { CSSProperties, ReactNode } from "react";

import { cn } from "@/lib/utils";

export type StickyColumn<Row> = {
  header: ReactNode;
  width: number;
  align?: "left" | "right";
  cell: (row: Row) => ReactNode;
};

function StickyTable<Row>({
  columns,
  rows,
  rowKey,
  empty,
  muted,
}: {
  columns: StickyColumn<Row>[];
  rows: Row[];
  rowKey: (row: Row) => string;
  empty?: string;
  muted?: (row: Row) => boolean;
}) {
  const minWidth = columns.reduce((total, column) => total + column.width, 0);
  return (
    <div className="overflow-x-auto">
      <table className="w-full border-separate border-spacing-0 text-left text-[12px]" style={{ minWidth }}>
        <thead>
          <tr>
            {columns.map((column, index) => (
              <th
                key={index}
                className={cn("bg-[#f4f7f9] px-3 py-2.5 text-[11px] font-medium text-ink-500", pinClass(index, columns.length, true), column.align === "right" && "text-right")}
                style={pinStyle(index, columns)}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const quiet = muted?.(row) ?? false;
            return (
              <tr key={rowKey(row)} data-muted={quiet ? "true" : undefined} className="group">
                {columns.map((column, index) => (
                  <td
                    key={index}
                    className={cn(
                      "border-t border-ink-100 bg-white px-3 py-2.5 align-middle text-ink-700 group-hover:bg-[#f4f7f9] group-data-[muted=true]:bg-ink-50 group-data-[muted=true]:text-ink-400",
                      pinClass(index, columns.length, false),
                      column.align === "right" && "text-right",
                    )}
                    style={pinStyle(index, columns)}
                  >
                    {column.cell(row)}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
      {rows.length === 0 && empty ? <p className="px-4 py-10 text-center text-[12px] text-ink-400">{empty}</p> : null}
    </div>
  );
}

function pinClass(index: number, total: number, head: boolean) {
  const last = index === total - 1;
  if (index !== 0 && index !== 1 && !last) return "";
  return cn("sticky", head ? "z-30" : "z-20");
}

function pinStyle(index: number, columns: { width: number }[]): CSSProperties {
  const width = columns[index]?.width ?? 120;
  const base: CSSProperties = { width, minWidth: width, maxWidth: width };
  if (index === 0) return { ...base, left: 0 };
  if (index === 1) return { ...base, left: columns[0]?.width ?? 0 };
  if (index === columns.length - 1) return { ...base, right: 0 };
  return { minWidth: width };
}

export { StickyTable };
