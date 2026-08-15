// STATUS: DIAMANT VGT SUPREME

import type { SanitizeReport, TextChange } from "../../types";
import { useI18n } from "../../i18n";

function codePoint(value: string): string {
  if (value.length === 0) return "∅";
  const point = value.codePointAt(0);
  if (point === undefined) return "∅";
  if (value === " ") return "SPACE · U+0020";
  if (value === "\n") return "LF · U+000A";
  if (value === "\t") return "TAB · U+0009";
  return `${value} · U+${point.toString(16).toUpperCase().padStart(4, "0")}`;
}

function ChangeRow({ change }: { change: TextChange }) {
  const { t } = useI18n();
  return (
    <tr>
      <td><code>{change.source_index}</code></td>
      <td><code>{codePoint(change.before)}</code></td>
      <td><code>{codePoint(change.after)}</code></td>
      <td><span className={`change-kind ${change.kind}`}>{change.kind === "removed" ? t("removedKind") : t("normalizedKind")}</span></td>
    </tr>
  );
}

export function DiffViewer({ source, report }: { source: string; report: SanitizeReport }) {
  const { t } = useI18n();
  return (
    <div className="diff-view">
      <div className="split-diff">
        <label>
          <span>{t("source")}</span>
          <textarea readOnly value={source} aria-label={t("source")} />
        </label>
        <label>
          <span>{t("result")}</span>
          <textarea readOnly value={report.output} aria-label={t("result")} />
        </label>
      </div>
      <div className="change-ledger">
        <div className="ledger-head">
          <strong>{t("changesTitle")}</strong>
          <code>{report.change_count}</code>
        </div>
        {report.changes.length === 0 ? (
          <p className="no-changes">{t("noChanges")}</p>
        ) : (
          <div className="table-scroll">
            <table>
              <thead><tr><th>{t("changePosition")}</th><th>{t("before")}</th><th>{t("after")}</th><th>{t("operation")}</th></tr></thead>
              <tbody>{report.changes.map((change, index) => <ChangeRow key={`${change.source_index}-${index}`} change={change} />)}</tbody>
            </table>
          </div>
        )}
        {report.changes_truncated && <p className="truncated">{t("truncated", { shown: report.changes.length, total: report.change_count })}</p>}
      </div>
    </div>
  );
}
