import type { TextareaHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

function Textarea({ className, ...props }: TextareaHTMLAttributes<HTMLTextAreaElement>) {
  return (
    <textarea
      className={cn(
        "flex min-h-24 w-full rounded-lg border border-ink-200 bg-white px-2.5 py-2 text-[12.5px] leading-5 text-ink-800 outline-none transition placeholder:text-ink-400 focus:border-teal-400 focus:ring-2 focus:ring-teal-400/20 disabled:cursor-not-allowed disabled:bg-ink-50 disabled:opacity-60 aria-invalid:border-cinnabar aria-invalid:bg-[#fdf6f6]",
        className,
      )}
      {...props}
    />
  );
}

export { Textarea };
