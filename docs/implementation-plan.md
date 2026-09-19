# Implementation Plan

Status date: 2026-07-16

This plan tracks the remaining `plan.md` gaps and the implementation work that
has already landed. Work should land in small pull requests by track, with CI
green before release builds run.

## Completed Baseline

- Password setup/change now enforces Rust-side strength checks.
- Password and passkey unlocks share persisted exponential backoff after failed
  attempts.
- Registered passkeys can unlock the vault through the WebAuthn PRF flow.
- Tier 1 pdfium extraction is the default ingest path; `build.rs` attempts to
  fetch the matching pdfium sidecar at build time and reports runtime readiness
  through settings/status.
- Tier 2 Tesseract OCR is feature-gated behind `tesseract-ocr`; sparse PDF text
  can fall back to rendered-page OCR and persist `ingest_tier = 2` when OCR text
  is selected.
- Tier 3 model settings/status paths validate local model files, expose
  compiled, present, loading, and loaded state, and provide settings-page
  file picking plus load/unload controls. Low-confidence Tesseract output can
  invoke the local Phi-4 repair runtime and is promoted to `ingest_tier = 3`
  only when non-empty repaired text is used.
- Ingest and reparse now write `parse_audit` diagnostics for unmatched analytes,
  unparsed ranges, unrecognized units, missing values, and low-confidence rows.
- Report detail now exposes those `parse_audit` diagnostics as a compact
  parser-quality panel.
- Parser acceptance tests cover realistic extracted text for headers, rows,
  prior-result columns, tier commentary filtering, and wrapped units.
- The repository has separate CI/release and documentation workflows:
  `.github/workflows/ci.yml` and `.github/workflows/docs.yml`.
- CI runs frontend format, lint, Svelte type check, Rust format, Rust clippy,
  Rust tests, and gated Tauri build artifacts.
- Manual `workflow_dispatch` computes `YY.N` release tags, converts app metadata
  to semver, builds the matrix, and creates a GitHub Release with downloaded
  artifacts.

## 1. Tier 3 olmOCR-2

Scope:

- Keep default builds and non-model machines functional.
- Do not fake successful OCR when model runtime is unavailable.
- Replace the current status/path-validation skeleton with a real local
  olmOCR-2 runtime.
- Wire low-confidence Tier 2 output into a structured image/PDF-page OCR request
  only when the feature is compiled, enabled, and model-ready.

Acceptance:

- Settings continue to distinguish compiled, model present, loaded, and
  unavailable.
- Low-confidence tier 2 output can escalate to tier 3 when enabled and loaded;
  the successful lower tier remains the fallback when an optional model is not
  ready or fails.
- Errors clearly identify missing feature, missing model, or runtime failure.
- No successful OCR result is returned unless the model runtime actually
  produced text.

Requirements:

- Cargo feature: `embedded-ocr-vision`.
- Native/runtime dependencies: llama.cpp stack as required by `llama-cpp-2`.
- Runtime files: local olmOCR-2-compatible model path configured in settings.

## 2. Tier 4 Phi-4 Repair

Scope:

- Add a gated repair hook for low-confidence parser rows.
- Persist repair decisions into parse audit metadata.
- Extend the current local model validation/status skeleton into a real Phi-4
  repair runtime.
- Define a structured repair request containing row text, parsed fields,
  Library candidates, and parser diagnostics.

Acceptance:

- Tier 4 is opt-in and never runs unless enabled and model-ready.
- Low-confidence rows create parse-audit entries even when repair is unavailable.
- Repaired rows retain original raw text plus repair provenance.
- No successful repair result is returned unless the model runtime actually
  produced repaired output.

Requirements:

- Cargo feature: `embedded-llm`.
- Native/runtime dependencies: llama.cpp stack as required by `llama-cpp-2`.
- Runtime files: local Phi-4-compatible model path configured in settings.

## 3. Parser Audit and Fixtures

Scope:

- Add sample-PDF fixture coverage for the claimed acceptance criteria.
- Add regression tests for known CUF and Germano de Sousa layout variants.

Acceptance:

- The sample suite proves all tracked fixtures produce reports and rows.
- Unmatched analytes, unparsed ranges, and unit mismatches are visible from
  stored parse-audit diagnostics on report detail.

Current limitation:

- The repository has text-level parser acceptance coverage, but no committed PDF
  fixture corpus. Real PDF fixture acceptance needs representative, shareable
  sample files.

## 4. Cross-Platform Build Hardening

Scope:

- Keep the existing Tauri bundle matrix green for Windows x64, Linux x64, Linux
  arm64, macOS x64, and macOS arm64.
- Confirm hosted runner labels and Tauri system package names as GitHub images
  evolve.
- Add optional signed-build coverage once signing providers are chosen.

Acceptance:

- Builds run only after checks pass.
- Linux runners install WebKit and Tauri system dependencies.
- macOS uses distinct Intel and Apple Silicon runner labels.
- Signing is optional by default, with documented secret names for signed builds.

## 5. Automated Release Hardening

Scope:

- Keep the manual release workflow aligned with the build matrix.
- Validate release assets and naming on each supported platform.
- Add signed/notarized release coverage after project-specific signing secrets
  are configured.

Acceptance:

- Triggering `workflow_dispatch` on July 2026 creates tags like `26.1`,
  `26.2`, etc.
- Release creation uses GitHub token permissions scoped to contents write.
- Release notes include the generated version and commit SHA.
- Unsigned builds are clearly labeled until signing secrets are configured.

## CI And Release Notes

The CI/release workflow is `.github/workflows/ci.yml`; the documentation site
is built and deployed by `.github/workflows/docs.yml`.

On pull requests and pushes it runs checks plus gated Tauri artifacts. On manual
`workflow_dispatch` it computes the next `YY.N` tag for the current UTC year,
uses `YY.N.0` for Tauri/package metadata, builds all matrix entries, and
publishes a GitHub Release.

Signing and update secrets are optional. The workflow currently reads:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`
- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Windows code signing still needs a project-specific certificate or signing
provider configuration before signed Windows installers can be expected.
