# Text-watermark research baseline

Last verified: 2026-08-15. This document records public primary sources and the
claims NullMark is technically allowed to make.

## Vendor matrix

| Source | Publicly documented text mechanism | NullMark Beta 1.0 coverage |
|---|---|---|
| Google Gemini | SynthID Text modifies token sampling and uses probabilistic detection. | Deterministic Unicode and token-surface canonicalization. Removal of the private SynthID signal cannot be proven without the configured vendor detector. |
| OpenAI / ChatGPT | OpenAI states that it developed and evaluated a text-watermark method, while describing global rewriting, translation, or systematic edits as weaknesses. Public deployment of a ChatGPT text watermark is not established by the cited source. | Removes explicit Unicode/edit-based payloads. No unsupported ChatGPT-specific success claim. |
| Anthropic Claude | No official public text-watermark specification or public detector was located during this research pass. | Vendor-neutral Unicode sanitation only; no unsupported Claude-specific success claim. |

## Primary sources

- Google DeepMind, [SynthID overview](https://deepmind.google/models/synthid/).
- Dathathri et al., [Scalable watermarking for identifying large language model outputs](https://www.nature.com/articles/s41586-024-08025-4), Nature 634 (2024).
- Google AI for Developers, [SynthID Text implementation and limitations](https://ai.google.dev/responsible/docs/safeguards/synthid).
- OpenAI, [Understanding the source of what we see and hear online](https://openai.com/index/understanding-the-source-of-what-we-see-and-hear-online/).
- OpenAI, [Advancing content provenance](https://openai.com/index/advancing-content-provenance/).
- C2PA, [Content Credentials Technical Specification 2.4, Appendix A](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html).

## Engineering conclusion

Text watermarks fall into distinct classes:

1. Explicit edit-based signals: zero-width code points, tag characters, bidi
   controls, unusual spacing, variation selectors, private-use payloads, and
   compatibility typography. These are inspectable and deterministically removable.
2. Generative statistical signals: token choices biased during model sampling.
   These are not hidden characters. Thorough rewriting can reduce detector
   confidence, but a local rule engine cannot truthfully certify removal.
3. Metadata/provenance: attached to a document or media container rather than
   the text stream. These require format-specific parsers and independent re-open scans.

Container carrier handling follows the C2PA 2.4 embedding annex: JPEG APP11,
PNG `caBX`, WebP RIFF `C2PA`, PDF associated-file specifications with
`AFRelationship=C2PA_Manifest`, and ZIP-based `META-INF/content_credential.c2pa`.
Carrier removal is not signature or provenance validation.

Accordingly, `verification_passed` covers only the named deterministic rule set.
The API separately returns `probabilistic_watermark_status` so the UI cannot
collapse a deterministic rescan into a universal vendor-watermark guarantee.
