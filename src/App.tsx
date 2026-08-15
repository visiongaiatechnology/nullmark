// STATUS: DIAMANT VGT SUPREME

import { ChangeEvent, DragEvent, useMemo, useRef, useState } from "react";
import logoUrl from "./assets/nullmark-symbol.svg";
import { FileWorkspace } from "./features/files/FileWorkspace";
import { DiffViewer } from "./features/text/DiffViewer";
import { useI18n } from "./i18n";
import { analyzeText, sanitizeText } from "./lib/backend";
import type { AnalysisReport, FindingGroup, SanitizeMode, SanitizeReport } from "./types";

const MAX_FILE_BYTES = 8 * 1024 * 1024;
const ACCEPTED_EXTENSIONS = ["txt", "md", "markdown", "html", "htm", "csv", "json", "xml", "yaml", "yml"];

function humanBytes(value: number): string {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KiB`;
  return `${(value / (1024 * 1024)).toFixed(2)} MiB`;
}

function extensionOf(name: string): string {
  return name.toLowerCase().split(".").pop() ?? "";
}

function requiredMode(report: AnalysisReport): SanitizeMode | null {
  if (report.findings.some((finding) => finding.action === "remove_maximum" || finding.action === "normalize_maximum")) return "maximum";
  if (report.findings.some((finding) => finding.action === "remove_strict" || finding.action === "normalize_strict")) return "strict";
  if (report.findings.some((finding) => finding.action === "remove_safe")) return "safe";
  return null;
}

export default function App() {
  const { locale, setLocale, t } = useI18n();
  const [input, setInput] = useState("");
  const [analysis, setAnalysis] = useState<AnalysisReport | null>(null);
  const [result, setResult] = useState<SanitizeReport | null>(null);
  const [mode, setMode] = useState<SanitizeMode>("safe");
  const [workspaceKind, setWorkspaceKind] = useState<"text" | "files">("text");
  const [resultView, setResultView] = useState<"output" | "diff">("output");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);
  const inputRevision = useRef(0);

  const displayedReport = result?.after ?? analysis;
  const fullyClean = result?.verification_passed === true && result.after.finding_count === 0;
  const remainingRequiredMode = result ? requiredMode(result.after) : null;
  const status = useMemo(() => {
    if (busy) return t("processing");
    if (fullyClean) return t("clean");
    if (result?.verification_passed) return t("policyVerified", { mode: result.mode });
    if (analysis) return analysis.finding_count > 0 ? t("findingsDetected") : t("noMarkers");
    return t("ready");
  }, [analysis, busy, fullyClean, result, t]);

  function actionLabel(action: FindingGroup["action"]): string {
    switch (action) {
      case "remove_safe": return `${t("safe")} · ${t("removedKind")}`;
      case "remove_strict": return `${t("strict")} · ${t("removedKind")}`;
      case "remove_maximum": return `${t("maximum")} · ${t("removedKind")}`;
      case "normalize_strict": return `${t("strict")} · ${t("normalizedKind")}`;
      case "normalize_maximum": return `${t("maximum")} · ${t("normalizedKind")}`;
      case "report_only": return t("review");
    }
  }

  async function runAnalysis(): Promise<void> {
    if (!input) return;
    const revision = inputRevision.current;
    const snapshot = input;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      const report = await analyzeText(snapshot);
      if (revision === inputRevision.current) setAnalysis(report);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function runSanitize(selectedMode: SanitizeMode = mode): Promise<void> {
    if (!input) return;
    const revision = inputRevision.current;
    const snapshot = input;
    setBusy(true);
    setError(null);
    try {
      const sanitized = await sanitizeText(snapshot, selectedMode);
      if (revision === inputRevision.current) {
        setResult(sanitized);
        setAnalysis(sanitized.before);
        setResultView(sanitized.change_count > 0 ? "diff" : "output");
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  function reset(): void {
    inputRevision.current += 1;
    setInput("");
    setAnalysis(null);
    setResult(null);
    setFileName(null);
    setError(null);
    setCopied(false);
  }

  async function loadFile(file: File): Promise<void> {
    const extension = extensionOf(file.name);
    if (!ACCEPTED_EXTENSIONS.includes(extension)) throw new Error(`Unsupported text file type: .${extension || "unknown"}`);
    if (file.size > MAX_FILE_BYTES) throw new Error("File exceeds the 8 MiB safety limit.");
    const text = await file.text();
    inputRevision.current += 1;
    setInput(text);
    setFileName(file.name);
    setAnalysis(null);
    setResult(null);
  }

  async function onFileChange(event: ChangeEvent<HTMLInputElement>): Promise<void> {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    try {
      await loadFile(file);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function onDrop(event: DragEvent<HTMLDivElement>): Promise<void> {
    event.preventDefault();
    const file = event.dataTransfer.files?.[0];
    if (!file) return;
    try {
      await loadFile(file);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  async function copyResult(): Promise<void> {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result.output);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      setError("Clipboard access was denied.");
    }
  }

  function exportResult(): void {
    if (!result) return;
    const url = URL.createObjectURL(new Blob([result.output], { type: "text/plain;charset=utf-8" }));
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${fileName?.replace(/\.[^.]+$/, "") || "nullmark-output"}.sanitized.txt`;
    anchor.rel = "noopener";
    document.body.appendChild(anchor);
    anchor.click();
    anchor.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand">
          <img src={logoUrl} alt="" className="brand-mark" />
          <div><strong>NULLMARK</strong><span>{t("productSubtitle")}</span></div>
        </div>
        <div className="topbar-tools">
          <div className="local-indicator"><i />{t("localProcessing")}</div>
          <div className="locale-switch" aria-label={t("language")}>
            <button className={locale === "de" ? "active" : ""} onClick={() => setLocale("de")}>DE</button>
            <button className={locale === "en" ? "active" : ""} onClick={() => setLocale("en")}>EN</button>
          </div>
        </div>
      </header>

      <section className="page-intro">
        <div><h1>{t("overviewTitle")}</h1><p>{t("overviewBody")}</p></div>
        <div className="engine-state"><span>{t("status")}</span><strong>{status}</strong></div>
      </section>

      <nav className="workspace-tabs" aria-label="Workspace">
        <button className={workspaceKind === "text" ? "active" : ""} onClick={() => setWorkspaceKind("text")}>{t("textTab")}</button>
        <button className={workspaceKind === "files" ? "active" : ""} onClick={() => setWorkspaceKind("files")}>{t("fileTab")}</button>
      </nav>

      {workspaceKind === "files" ? <FileWorkspace /> : (
        <>
          {error && <div className="error-banner" role="alert">{error}</div>}
          <section className="workspace">
            <div className="panel input-panel">
              <div className="panel-head">
                <div><span className="step">01</span><h2>{t("input")}</h2></div>
                <div className="panel-actions">
                  {input && <button className="ghost" onClick={reset}>{t("clear")}</button>}
                  <button className="ghost" onClick={() => fileInput.current?.click()}>{t("openText")}</button>
                  <input ref={fileInput} className="visually-hidden" type="file" onChange={onFileChange} accept=".txt,.md,.markdown,.html,.htm,.csv,.json,.xml,.yaml,.yml,text/*" />
                </div>
              </div>
              <div className="drop-zone" onDragOver={(event) => event.preventDefault()} onDrop={onDrop}>
                <textarea
                  value={input}
                  onChange={(event) => {
                    inputRevision.current += 1;
                    setInput(event.target.value);
                    setFileName(null);
                    setAnalysis(null);
                    setResult(null);
                  }}
                  spellCheck={false}
                  autoCapitalize="off"
                  autoCorrect="off"
                  placeholder={t("inputPlaceholder")}
                  aria-label={t("input")}
                />
                <div className="input-footer"><span>{fileName ?? t("directInput")}</span><span>{input.length.toLocaleString(locale)} {t("chars")}</span></div>
              </div>
              <div className="control-row">
                <div className="mode-switch" aria-label="Sanitization mode">
                  {(["safe", "strict", "maximum"] as const).map((value) => (
                    <button key={value} className={mode === value ? "active" : ""} onClick={() => setMode(value)}>{t(value)}</button>
                  ))}
                </div>
                <div className="primary-actions">
                  <button disabled={!input || busy} className="secondary" onClick={() => void runAnalysis()}>{t("analyze")}</button>
                  <button disabled={!input || busy} className="primary" onClick={() => void runSanitize()}>{busy ? t("working") : t("sanitizeMode", { mode: t(mode) })}</button>
                </div>
              </div>
              <p className="mode-note">{t(mode === "safe" ? "safeNote" : mode === "strict" ? "strictNote" : "maximumNote")}</p>
            </div>

            <div className="panel findings-panel">
              <div className="panel-head"><div><span className="step">02</span><h2>{t("analysis")}</h2></div></div>
              {displayedReport ? (
                <>
                  <div className="summary-grid">
                    <div className="metric"><span>{t("characters")}</span><strong>{displayedReport.characters.toLocaleString(locale)}</strong></div>
                    <div className="metric"><span>{t("findings")}</span><strong>{displayedReport.finding_count.toLocaleString(locale)}</strong></div>
                    <div className="metric"><span>{t("highRisk")}</span><strong>{displayedReport.risk_counts.high}</strong></div>
                    <div className="metric"><span>{t("payload")}</span><strong>{humanBytes(displayedReport.bytes)}</strong></div>
                  </div>
                  <div className="hash-row"><span>SHA-256</span><code title={displayedReport.sha256}>{displayedReport.sha256.slice(0, 18)}…</code></div>
                  <div className="findings-list">
                    {displayedReport.findings.length === 0 ? (
                      <div className="empty-state"><strong>{t("noKnownMarkers")}</strong><span>{t("noKnownMarkersBody")}</span></div>
                    ) : displayedReport.findings.map((finding) => (
                      <article className="finding" key={finding.codepoint}>
                        <div className={`risk risk-${finding.severity}`} />
                        <div className="finding-main">
                          <div className="finding-title"><code>{finding.codepoint}</code><strong>{finding.name}</strong></div>
                          <div className="finding-meta"><span>{finding.category}</span><span>{actionLabel(finding.action)}</span><span>{t("firstAt")} {finding.first_positions.join(", ")}</span></div>
                        </div>
                        <b>×{finding.count}</b>
                      </article>
                    ))}
                  </div>
                </>
              ) : <div className="waiting-state"><strong>{t("awaiting")}</strong><span>{t("awaitingBody")}</span></div>}
            </div>
          </section>

          {result && (
            <section className="panel result-panel">
              <div className="panel-head">
                <div><span className="step">03</span><h2>{t("output")}</h2></div>
                <div className={`verify-chip ${fullyClean ? "" : "review"}`}>{fullyClean ? `✓ ${t("cleanLabel")}` : result.verification_passed ? `! ${t("policyOnly")}` : `! ${t("review")}`}</div>
              </div>
              <div className="verification-grid">
                <div><span>{t("removed")}</span><strong>{result.removed_count}</strong></div>
                <div><span>{t("normalized")}</span><strong>{result.normalized_count}</strong></div>
                <div><span>{t("remaining")}</span><strong>{result.after.finding_count}</strong></div>
                <div><span>{t("projection")}</span><strong>{result.canonical_projection_unchanged ? t("stable") : t("changed")}</strong></div>
              </div>
              <div className="scope-note"><strong>{t("verificationScope")}</strong><span>{result.verification_scope}</span><span>{t("vendorStatus", { status: result.probabilistic_watermark_status })}</span></div>
              {result.verification_passed && result.after.finding_count > 0 && (
                <div className="policy-warning" role="alert">
                  <div><strong>{t("remainWarning", { count: result.after.finding_count, mode: result.mode })}</strong><span>{t("remainBody")}</span></div>
                  {remainingRequiredMode && remainingRequiredMode !== result.mode && (
                    <button className="secondary" disabled={busy} onClick={() => { setMode(remainingRequiredMode); void runSanitize(remainingRequiredMode); }}>{t("rerun", { mode: t(remainingRequiredMode) })}</button>
                  )}
                </div>
              )}
              <div className="result-toolbar">
                <div className="view-switch">
                  <button className={resultView === "output" ? "active" : ""} onClick={() => setResultView("output")}>{t("outputView")}</button>
                  <button className={resultView === "diff" ? "active" : ""} onClick={() => setResultView("diff")}>{t("diffView")} <b>{result.change_count}</b></button>
                </div>
                <div className="result-actions">
                  <button className="secondary" onClick={() => void copyResult()}>{copied ? t("copied") : t("copy")}</button>
                  <button className="primary" onClick={exportResult}>{t("exportText")}</button>
                </div>
              </div>
              {resultView === "output" ? <textarea className="output" readOnly value={result.output} aria-label={t("output")} /> : <DiffViewer source={input} report={result} />}
            </section>
          )}
        </>
      )}

      <footer><span>{t("footerPrimary")}</span><span>{t("footerSecondary")}</span></footer>
    </main>
  );
}
