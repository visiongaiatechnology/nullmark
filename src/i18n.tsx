// STATUS: DIAMANT VGT SUPREME

import { createContext, ReactNode, useContext, useMemo, useState } from "react";

export type Locale = "de" | "en";

const en = {
  productSubtitle: "Local content integrity",
  localProcessing: "Local processing",
  language: "Language",
  overviewTitle: "Remove hidden text markers and document privacy traces.",
  overviewBody: "Deterministic inspection for Unicode payloads, metadata, provenance and active document content. Files never leave this device.",
  status: "Engine status",
  ready: "Ready",
  processing: "Processing locally",
  clean: "Deterministic scan clean",
  policyVerified: "{mode} policy verified · findings remain",
  findingsDetected: "Findings detected",
  noMarkers: "No known markers detected",
  textTab: "Text & Unicode",
  fileTab: "Documents & media",
  input: "Input",
  clear: "Clear",
  openText: "Open text file",
  inputPlaceholder: "Paste text here or drop a supported text file…",
  directInput: "Direct text input",
  chars: "characters",
  safe: "Safe",
  strict: "Strict",
  maximum: "Maximum",
  analyze: "Analyze",
  sanitizeMode: "Sanitize {mode}",
  working: "Working…",
  safeNote: "Removes high-confidence invisible payloads while preserving script-sensitive characters.",
  strictNote: "Also removes format controls and normalizes unusual spaces. Review complex scripts and emoji.",
  maximumNote: "Also removes private-use payloads and canonicalizes token-facing typography. Proprietary statistical watermarks remain vendor-dependent.",
  analysis: "Analysis",
  characters: "Characters",
  findings: "Findings",
  highRisk: "High risk",
  payload: "Payload",
  firstAt: "first at",
  noKnownMarkers: "No known invisible markers",
  noKnownMarkersBody: "No character covered by the current deterministic ruleset was found.",
  awaiting: "Awaiting analysis",
  awaitingBody: "Run an analysis or sanitization to inspect the current text.",
  output: "Sanitized output",
  cleanLabel: "CLEAN",
  policyOnly: "POLICY ONLY",
  review: "REVIEW",
  removed: "Removed",
  normalized: "Normalized",
  remaining: "Remaining findings",
  projection: "Canonical projection",
  stable: "STABLE",
  changed: "CHANGED",
  verificationScope: "Verification scope",
  vendorStatus: "Vendor token watermark: {status}",
  remainWarning: "{count} finding(s) remain outside the {mode} policy.",
  remainBody: "The selected policy passed, but the output is not clean under the complete deterministic ruleset.",
  rerun: "Re-run {mode}",
  outputView: "Output",
  diffView: "Changes",
  copy: "Copy output",
  copied: "Copied",
  exportText: "Export .txt",
  source: "Source",
  result: "Result",
  changesTitle: "Exact change ledger",
  changePosition: "Position",
  before: "Before",
  after: "After",
  operation: "Operation",
  removedKind: "Removed",
  normalizedKind: "Normalized",
  noChanges: "The selected policy made no changes.",
  truncated: "Showing the first {shown} of {total} changes.",
  filesTitle: "Document and media privacy",
  clearBatch: "Clear batch",
  dropFiles: "Drop PNG, JPEG, WebP, PDF, SVG, DOCX, XLSX, PPTX or ODT",
  loadedFiles: "{count} file(s) loaded",
  fileLimits: "Signature checked · 32 MiB/file · 128 MiB/batch · max. 50 · source files remain untouched",
  selectFiles: "Select files",
  sourceData: "{size} source data",
  currentFindings: "{count} current finding(s)",
  verifiedCount: "{verified}/{total} verified",
  analyzeBatch: "Analyze batch",
  cleanAll: "Privacy clean & verify all",
  processingBatch: "Processing sequentially…",
  export: "Export",
  metadata: "{count} findings",
  c2paPresent: "C2PA present",
  c2paNone: "C2PA none",
  provenanceWarning: "Signed provenance will be removed from the exported copy.",
  noFileFindings: "No supported privacy metadata detected.",
  reopenVerified: "reopen verified · {count} removed",
  verificationFailed: "verification failed · export blocked",
  footerPrimary: "NullMark Beta · deterministic local processing",
  footerSecondary: "No telemetry · no CDN · no shell · no network permission"
} as const;

type MessageKey = keyof typeof en;

const de: Record<MessageKey, string> = {
  productSubtitle: "Lokale Inhaltsintegrität",
  localProcessing: "Lokale Verarbeitung",
  language: "Sprache",
  overviewTitle: "Versteckte Textmarker und private Dokumentspuren entfernen.",
  overviewBody: "Deterministische Prüfung von Unicode-Payloads, Metadaten, Provenienz und aktiven Dokumentinhalten. Dateien verlassen dieses Gerät nicht.",
  status: "Engine-Status",
  ready: "Bereit",
  processing: "Wird lokal verarbeitet",
  clean: "Deterministischer Scan sauber",
  policyVerified: "{mode}-Richtlinie bestätigt · Funde verbleiben",
  findingsDetected: "Funde erkannt",
  noMarkers: "Keine bekannten Marker erkannt",
  textTab: "Text & Unicode",
  fileTab: "Dokumente & Medien",
  input: "Eingabe",
  clear: "Leeren",
  openText: "Textdatei öffnen",
  inputPlaceholder: "Text einfügen oder eine unterstützte Textdatei ablegen…",
  directInput: "Direkte Texteingabe",
  chars: "Zeichen",
  safe: "Sicher",
  strict: "Strikt",
  maximum: "Maximum",
  analyze: "Analysieren",
  sanitizeMode: "{mode} bereinigen",
  working: "Verarbeitung…",
  safeNote: "Entfernt eindeutige unsichtbare Payloads und bewahrt schriftkritische Zeichen.",
  strictNote: "Entfernt zusätzlich Formatsteuerzeichen und normalisiert ungewöhnliche Leerzeichen. Komplexe Schriften und Emoji prüfen.",
  maximumNote: "Entfernt zusätzlich Private-Use-Payloads und vereinheitlicht tokenrelevante Typografie. Proprietäre statistische Watermarks bleiben herstellerabhängig.",
  analysis: "Analyse",
  characters: "Zeichen",
  findings: "Funde",
  highRisk: "Hohes Risiko",
  payload: "Datenmenge",
  firstAt: "zuerst bei",
  noKnownMarkers: "Keine bekannten unsichtbaren Marker",
  noKnownMarkersBody: "Es wurde kein Zeichen aus dem aktuellen deterministischen Regelwerk gefunden.",
  awaiting: "Analyse ausstehend",
  awaitingBody: "Analyse oder Bereinigung starten, um den aktuellen Text zu prüfen.",
  output: "Bereinigte Ausgabe",
  cleanLabel: "SAUBER",
  policyOnly: "NUR RICHTLINIE",
  review: "PRÜFEN",
  removed: "Entfernt",
  normalized: "Normalisiert",
  remaining: "Verbleibende Funde",
  projection: "Kanonische Projektion",
  stable: "STABIL",
  changed: "GEÄNDERT",
  verificationScope: "Prüfumfang",
  vendorStatus: "Hersteller-Token-Watermark: {status}",
  remainWarning: "{count} Fund/Funde liegen außerhalb der {mode}-Richtlinie.",
  remainBody: "Die gewählte Richtlinie wurde bestanden, die Ausgabe ist nach dem vollständigen Regelwerk jedoch nicht sauber.",
  rerun: "Erneut mit {mode}",
  outputView: "Ausgabe",
  diffView: "Änderungen",
  copy: "Ausgabe kopieren",
  copied: "Kopiert",
  exportText: ".txt exportieren",
  source: "Quelle",
  result: "Ergebnis",
  changesTitle: "Exaktes Änderungsprotokoll",
  changePosition: "Position",
  before: "Vorher",
  after: "Nachher",
  operation: "Operation",
  removedKind: "Entfernt",
  normalizedKind: "Normalisiert",
  noChanges: "Die gewählte Richtlinie hat keine Änderungen vorgenommen.",
  truncated: "Die ersten {shown} von {total} Änderungen werden angezeigt.",
  filesTitle: "Dokument- und Medien-Privacy",
  clearBatch: "Stapel leeren",
  dropFiles: "PNG, JPEG, WebP, PDF, SVG, DOCX, XLSX, PPTX oder ODT ablegen",
  loadedFiles: "{count} Datei(en) geladen",
  fileLimits: "Signaturprüfung · 32 MiB/Datei · 128 MiB/Stapel · max. 50 · Quelldateien bleiben unverändert",
  selectFiles: "Dateien auswählen",
  sourceData: "{size} Quelldaten",
  currentFindings: "{count} aktuelle Funde",
  verifiedCount: "{verified}/{total} verifiziert",
  analyzeBatch: "Stapel analysieren",
  cleanAll: "Alle bereinigen & verifizieren",
  processingBatch: "Sequenzielle Verarbeitung…",
  export: "Exportieren",
  metadata: "{count} Funde",
  c2paPresent: "C2PA vorhanden",
  c2paNone: "C2PA keines",
  provenanceWarning: "Signierte Provenienz wird aus der exportierten Kopie entfernt.",
  noFileFindings: "Keine unterstützten privaten Metadaten erkannt.",
  reopenVerified: "erneut geöffnet & verifiziert · {count} entfernt",
  verificationFailed: "Verifizierung fehlgeschlagen · Export blockiert",
  footerPrimary: "NullMark Beta · deterministische lokale Verarbeitung",
  footerSecondary: "Keine Telemetrie · kein CDN · keine Shell · keine Netzwerkberechtigung"
};

interface I18nValue {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: MessageKey, values?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nValue | null>(null);

function initialLocale(): Locale {
  try {
    const stored = window.localStorage.getItem("nullmark.locale");
    if (stored === "de" || stored === "en") return stored;
  } catch {
    // Storage may be denied; browser language remains a deterministic fallback.
  }
  return navigator.language.toLowerCase().startsWith("de") ? "de" : "en";
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, updateLocale] = useState<Locale>(initialLocale);
  const value = useMemo<I18nValue>(() => ({
    locale,
    setLocale(next) {
      updateLocale(next);
      try {
        window.localStorage.setItem("nullmark.locale", next);
      } catch {
        // Locale still applies for the current process.
      }
    },
    t(key, values = {}) {
      const catalog = locale === "de" ? de : en;
      return Object.entries(values).reduce(
        (message, [name, replacement]) => message.replaceAll(`{${name}}`, String(replacement)),
        catalog[key]
      );
    }
  }), [locale]);
  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nValue {
  const value = useContext(I18nContext);
  if (!value) throw new Error("I18n provider missing");
  return value;
}
