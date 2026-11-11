import type { HTMLAttributes } from "react";

import { cn } from "@/lib/utils";

function Badge({ className, tone = "neutral", ...props }: HTMLAttributes<HTMLSpanElement> & { tone?: "neutral" | "teal" | "gold" }) {
  return (
    <span
      className={cn(
        "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-[11px]",
        tone === "teal" && "bg-teal-50 text-teal-700",
        tone === "gold" && "bg-gold-50 text-gold-700",
        tone === "neutral" && "bg-ink-100 text-ink-600",
        className,
      )}
      {...props}
    />
  );
}

export { Badge };
