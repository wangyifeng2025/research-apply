"use client";

import * as AlertDialogPrimitive from "@radix-ui/react-alert-dialog";
import type { ComponentProps, ReactNode } from "react";

import { buttonVariants } from "@/components/ui/button";
import { cn } from "@/lib/utils";

function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  busy = false,
  onConfirm,
  onClose,
}: {
  open: boolean;
  title: string;
  description: ReactNode;
  confirmLabel: string;
  busy?: boolean;
  onConfirm: () => void;
  onClose: () => void;
}) {
  return (
    <AlertDialogPrimitive.Root open={open} onOpenChange={(next) => { if (!next && !busy) onClose(); }}>
      <AlertDialogPrimitive.Portal>
        <AlertDialogPrimitive.Overlay className="fixed inset-0 z-50 bg-ink-950/50" />
        <AlertDialogPrimitive.Content className="fixed top-1/2 left-1/2 z-50 w-[calc(100%-2rem)] max-w-[420px] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white p-5 shadow-2xl focus:outline-none">
          <AlertDialogPrimitive.Title className="text-[15px] font-semibold text-ink-900">{title}</AlertDialogPrimitive.Title>
          <AlertDialogPrimitive.Description className="mt-2 text-[12.5px] leading-5 text-ink-600">{description}</AlertDialogPrimitive.Description>
          <div className="mt-5 flex justify-end gap-2">
            <AlertDialogPrimitive.Cancel className={buttonVariants({ variant: "outline" })} disabled={busy}>取消</AlertDialogPrimitive.Cancel>
            <AlertDialogPrimitive.Action className={cn(buttonVariants({ variant: "default" }), "bg-cinnabar hover:bg-cinnabar/90")} disabled={busy} onClick={(event) => { event.preventDefault(); onConfirm(); }}>
              {busy ? "正在处理…" : confirmLabel}
            </AlertDialogPrimitive.Action>
          </div>
        </AlertDialogPrimitive.Content>
      </AlertDialogPrimitive.Portal>
    </AlertDialogPrimitive.Root>
  );
}

export { ConfirmDialog };
export type AlertDialogContentProps = ComponentProps<typeof AlertDialogPrimitive.Content>;
