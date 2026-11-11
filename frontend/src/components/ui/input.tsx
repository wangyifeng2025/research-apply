import type { InputHTMLAttributes } from "react";

import { cn } from "@/lib/utils";

function Input({ className, ...props }: InputHTMLAttributes<HTMLInputElement>) {
  return (
    <input
      className={cn(
        "flex h-8 w-full rounded-lg border border-ink-200 bg-white px-2.5 text-[12.5px] text-ink-800 outline-none transition placeholder:text-ink-400 focus:border-teal-400 focus:ring-2 focus:ring-teal-400/20 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-cinnabar aria-invalid:bg-[#fdf6f6]",
        className,
      )}
      {...props}
    />
  );
}

export { Input };
