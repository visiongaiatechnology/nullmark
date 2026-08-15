// STATUS: DIAMANT VGT SUPREME

import { invoke } from "@tauri-apps/api/core";
import type {
  AnalysisReport,
  BinaryAnalysisReport,
  BinarySanitizeReport,
  SanitizeMode,
  SanitizeReport
} from "../types";

const MAX_INPUT_BYTES = 8 * 1024 * 1024;

function assertInput(text: string): void {
  const bytes = new TextEncoder().encode(text).byteLength;
  if (bytes > MAX_INPUT_BYTES) {
    throw new Error("Input exceeds the 8 MiB beta safety limit.");
  }
}

export async function analyzeText(text: string): Promise<AnalysisReport> {
  assertInput(text);
  return invoke<AnalysisReport>("analyze_text", { text });
}

export async function sanitizeText(text: string, mode: SanitizeMode): Promise<SanitizeReport> {
  assertInput(text);
  return invoke<SanitizeReport>("sanitize_text", { text, mode });
}

export async function analyzeBinary(payloadBase64: string): Promise<BinaryAnalysisReport> {
  return invoke<BinaryAnalysisReport>("analyze_binary", { payloadBase64 });
}

export async function sanitizeBinary(payloadBase64: string): Promise<BinarySanitizeReport> {
  return invoke<BinarySanitizeReport>("sanitize_binary", { payloadBase64 });
}
