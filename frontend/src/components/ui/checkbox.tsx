"use client";

import * as CheckboxPrimitive from "@radix-ui/react-checkbox";
import type { ComponentProps } from "react";

import { Icon } from "@/components/icon";
import { cn } from "@/lib/utils";

function Checkbox({ className, ...props }: ComponentProps<typeof CheckboxPrimitive.Root>) {
  return (
    <CheckboxPrimitive.Root
      className={cn(
        "flex h-4 w-4 shrink-0 items-center justify-center rounded border border-ink-300 bg-white text-white outline-none focus-visible:ring-2 focus-visible:ring-teal-400/40 data-[state=checked]:border-teal-700 data-[state=checked]:bg-teal-700",
        className,
      )}
      {...props}
    >
      <CheckboxPrimitive.Indicator>
        <Icon name="mdi:check" className="text-[13px]" />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  );
}

export { Checkbox };
