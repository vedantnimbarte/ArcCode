//! arccode provider implementations.
//!
//! Each provider translates the provider-agnostic `arccode_core::Message` /
//! `CompletionRequest` shape into its native wire format and back. The
//! Anthropic implementation is the reference — it exercises every feature
//! (streaming, tool use, explicit prompt caching).

pub mod anthropic;

pub use anthropic::AnthropicProvider;
