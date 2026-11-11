"use client";

import { useRef, useState } from "react";

import { Icon } from "@/components/icon";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ApiError } from "@/lib/api";
import {
  attachmentKindLabels,
  deleteAttachment,
  downloadAttachment,
  uploadAttachment,
  type AttachmentItem,
  type AttachmentKind,
} from "@/lib/applications";

const LIMIT = 20 * 1024 * 1024;

export function AttachmentPanel({
  applicationId,
  items,
  editable,
  onChanged,
}: {
  applicationId: string;
  items: AttachmentItem[];
  editable: boolean;
  onChanged?: (items: AttachmentItem[]) => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [kind, setKind] = useState<AttachmentKind>("supporting");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function onFile(file: File | undefined) {
    if (!file || !editable) {
      return;
    }
    setError("");
    if (file.size > LIMIT) {
      setError("单个附件不能超过 20 MB");
      if (inputRef.current) {
        inputRef.current.value = "";
      }
      return;
    }
    setBusy(true);
    try {
      const saved = await uploadAttachment(applicationId, kind, file);
      onChanged?.([...items, saved]);
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "上传失败");
    } finally {
      setBusy(false);
      if (inputRef.current) {
        inputRef.current.value = "";
      }
    }
  }

  async function remove(item: AttachmentItem) {
    setError("");
    setBusy(true);
    try {
      await deleteAttachment(applicationId, item.id);
      onChanged?.(items.filter((row) => row.id !== item.id));
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "删除失败");
    } finally {
      setBusy(false);
    }
  }

  async function download(item: AttachmentItem) {
    setError("");
    try {
      await downloadAttachment(applicationId, item);
    } catch (caught) {
      setError(caught instanceof ApiError ? caught.message : "下载失败");
    }
  }

  return (
    <section className="rounded-xl border border-ink-200/80 bg-white p-4 shadow-sm sm:p-5">
      <div className="flex flex-wrap items-center gap-2">
        <span className="flex h-6 w-6 items-center justify-center rounded bg-ink-900 text-white">
          <Icon name="mdi:paperclip" className="text-sm" />
        </span>
        <h2 className="text-[14px] font-semibold text-ink-900">附件</h2>
        <span className="ml-auto text-[10.5px] text-ink-400">PDF、JPG、PNG，单个不超过 20 MB</span>
      </div>
      <p className="sheet-hint mt-1.5">推荐人签字扫描件只给申报人、本部门审核和科研处查看。专家盲审看不到签字件。</p>
      {editable ? (
        <div className="mt-3 flex flex-wrap items-end gap-2">
          <label className="block min-w-44">
            <span className="field-label">附件类型</span>
            <Select value={kind} onValueChange={(value) => setKind(value as AttachmentKind)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="signature">推荐人签字扫描件</SelectItem>
                <SelectItem value="supporting">成果证明材料</SelectItem>
                <SelectItem value="other">其他</SelectItem>
              </SelectContent>
            </Select>
          </label>
          <Button disabled={busy} onClick={() => inputRef.current?.click()}>
            <Icon name="mdi:upload-outline" className="text-sm" />
            {busy ? "请稍候" : "选择文件"}
          </Button>
          <input
            ref={inputRef}
            type="file"
            accept=".pdf,.jpg,.jpeg,.png,application/pdf,image/jpeg,image/png"
            className="hidden"
            onChange={(event) => void onFile(event.target.files?.[0])}
          />
        </div>
      ) : null}
      {error ? <p className="mt-2 text-[12px] text-cinnabar">{error}</p> : null}
      {items.length === 0 ? (
        <p className="mt-3 text-[12.5px] text-ink-400">还没有附件</p>
      ) : (
        <ul className="mt-3 divide-y divide-ink-100 rounded-lg border border-ink-100">
          {items.map((item) => (
            <li key={item.id} className="flex flex-wrap items-center gap-2 px-3 py-2">
              <span className="min-w-0 flex-1">
                <span className="block truncate text-[12.5px] text-ink-800">{item.file_name}</span>
                <span className="text-[11px] text-ink-400">
                  {attachmentKindLabels[item.kind] ?? item.kind} · {formatSize(item.byte_size)}
                </span>
              </span>
              <Button variant="outline" size="sm" disabled={busy} onClick={() => void download(item)}>
                下载
              </Button>
              {editable ? (
                <Button variant="destructive" size="sm" disabled={busy} onClick={() => void remove(item)}>
                  删除
                </Button>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

function formatSize(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
