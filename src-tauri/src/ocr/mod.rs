// Tier-2 OCR — Tesseract (no LLM). Feature-gated.

pub fn is_available() -> bool {
    cfg!(feature = "tesseract-ocr")
}
