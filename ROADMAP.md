# NullMark roadmap status

## Implemented in 1.0.0-beta.2

- Exact source/output split view with a bounded 4,096-entry change ledger.
- Character positions, Unicode code points and removal/normalization labels.
- XLSX shared strings, properties and comment/person authorship sanitation.
- PPTX slide XML, properties and comment-author sanitation.
- Strict SVG parsing with metadata/comment/script/event/external-reference removal.
- PDF document/page XMP, annotation identity fields and JavaScript/launch/action removal.
- Complete renderer catalogs for German and English with local preference persistence.
- Project-owned SVG identity and a restrained desktop information design.
- Independent reparse and re-analysis before cleaned file export.

## Deliberate policy boundary

Regular PDF attachments are inventoried structurally by the parser but are not
silently deleted. Only C2PA associated-file manifests are removed automatically.
A future attachment-removal control must be explicit per file and must show the
attachment name, MIME claim and size before modification.

## Follow-up gates

- Virtualized change-table rendering above the current bounded 4,096-row ledger.
- Signed Windows/macOS distribution and reproducible release provenance.
- Hostile real-world corpus expansion for complex Office relationships and SVG namespaces.
- Explicit PDF attachment inventory/export/removal workflow.
