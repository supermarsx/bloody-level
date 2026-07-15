// Tier-3 vision OCR — olmOCR-2. Feature-gated.

pub fn is_available() -> bool {
    cfg!(feature = "embedded-ocr-vision")
}
