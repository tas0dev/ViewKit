// ViewKit のホスト向け Shim を公開する
// Minimal crate exposing only the libkagami host shim.

pub mod libkagami;
pub mod pipeline;
pub use libkagami::*;
