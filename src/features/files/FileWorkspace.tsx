// STATUS: DIAMANT VGT SUPREME

import { ChangeEvent, DragEvent, useRef, useState } from "react";
import { useI18n } from "../../i18n";
import { analyzeBinary, sanitizeBinary } from "../../lib/backend";
import type { BinaryAnalysisReport, BinarySanitizeReport } from "../../types";

const MAX_BINARY_BYTES = 32 * 1024 * 1024;
const MAX_BATCH_BYTES = 128 * 1024 * 1024;
const MAX_BATCH_FILES = 50;
const BASE64_CHUNK_BYTES = 0x8000;

type JobStatus = "ready" | "working" | "analyzed" | "verified" | "error";

interface FileJob {
  id: string;
  name: string;
  sourceBytes: number;
  payload: string;
  status: JobStatus;
  analysis: BinaryAnalysisReport | null;
  result: BinarySanitizeReport | null;
  error: string | null;
}

function humanBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / (1024 * 1024)).toFixed(2)} MiB`;
}

function toBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  const chunks: string[] = [];
  for (let offset = 0; offset < bytes.length; offset += BASE64_CHUNK_BYTES) {
    chunks.push(String.fromCharCode(...bytes.subarray(offset, Math.min(offset + BASE64_CHUNK_BYTES, bytes.length))));
  }
  return btoa(chunks.join(""));
}

function decodeBase64(value: string): Uint8Array {
  const binary = atob(value);
  const output = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) output[index] = binary.charCodeAt(index);
  return output;
}

function safeBaseName(name: string): string {
  return name.replace(/\.[^.]+$/, "").replace(/[^a-zA-Z0-9._-]/g, "_").slice(0, 120) || "nullmark-file";
}

function jobReport(job: FileJob): BinaryAnalysisReport | null {
  return job.result?.after ?? job.analysis;
}

export function FileWorkspace() {
  const { t } = useI18n();
  const [jobs, setJobs] = useState<FileJob[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const revision = useRef(0);
  const verifiedCount = jobs.filter((job) => job.result?.verification_passed).length;
  const findingCount = jobs.reduce((total, job) => total + (jobReport(job)?.metadata_count ?? 0), 0);

  function updateJob(id: string, patch: Partial<FileJob>): void {
    setJobs((current) => current.map((job) => job.id === id ? { ...job, ...patch } : job));
  }

  async function load(files: readonly File[]): Promise<void> {
    const current = ++revision.current;
    setBusy(true);
    setError(null);
    setJobs([]);
    try {
      if (files.length === 0 || files.length > MAX_BATCH_FILES) throw new Error(`Select between 1 and ${MAX_BATCH_FILES} files.`);
      const totalBytes = files.reduce((total, file) => total + file.size, 0);
      if (totalBytes > MAX_BATCH_BYTES) throw new Error("Batch exceeds the 128 MiB safety limit.");
      const next: FileJob[] = [];
      for (const file of files) {
        if (file.size === 0 || file.size > MAX_BINARY_BYTES) throw new Error(`${file.name}: size must be between 1 byte and 32 MiB.`);
        next.push({
          id: crypto.randomUUID(),
          name: file.name,
          sourceBytes: file.size,
          payload: toBase64(await file.arrayBuffer()),
          status: "ready",
          analysis: null,
          result: null,
          error: null
        });
      }
      if (current === revision.current) setJobs(next);
    } catch (cause) {
      if (current === revision.current) setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      if (current === revision.current) setBusy(false);
    }
  }

  async function processJobs(clean: boolean): Promise<void> {
    if (jobs.length === 0) return;
    const current = revision.current;
    setBusy(true);
    setError(null);
    for (const job of jobs) {
      if (current !== revision.current) break;
      updateJob(job.id, { status: "working", result: clean ? job.result : null, error: null });
      try {
        if (clean) {
          const result = await sanitizeBinary(job.payload);
          if (current === revision.current) updateJob(job.id, {
            analysis: result.before,
            result,
            status: result.verification_passed ? "verified" : "error",
            error: result.verification_passed ? null : t("verificationFailed")
          });
        } else {
          const analysis = await analyzeBinary(job.payload);
          if (current === revision.current) updateJob(job.id, { analysis, status: "analyzed" });
        }
      } catch (cause) {
        if (current === revision.current) updateJob(job.id, { status: "error", error: cause instanceof Error ? cause.message : String(cause) });
      }
    }
    if (current === revision.current) setBusy(false);
  }

  function exportCleaned(job: FileJob): void {
    if (!job.result?.verification_passed) return;
    const blob = new Blob([decodeBase64(job.result.output_base64)], { type: job.result.after.mime });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${safeBaseName(job.name)}.clean.${job.result.after.format === "jpeg" ? "jpg" : job.result.after.format}`;
    anchor.rel = "noopener";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  function reset(): void {
    revision.current += 1;
    setJobs([]);
    setError(null);
    setBusy(false);
  }

  async function onChange(event: ChangeEvent<HTMLInputElement>): Promise<void> {
    const files = Array.from(event.target.files ?? []);
    event.target.value = "";
    await load(files);
  }

  async function onDrop(event: DragEvent<HTMLDivElement>): Promise<void> {
    event.preventDefault();
    await load(Array.from(event.dataTransfer.files));
  }

  return (
    <section className="panel file-workspace">
      <div className="panel-head">
        <div><span className="step">FILES</span><h2>{t("filesTitle")}</h2></div>
        {jobs.length > 0 && <button className="ghost" onClick={reset}>{t("clearBatch")}</button>}
      </div>
      {error && <div className="error-banner" role="alert">{error}</div>}
      <div className="file-drop" onDragOver={(event) => event.preventDefault()} onDrop={onDrop}>
        <div className="file-drop-icon" aria-hidden="true"><span>+</span></div>
        <strong>{jobs.length > 0 ? t("loadedFiles", { count: jobs.length }) : t("dropFiles")}</strong>
        <span>{t("fileLimits")}</span>
        <button className="secondary" disabled={busy} onClick={() => inputRef.current?.click()}>{t("selectFiles")}</button>
        <input
          ref={inputRef}
          className="visually-hidden"
          type="file"
          multiple
          accept=".png,.jpg,.jpeg,.webp,.pdf,.svg,.docx,.xlsx,.pptx,.odt,image/png,image/jpeg,image/webp,image/svg+xml,application/pdf,application/vnd.openxmlformats-officedocument.wordprocessingml.document,application/vnd.openxmlformats-officedocument.spreadsheetml.sheet,application/vnd.openxmlformats-officedocument.presentationml.presentation,application/vnd.oasis.opendocument.text"
          onChange={onChange}
        />
      </div>
      {jobs.length > 0 && (
        <>
          <div className="batch-toolbar">
            <div className="batch-summary">
              <span>{t("sourceData", { size: humanBytes(jobs.reduce((total, job) => total + job.sourceBytes, 0)) })}</span>
              <span>{t("currentFindings", { count: findingCount })}</span>
              <span>{t("verifiedCount", { verified: verifiedCount, total: jobs.length })}</span>
            </div>
            <div className="file-actions">
              <button className="secondary" disabled={busy} onClick={() => void processJobs(false)}>{t("analyzeBatch")}</button>
              <button className="primary" disabled={busy} onClick={() => void processJobs(true)}>{busy ? t("processingBatch") : t("cleanAll")}</button>
            </div>
          </div>
          <div className="job-list">
            {jobs.map((job) => {
              const report = jobReport(job);
              return (
                <article className="job-card" key={job.id}>
                  <div className="job-head">
                    <div><strong>{job.name}</strong><span>{humanBytes(job.sourceBytes)} · {job.status}</span></div>
                    {job.result?.verification_passed && <button className="secondary" onClick={() => exportCleaned(job)}>{t("export")}</button>}
                  </div>
                  {job.error && <div className="job-error" role="alert">{job.error}</div>}
                  {report && (
                    <>
                      <div className="job-metrics">
                        <span className="format-label">{report.format.toUpperCase()}</span>
                        <span>{t("metadata", { count: report.metadata_count })}</span>
                        <span>{report.c2pa_detected ? t("c2paPresent") : t("c2paNone")}</span>
                        <code title={report.sha256}>{report.sha256.slice(0, 12)}…</code>
                      </div>
                      {report.c2pa_detected && <div className="provenance-warning">{t("provenanceWarning")}</div>}
                      <div className="job-findings">
                        {report.findings.length === 0 ? <span>{t("noFileFindings")}</span> : report.findings.map((finding) => <span key={finding.kind}>{finding.description} ×{finding.count}</span>)}
                      </div>
                    </>
                  )}
                  {job.result && <div className={`job-verify ${job.result.verification_passed ? "ok" : "fail"}`}>{job.result.verification_passed ? `✓ ${t("reopenVerified", { count: job.result.removed_items })}` : t("verificationFailed")}</div>}
                </article>
              );
            })}
          </div>
        </>
      )}
    </section>
  );
}
