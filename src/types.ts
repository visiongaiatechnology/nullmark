export type Severity = "info" | "low" | "medium" | "high";
// STATUS: DIAMANT VGT SUPREME

export type Action =
  | "remove_safe"
  | "remove_strict"
  | "remove_maximum"
  | "normalize_strict"
  | "normalize_maximum"
  | "report_only";
export type SanitizeMode = "safe" | "strict" | "maximum";

export interface FindingGroup {
  codepoint: string;
  name: string;
  category: string;
  severity: Severity;
  action: Action;
  count: number;
  first_positions: number[];
}

export interface RiskCounts {
  info: number;
  low: number;
  medium: number;
  high: number;
}

export interface AnalysisReport {
  rule_set_version: string;
  bytes: number;
  characters: number;
  finding_count: number;
  sha256: string;
  risk_counts: RiskCounts;
  findings: FindingGroup[];
}

export interface SanitizeReport {
  output: string;
  mode: SanitizeMode;
  removed_count: number;
  normalized_count: number;
  verification_passed: boolean;
  canonical_projection_unchanged: boolean;
  verification_scope: string;
  probabilistic_watermark_status: "not-verifiable-without-vendor-detector";
  change_count: number;
  changes_truncated: boolean;
  changes: TextChange[];
  before: AnalysisReport;
  after: AnalysisReport;
}

export interface TextChange {
  source_index: number;
  output_index: number;
  before: string;
  after: string;
  kind: "removed" | "normalized";
}

export interface BinaryFinding {
  kind: string;
  count: number;
  description: string;
}

export interface BinaryAnalysisReport {
  format: "png" | "jpeg" | "webp" | "docx" | "xlsx" | "pptx" | "odt" | "svg" | "pdf";
  mime: "image/png" | "image/jpeg" | "image/webp" | "image/svg+xml" | "application/vnd.openxmlformats-officedocument.wordprocessingml.document" | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" | "application/vnd.openxmlformats-officedocument.presentationml.presentation" | "application/vnd.oasis.opendocument.text" | "application/pdf";
  bytes: number;
  sha256: string;
  metadata_count: number;
  c2pa_detected: boolean;
  findings: BinaryFinding[];
}

export interface BinarySanitizeReport {
  output_base64: string;
  removed_items: number;
  verification_passed: boolean;
  before: BinaryAnalysisReport;
  after: BinaryAnalysisReport;
}
