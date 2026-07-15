# blevel-tracker — Engineering Plan

A fully local, embedded desktop application that ingests clinical-pathology PDFs (CUF / Germano de Sousa format), extracts every analyte programmatically, and presents a clinician-grade longitudinal dashboard with deltas, reference bands, panels, and full dark/light theming.

No network calls. No cloud APIs. All inference, OCR, and storage stay on-device.

---

## 1. Goals & Non-Goals

### Goals
- Parse every analyte in every PDF with high precision via deterministic, rule-based methods.
- Embedded-only LLM/OCR tiers — opt-in via settings, lazy-loaded, never required for the happy path.
- Encrypted-at-rest storage with **passkey-first** unlock and **password fallback**.
- Clinician-grade longitudinal UI: trends, deltas, reference bands, panels, sex/age-aware ranges, unit toggles.
- Cross-platform via Tauri 2; primary target Windows 11 (user platform).
- Zero telemetry. Zero outbound network.

### Non-Goals
- Multi-user / cloud sync (single-user local app).
- HL7 / FHIR ingestion (PDF-only for v1).
- Diagnostic recommendations — interpretation surfaces are descriptive, not prescriptive.

---

## 2. Architecture Overview

```
┌─────────────────────────── Tauri 2 process ───────────────────────────┐
│                                                                       │
│  WebView2 (Svelte 5 + ECharts)                                        │
│  ├─ Routes: dashboard / patient / analyte / report / compare / ingest │
│  ├─ Theme: CSS custom properties + chart palette registry             │
│  └─ tauri-invoke API layer                                            │
│                                                                       │
│            ▲                                                          │
│            │  invoke()                                                │
│            ▼                                                          │
│  Rust core                                                            │
│  ├─ commands/         Tauri command handlers                          │
│  ├─ crypto/           passkey (WebAuthn PRF) + Argon2id pw KDF        │
│  ├─ db/               SQLCipher (rusqlite bundled-sqlcipher)          │
│  ├─ pdf/              pdfium-render text + layout extraction          │
│  ├─ ocr/              tier 2: Tesseract (eng+por) — always available  │
│  ├─ ocr_vision/       tier 3: olmOCR-2 — opt-in, lazy load            │
│  ├─ llm/              tier 4: Phi-4-mini-reasoning — opt-in           │
│  ├─ parse/            ~100 deterministic parser methods               │
│  ├─ normalize/        canonicalization, units, flags, deltas          │
│  ├─ ontology/         analyte registry seeded from sample PDFs        │
│  └─ audit/            confidence scoring + repair triggers            │
│                                                                       │
└───────────────────────────────────────────────────────────────────────┘
```

---

## 3. Tech Stack

| Layer        | Choice                                  | Rationale |
|--------------|-----------------------------------------|-----------|
| Shell        | Tauri 2.x                               | Tiny installer, native WebView2 on Windows. |
| Frontend     | Svelte 5 (runes) + Vite                 | Best perf-to-DX ratio; reactivity model fits clinical dashboards. |
| Charts       | Apache ECharts (`echarts` 5.x)          | First-class theming, ref-band shading, boxplots, candlesticks. |
| UI primitives| `bits-ui` + `tailwindcss-variants`      | Headless components, themeable. |
| Style        | Tailwind 4 + CSS custom properties      | Dual theme via CSS vars; chart palette mirrors. |
| DB           | SQLCipher via `rusqlite` (bundled)      | Encrypted-at-rest with battle-tested AES-256. |
| KDF          | `argon2` (Argon2id, m=64MB, t=3, p=1)   | Modern password-stretching. |
| Passkey      | WebAuthn via WebView2 + PRF extension   | Hardware-backed unlock. |
| PDF          | `pdfium-render` (or `lopdf` fallback)   | Highest text-extraction fidelity. |
| OCR (no-LLM) | `tesseract` (Leptess / `rusty-tesseract`) | Battle-tested, ~30MB traineddata, no model GPU. |
| OCR (vision) | olmOCR-2 (Qwen2.5-VL-7B-derived)        | Opt-in, ~4.5GB Q4 GGUF or sidecar. |
| LLM          | Phi-4-mini-reasoning Q4_K_M             | ~2.3GB, llama-cpp-2 bindings. |

---

## 4. Encryption & Authentication

### 4.1 Threat model
Single-user local DB on a personal device. Threats: device theft, file exfil, casual access by other users on the same machine. **Out of scope:** privileged malware on the unlocked device.

### 4.2 Key hierarchy

```
┌─────────────────────────────┐
│ DB Master Key (DMK)         │  256-bit random, generated once at first run
│ (never stored in plaintext) │  used directly as SQLCipher PRAGMA key
└────────────┬────────────────┘
             │  wrapped twice (XChaCha20-Poly1305)
   ┌─────────┴─────────┐
   ▼                   ▼
┌─────────┐       ┌──────────┐
│ KEK_PK  │       │ KEK_PW   │
│ passkey │       │ password │
└────┬────┘       └─────┬────┘
     │                  │
WebAuthn PRF        Argon2id
(salt per cred)     (salt per device, m=64MB t=3 p=1)
```

- **DMK** is the SQLCipher key — never persisted in plaintext, never leaves memory.
- Two **wrapped copies** of the DMK live in `keystore.bin`:
  - `wrap_pk`: encrypted under KEK derived from passkey PRF output.
  - `wrap_pw`: encrypted under KEK derived from `Argon2id(password, salt)`.
- **Either method unlocks**. Both unwraps yield the same DMK → same DB.
- Adding a new passkey or rotating the password re-wraps DMK; DMK itself never rotates unless the user explicitly requests "re-encrypt DB".
- Failed-unlock counter persisted; exponential backoff after 5 failures.

### 4.3 Passkey flow

1. **Register** (first-run or "Add passkey" in Settings):
   - `navigator.credentials.create()` with `prf: { eval: { first: <fixed 32-byte salt> } }`.
   - Persist `credentialId`, `prfSalt`, `wrap_pk` ciphertext.
2. **Unlock**:
   - `navigator.credentials.get()` with PRF eval → 32-byte secret.
   - HKDF(secret, info=`"blevel-tracker DMK wrap v1"`) → KEK_PK.
   - Decrypt `wrap_pk` → DMK.
3. **WebView2 / WebAuthn caveat**: PRF requires WebView2 ≥ 122 + Windows Hello + a PRF-capable authenticator. If unsupported, surface a clear "passkey unavailable on this device — use password" message. Detection happens at register time.

### 4.4 Password flow

1. **Set password** (first-run mandatory; passkey is optional secondary):
   - Generate 16-byte salt.
   - KEK_PW = Argon2id(password, salt).
   - `wrap_pw` = XChaCha20-Poly1305-Encrypt(DMK, KEK_PW).
2. **Unlock**: prompt → derive KEK_PW → unwrap.
3. **Strength meter** at set time using `zxcvbn-rs`. Minimum entropy gate: 14 bits (configurable, but enforced ≥ 10).

### 4.5 Tauri-side implementation
- All key material lives in pinned `secrecy::Secret<[u8; 32]>` buffers — zeroized on drop.
- `CryptoCommand::unlock` returns only a session handle to the frontend; raw keys never cross the IPC boundary.
- DB connection lives in a `tokio::Mutex<rusqlite::Connection>` inside the Rust core, exposed only via narrow command surfaces.

---

## 5. Ingestion Pipeline (Tiered)

Each tier escalates only on low confidence from the prior. The user can disable any tier in Settings; tier 4 (LLM) is **off by default**.

```
PDF
 │
 ▼
[Tier 1] pdfium digital text extraction        ← always on, no model
 │
 │  if no text layer / low char density
 ▼
[Tier 2] Tesseract OCR (eng+por)               ← always available, no LLM
 │
 │  if ocr confidence < threshold OR layout corrupted
 ▼
[Tier 3] olmOCR-2 vision OCR                   ← opt-in, lazy load (~4.5GB)
 │
 ▼
Programmatic parser pipeline (~100 methods)
 │
 │  if rule-based parse confidence < threshold per row
 ▼
[Tier 4] Phi-4-mini-reasoning repair pass      ← opt-in, lazy load (~2.3GB)
 │
 ▼
Persist to encrypted SQLite + audit log
```

### 5.1 Confidence scoring (drives tier escalation)

Each parsed row gets a `confidence ∈ [0,1]` derived from:
- analyte name match score (1.0 exact, 0.8 alias, 0.6 fuzzy ≥ 90%, 0.0 miss)
- value parse status (numeric-clean = 1.0, qualitative-known = 1.0, unparsed = 0.0)
- range parse status (grammar match = 1.0, unrecognized = 0.0)
- unit match against analyte's expected units (1.0 / 0.5 / 0.0)

Row confidence = weighted geo-mean. Document confidence = min row confidence within a section.

Thresholds (tunable in Settings):
- `tier_2_escalate_below` = 0.4 (text density / pdfium-confidence)
- `tier_3_escalate_below` = 0.55 (Tesseract `mean_confidence`)
- `tier_4_escalate_below` = 0.7 (parser row confidence)

---

## 6. The ~100 Parser Methods

Organized into 10 modules. Every method is a pure function (`&Tokens → Result<T, ParseDiagnostic>`), independently unit-testable.

### 6.1 Layout & tokenization (10)
1. `extract_pages_pdfium` — page-aware text + bounding boxes.
2. `dehyphenate_lines` — fix soft-hyphen line breaks.
3. `reflow_columns` — reconstruct multi-column layouts via x-coordinate clustering.
4. `detect_column_boundaries` — k-means on x-positions of right-aligned numerics.
5. `group_lines_to_rows` — y-band coalescing for multi-line analyte rows.
6. `strip_page_headers` — remove repeated CUF letterhead.
7. `strip_page_footers` — remove "Pólo Tecnológico de Lisboa…" footer.
8. `find_lab_letterhead` — anchor for layout calibration.
9. `detect_logo_band` — exclude logo region from text reflow.
10. `detect_signature_block` — strip "Dr. X" trailer to avoid spurious analytes.

### 6.2 Section detection (10)
11. `detect_section_HEMATOLOGIA`
12. `detect_section_PATOLOGIA_QUIMICA`
13. `detect_section_IMUNOLOGIA`
14. `detect_section_MICROBIOLOGIA`
15. `detect_section_SEROLOGIA`
16. `detect_section_ENDOCRINOLOGIA`
17. `detect_subsection_eritrograma_leucograma_trombocitograma`
18. `detect_subsection_metabolismo_*` (lipídico, hidratos, fosfocálcico, hidroelectrolítico)
19. `detect_subsection_funcao_*` (renal, hepato-biliar)
20. `detect_subsection_eixo_*` (hipófiso-tiroideu, hipófiso-gonadal)

### 6.3 Header / metadata extraction (10)
21. `extract_patient_name` (Exmo Sr. / Exma Sra. block)
22. `extract_patient_address`
23. `extract_collection_date` (`Data de colheita`)
24. `extract_emission_date` (`Data de emissão`)
25. `extract_patient_age`
26. `extract_patient_sex_inferred` (Sr./Sra., reference-range hints)
27. `extract_inscription_id` (`Nº Inscrição`)
28. `extract_process_id` (`Nº Processo`)
29. `extract_origin_id` (`Nº Origem`)
30. `extract_clinic_entity` + `extract_requesting_physician`

### 6.4 Reference-range grammar (15)
31. `parse_range_a_b` — `13.0 - 17.0`
32. `parse_range_lt` — `< 190`
33. `parse_range_lte` — `<= 42`
34. `parse_range_gt` — `> 5.4`
35. `parse_range_gte` — `>= 60`
36. `parse_range_eq` — `= 0` (rare)
37. `parse_range_categorical_thresholds` — Vit D Deficiência/Insuficiência/Suficiência/Toxicidade tiers
38. `parse_range_age_stratified` — IgE table by age (0d, 6 sem, 3 m, …, Adultos)
39. `parse_range_sex_stratified` — male/female differentiated bounds
40. `parse_range_cycle_phase` — Estradiol Fase folicular / Ovulação / Fase luteínica / Pós-Menopausa
41. `parse_range_gestational_trimester` — TSH 1º/2º/3º trimestre
42. `parse_range_qualitative` — Negativo / Positivo / Não detectado / Reactivo / Não Reactivo
43. `parse_range_titer` — 1:80, 1:160, etc.
44. `parse_range_with_caveat` — pre-existing condition footnotes ("Pacientes com d. coronária: <175")
45. `parse_range_open_ended` — single-bound or guidance-only ranges.

### 6.5 Row parsing (15)
46. `parse_standard_row` — `analyte | result | unit | ref`
47. `parse_row_with_1_prior_result`
48. `parse_row_with_2_prior_results`
49. `parse_row_with_3_prior_results`
50. `parse_qualitative_row`
51. `parse_age_table_row` — IgE table
52. `parse_cycle_phase_row` — Estradiol multi-phase block
53. `parse_continuation_row` — analyte name spans multiple lines
54. `parse_indented_subitem` — Neutrófilos under Leucograma
55. `parse_paired_value_row` — `% / 5.54 x 10^3` percent + absolute
56. `parse_calculated_value_row` — TFGe, Glicémia média estimada
57. `parse_ratio_row` — Razão Albuminúria/Creatinúria, Colesterol Total/HDL
58. `parse_titer_row`
59. `parse_panel_summary_row`
60. `parse_method_annotation_line` — italic methodology in parentheses

### 6.6 Value parsing (10)
61. `parse_numeric_decimal` — handles PT comma `13,8` and EN dot `13.8`
62. `parse_numeric_integer`
63. `parse_numeric_with_thousands_sep`
64. `parse_qualitative_value`
65. `parse_titer_value`
66. `parse_scientific_notation` — `x 10^3`, `x 10⁶`
67. `parse_percent_value`
68. `parse_inequality_value` — `< 0.1`, `>= 100.0`
69. `detect_critical_marker` — bold/asterisk in source, layout-derived
70. `detect_flag_marker` — H/L/* badges from layout

### 6.7 Unit handling (10)
71. `normalize_unit_g_dl`
72. `normalize_unit_mg_dl`
73. `normalize_unit_mmol_l`
74. `normalize_unit_per_uL` — handles `/µl`, `/uL`, `/μL` variants
75. `normalize_unit_si_conversion` — derive SI alt unit when relevant
76. `parse_compound_unit` — `mUI/l`, `ng/dL`, `pg/ml`
77. `detect_unitless_test`
78. `resolve_unit_aliases` — `10^3/µL` ≡ `K/uL` ≡ `× 10³/µL`
79. `derive_alternate_unit` — glucose mg/dL ↔ mmol/L, cholesterol mg/dL ↔ mmol/L
80. `validate_unit_for_analyte` — flag unit mismatch from ontology

### 6.8 Analyte canonicalization (10)
81. `fuzzy_match_pt_name` — Levenshtein ≥ 0.92 against ontology.
82. `resolve_acronym` — TSH, GGT, AST, ALT, HDL, LDL, eGFR, …
83. `dedup_diacritics` — `é/e`, `á/a` collapse for matching only.
84. `resolve_synonym` — Hemoglobina ↔ hemoglobin ↔ Hb; Eritrócitos ↔ RBC.
85. `infer_analyte_from_unit_hint` — disambiguate via expected unit.
86. `infer_analyte_from_ref_range_hint` — disambiguate via expected range.
87. `promote_unmatched_to_review_queue` — surface as user-confirmable mapping.
88. `resolve_panel_membership` — Hemograma, Lipid panel, CMP, Thyroid, etc.
89. `apply_loinc_mapping` (optional, behind feature flag) — LOINC codes for export.
90. `parse_method_annotation` — Quimioluminescência, HPLC, Jaffé Cinético modificado, etc.

### 6.9 Audit & repair (10)
91. `compute_row_confidence` — weighted geo-mean of sub-scores.
92. `detect_layout_anomaly` — column count mismatch on a row.
93. `detect_orphan_value` — value with no left-side analyte name.
94. `flag_unrecognized_analyte` — push to review queue.
95. `flag_unparsed_range` — range string didn't match any grammar rule.
96. `flag_unit_mismatch` — unit not in analyte's expected set.
97. `flag_missing_required_field` — value or range missing.
98. `trigger_llm_repair` — gated by tier-4 toggle and per-row threshold.
99. `trigger_ocr_fallback` — escalate to Tesseract or olmOCR-2.
100. `write_parse_audit` — persist the full diagnostic trail.

### Method conventions
- Every method has matching test fixtures in `parse/fixtures/` derived from the 14 sample PDFs (after the ontology-bootstrap agent finishes).
- All methods log to a structured `ParseDiagnostic` enum so the audit page can render exactly which method matched (or didn't).

---

## 7. Analyte Ontology

Bootstrapped from the 14 sample PDFs by an agent (running now). Output written to `ontology/analytes.seed.json` with shape:

```json
{
  "analytes": [
    {
      "id": "hemoglobina",
      "pt_name": "Hemoglobina",
      "aliases": ["Hb"],
      "loinc": "718-7",
      "section": "HEMATOLOGIA",
      "subsection": "Eritrograma",
      "panel": "hemograma",
      "expected_units": ["g/dl"],
      "range_grammar": ["a_b"],
      "sex_dependent": true,
      "default_ref": { "m": [13.0, 17.0], "f": [12.0, 15.0] }
    }
  ],
  "panels": [...],
  "sections": [...]
}
```

Ontology is editable in-app under Settings → Ontology. Edits persist as overlays; the seed file is read-only.

---

## 8. Database Schema (SQLCipher)

```sql
-- 8.1 Identity
CREATE TABLE patients (
  id TEXT PRIMARY KEY,                 -- canonical slug
  display_name TEXT NOT NULL,
  sex TEXT CHECK (sex IN ('m','f','x','?')) DEFAULT '?',
  dob_iso TEXT,                        -- nullable
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

-- 8.2 Reports = one PDF
CREATE TABLE reports (
  id TEXT PRIMARY KEY,
  patient_id TEXT NOT NULL REFERENCES patients(id) ON DELETE CASCADE,
  source_path TEXT NOT NULL,
  source_sha256 TEXT NOT NULL UNIQUE,  -- idempotent ingest
  collection_date_iso TEXT NOT NULL,
  emission_date_iso TEXT,
  age_at_collection INTEGER,
  lab_entity TEXT,
  requesting_physician TEXT,
  inscription_id TEXT,
  process_id TEXT,
  origin_id TEXT,
  ingest_tier INTEGER NOT NULL,        -- 1=pdfium, 2=tesseract, 3=olmocr, 4=llm-repaired
  parse_version TEXT NOT NULL,         -- parser version stamp
  doc_confidence REAL NOT NULL,
  raw_text BLOB NOT NULL,              -- compressed extracted text
  raw_pdf_path TEXT NOT NULL,          -- relative to data dir
  created_at INTEGER NOT NULL
);

CREATE INDEX idx_reports_patient_date ON reports(patient_id, collection_date_iso);

-- 8.3 Analyte registry (canonical)
CREATE TABLE analytes (
  id TEXT PRIMARY KEY,                 -- 'hemoglobina'
  pt_name TEXT NOT NULL,
  loinc TEXT,
  section TEXT NOT NULL,
  subsection TEXT,
  panel TEXT,
  is_qualitative INTEGER NOT NULL DEFAULT 0,
  expected_units_json TEXT NOT NULL,   -- ["g/dl"]
  default_ref_json TEXT,               -- {"m":[13,17],"f":[12,15]}
  sex_dependent INTEGER NOT NULL DEFAULT 0,
  age_dependent INTEGER NOT NULL DEFAULT 0,
  cycle_dependent INTEGER NOT NULL DEFAULT 0
);

-- 8.4 Aliases (synonyms / abbreviations)
CREATE TABLE analyte_aliases (
  alias TEXT PRIMARY KEY,
  analyte_id TEXT NOT NULL REFERENCES analytes(id) ON DELETE CASCADE,
  source TEXT NOT NULL                 -- 'seed' | 'user'
);

-- 8.5 Per-report parsed results
CREATE TABLE results (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
  analyte_id TEXT REFERENCES analytes(id),       -- NULL if unmatched
  raw_analyte_text TEXT NOT NULL,                -- preserve printed name
  value_numeric REAL,
  value_qualitative TEXT,                        -- 'positivo'/'negativo'/...
  value_titer TEXT,                              -- '1:80'
  value_raw_text TEXT NOT NULL,                  -- always preserved
  unit TEXT,
  unit_raw TEXT NOT NULL,
  ref_low REAL,
  ref_high REAL,
  ref_grammar TEXT NOT NULL,                     -- 'a_b' | 'lt' | ... | 'unparsed'
  ref_raw_text TEXT,
  flag TEXT CHECK (flag IN ('low','normal','high','critical_low','critical_high','abnormal_qual','?')),
  method_annotation TEXT,
  parse_method TEXT NOT NULL,                    -- which of ~100 methods produced it
  confidence REAL NOT NULL,
  inline_prior_pdf BOOLEAN NOT NULL DEFAULT 0,   -- true if from "Resultados anteriores" col
  collection_date_iso TEXT NOT NULL              -- denorm for fast time-series
);

CREATE INDEX idx_results_analyte_date ON results(analyte_id, collection_date_iso);
CREATE INDEX idx_results_report ON results(report_id);

-- 8.6 Inline prior dedup precedence
-- Rule: if (analyte_id, collection_date_iso, patient) appears with both inline_prior_pdf=1
--       and inline_prior_pdf=0 records, prefer the inline_prior_pdf=0 (the original report)
-- Enforced in the query layer via a view:
CREATE VIEW results_canonical AS
SELECT r.* FROM results r
JOIN reports rep ON rep.id = r.report_id
WHERE NOT EXISTS (
  SELECT 1 FROM results r2
  JOIN reports rep2 ON rep2.id = r2.report_id
  WHERE r2.analyte_id = r.analyte_id
    AND r2.collection_date_iso = r.collection_date_iso
    AND rep2.patient_id = rep.patient_id
    AND r2.inline_prior_pdf = 0
    AND r.inline_prior_pdf = 1
);

-- 8.7 Audit trail
CREATE TABLE parse_audit (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  report_id TEXT NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
  row_index INTEGER NOT NULL,
  diagnostic TEXT NOT NULL,            -- enum-tag JSON
  parse_method TEXT,
  confidence REAL,
  llm_repaired INTEGER NOT NULL DEFAULT 0,
  llm_prompt_hash TEXT,
  ocr_tier INTEGER NOT NULL
);

-- 8.8 Settings
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL
);
-- seeded keys: tier_thresholds, llm_enabled, olmocr_enabled, theme,
-- units_preference, locale, passkey_credentials, password_argon2_params
```

**Migrations** via `refinery` or hand-rolled `schema_migrations` table. Forward-only.

---

## 9. Clinical UX (the "all the controls" list)

### 9.1 Reference bands (chart background)
- Green band = normal range.
- Yellow band = borderline (ref ± 10% if not specified).
- Red band = critical (configurable per analyte).
- Banded shading rendered as ECharts `markArea`, palette swaps with theme.

### 9.2 Per-result indicators
- Δ vs immediately prior: absolute + percent + arrow (▲ ▼ →).
- Δ vs first recorded: shown on hover.
- Z-score against reference midpoint (when both bounds present).
- H / L / Crit badges, color-coded.

### 9.3 Per-analyte detail page
- Time-series line + ref bands.
- Sparkline summary card with min/max/mean/last.
- Table of every reading with method, source PDF, parser-method tag.
- Toggle: show inline-prior-PDF readings vs original-report readings.
- Unit toggle when alternate unit is available.
- Sex-/age-/cycle-aware ref selection at hover-time.

### 9.4 Dashboard
- Most-recent snapshot per patient in a panel grid: Hemograma, Lipid, CMP, Thyroid, Vitamins, Hormones, etc.
- Each tile: latest value, Δ vs prior, status pill (Normal / Borderline / High / Low / Critical).
- "Worth-watching" section: analytes trending toward abnormal (slope test over last 3 readings).
- "Flagged" section: out-of-range latest values.

### 9.5 Reports view
- Split-pane: PDF on left (rendered via pdfium-render to canvas), parsed table on right.
- Click a parsed row → highlights the source bbox in the PDF.
- "Re-parse with LLM" button per row when LLM tier is enabled.

### 9.6 Compare view
- Pick 2 reports (same or different patient) side-by-side.
- Auto-aligned by analyte; deltas computed.
- Pick 2 patients → cohort view (e.g., comparing siblings).

### 9.7 Ingest
- Drag-drop folder or files.
- Live progress: per-file tier reached, confidence, analytes parsed.
- Conflict resolution prompts (new patient detected, duplicate report SHA, etc.).

### 9.8 Settings
- Theme (Light / Dark / System).
- LLM tier enable + model path.
- olmOCR-2 tier enable + model path.
- Tesseract enable.
- Tier thresholds (advanced).
- Unit preferences (mg/dL ↔ mmol/L, etc.).
- Auth: add passkey, change password, recovery codes.
- Data: export, import, wipe.

---

## 10. Frontend Structure

```
src/
├── app.html
├── app.css                         # CSS variables for both themes
├── lib/
│   ├── api/                        # tauri invoke wrappers
│   │   ├── ingest.ts
│   │   ├── reports.ts
│   │   ├── analytes.ts
│   │   ├── auth.ts
│   │   └── settings.ts
│   ├── theme/
│   │   ├── store.ts                # theme rune store
│   │   ├── tokens.ts               # design tokens
│   │   └── echarts-themes.ts       # registered ECharts themes
│   ├── charts/
│   │   ├── TimeSeries.svelte       # line + ref bands + crit markers
│   │   ├── PanelSnapshot.svelte    # tile with sparkline
│   │   ├── DeltaBadge.svelte
│   │   ├── ReferenceBand.svelte
│   │   └── ZScoreGauge.svelte
│   ├── components/
│   │   ├── PatientPicker.svelte
│   │   ├── PdfDropzone.svelte
│   │   ├── PdfViewer.svelte
│   │   ├── ParsedRowTable.svelte
│   │   ├── FlagPill.svelte
│   │   ├── UnlockGate.svelte
│   │   └── SettingsPanel.svelte
│   └── format/
│       ├── numbers.ts              # locale-aware decimal formatting
│       ├── dates.ts
│       └── units.ts
├── routes/
│   ├── +layout.svelte              # theme + auth gate
│   ├── +page.svelte                # dashboard
│   ├── patient/[id]/+page.svelte
│   ├── analyte/[id]/+page.svelte
│   ├── report/[id]/+page.svelte
│   ├── compare/+page.svelte
│   ├── ingest/+page.svelte
│   ├── audit/+page.svelte
│   └── settings/+page.svelte
└── styles/
    └── tailwind.css
```

---

## 11. Theming (dark + light)

### 11.1 CSS custom properties
A single `app.css` defines tokens for both modes. The `data-theme` attribute on `<html>` switches the active set.

```css
:root[data-theme='light'] {
  --bg-1: #ffffff;
  --bg-2: #f6f7f9;
  --fg-1: #0a0a0a;
  --fg-2: #4b5563;
  --line: #e5e7eb;
  --accent: #2563eb;
  --ok: #16a34a;
  --warn: #d97706;
  --crit: #dc2626;
  --band-normal: rgba(22,163,74,0.10);
  --band-borderline: rgba(217,119,6,0.10);
  --band-critical: rgba(220,38,38,0.10);
}
:root[data-theme='dark'] {
  --bg-1: #0b0d10;
  --bg-2: #14171c;
  --fg-1: #f3f4f6;
  --fg-2: #9ca3af;
  --line: #1f242b;
  --accent: #60a5fa;
  --ok: #22c55e;
  --warn: #f59e0b;
  --crit: #ef4444;
  --band-normal: rgba(34,197,94,0.12);
  --band-borderline: rgba(245,158,11,0.12);
  --band-critical: rgba(239,68,68,0.14);
}
```

### 11.2 ECharts theme registry
Two themes (`blevel-light`, `blevel-dark`) registered at app boot. Charts read the active CSS vars and pass them to ECharts via `getComputedStyle(document.documentElement)`.

### 11.3 Theme rune
```ts
export const theme = $state({ mode: 'system' as 'light'|'dark'|'system' });
```
A `MediaQueryList` listener flips `data-theme` and re-applies the ECharts theme on every chart instance.

---

## 12. LLM Integration & Toggles

### 12.1 Phi-4-mini-reasoning
- `llama-cpp-2` Rust bindings.
- Lazy-load on first need; emit progress to frontend.
- Two prompt families:
  - **Repair** — given a malformed row + section + ontology subset, return canonical JSON.
  - **Interpret** — given a free-text section (e.g., the IgE table footnote), summarize.
- Strict JSON-output mode via grammar (gbnf).

### 12.2 olmOCR-2
- Two integration options, decided at scaffold time after testing:
  - (a) Native via `llama-cpp-2` if Qwen2.5-VL multimodal is available in our llama.cpp build.
  - (b) Sidecar Python (`olmocr` CLI) over stdio, started on demand.
- Either way: opt-in toggle, lazy load, large-model warning, progress UI.

### 12.3 Settings model
```json
{
  "llm": {
    "enabled": false,
    "model_path": "models/phi-4-mini-reasoning-Q4_K_M.gguf",
    "n_ctx": 8192,
    "n_threads": "auto",
    "trigger_below_confidence": 0.7
  },
  "olmocr": {
    "enabled": false,
    "model_path": "models/olmocr-2-q4.gguf",
    "trigger_below_tesseract_confidence": 0.55
  },
  "tesseract": {
    "enabled": true,
    "languages": ["eng", "por"]
  }
}
```

---

## 13. Build & Distribution

- Lean installer: ~25MB Tauri binary + ~30MB Tesseract traineddata (eng+por) bundled.
- Models **not bundled** — first-run wizard offers download to `%APPDATA%/blevel-tracker/models/` with SHA-256 verification.
- Code-signed Windows MSI (user provides cert if desired; unsigned otherwise).
- Auto-update **disabled** (local app, no network).

---

## 14. Dependencies

### Rust crates
```toml
tauri = { version = "2", features = ["protocol-asset"] }
rusqlite = { version = "0.31", features = ["bundled-sqlcipher-vendored-openssl"] }
argon2 = "0.5"
chacha20poly1305 = "0.10"
hkdf = "0.12"
sha2 = "0.10"
secrecy = "0.10"
zeroize = "1"
pdfium-render = "0.8"
tesseract = "0.15"                   # or rusty-tesseract
llama-cpp-2 = "0.1"                  # Phi-4 + (maybe) olmOCR
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
once_cell = "1"
strsim = "0.11"                      # fuzzy matching
ahash = "0.8"
tokio = { version = "1", features = ["rt-multi-thread","macros","fs","sync"] }
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
```

### Frontend
```json
{
  "svelte": "^5",
  "@sveltejs/kit": "^2",
  "vite": "^5",
  "@tauri-apps/api": "^2",
  "echarts": "^5.5",
  "tailwindcss": "^4",
  "bits-ui": "^0.x",
  "zod": "^3",
  "date-fns": "^3"
}
```

---

## 15. Phased Build Plan

| Phase | Scope | Exit criterion |
|-------|-------|----------------|
| 0 | Plan + ontology bootstrap | This doc + `analytes.seed.json` produced. |
| 1 | Skeleton: Tauri + Svelte + Tailwind + theming | App boots, theme toggle works, sample chart renders. |
| 2 | DB + crypto: SQLCipher + password unlock | First-run sets password; relock works; data persists. |
| 3 | PDF tier 1: pdfium extraction + ingest command | Drag-drop PDF → raw_text + reports row. |
| 4 | Parser methods 1–60 (layout, sections, headers, ranges, basic rows) | All 14 sample PDFs produce rows; doc confidence reported. |
| 5 | Parser methods 61–100 (values, units, canonicalization, audit) | All sample analytes mapped to ontology; audit page renders. |
| 6 | Dashboard + analyte detail + ref bands + deltas | All 14 PDFs visualized end-to-end. |
| 7 | Tesseract OCR tier (always-on path for image PDFs) | A scanned-PDF fixture parses via Tesseract. |
| 8 | Passkey support (WebAuthn PRF + dual-wrap) | Register passkey → relock → unlock with passkey only. |
| 9 | Phi-4 LLM repair tier (opt-in) | Toggle on → low-confidence row gets LLM-repaired and shown in audit. |
| 10 | olmOCR-2 vision tier (opt-in) | Toggle on → image-only PDF parsed via olmOCR. |
| 11 | Compare view, export, settings polish | Feature-complete v1. |
| 12 | Bundle + installer + signing | Shippable. |

---

## 16. Open Questions Tracked

- olmOCR-2 native vs sidecar — deferred until Phase 10; tested empirically.
- WebAuthn PRF support detection on the user's exact WebView2 build — confirmed at Phase 8.
- LOINC mapping inclusion in v1 — currently behind a feature flag, no deps added until needed.

---

*End of plan. The ontology bootstrap agent runs concurrently with scaffold work in Phase 1.*
