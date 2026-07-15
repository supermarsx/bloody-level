// Tier-4 LLM — Phi-4-mini-reasoning via llama.cpp. Feature-gated.

pub fn is_available() -> bool {
    cfg!(feature = "embedded-llm")
}

pub fn is_loaded() -> bool {
    false
}
