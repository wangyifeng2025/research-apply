"use client";

import { useState } from "react";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { ApiError } from "@/lib/api";
import { downloadApplicationPdf } from "@/lib/applications";

export function ExportPdfButton({
  applicationId,
  size = "default",
}: {
  applicationId: string;
  size?: "default" | "sm";
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function download() {
    setBusy(true);
    setError("");
    try {
      await downloadApplicationPdf(applicationId);
    } catch (reason: unknown) {
      setError(reason instanceof ApiError ? reason.message : "导出申报书失败");
    } finally {
      setBusy(false);
    }
  }

  return (
    <span className="inline-flex flex-wrap items-center gap-2">
      <Button type="button" variant="outline" size={size} disabled={busy} onClick={() => void download()}>
        <Icon name="mdi:file-pdf-box" className="text-sm" />
        {busy ? "正在生成" : "导出 PDF"}
      </Button>
      {error ? <span className="text-[11px] text-cinnabar">{error}</span> : null}
    </span>
  );
}
