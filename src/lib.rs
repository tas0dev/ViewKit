// ViewKit のホスト向け Shim を公開する
// Shared libkagami is hosted under ../Kagami.
#[path = "../../Kagami/src/libkagami.rs"]
pub mod libkagami;
pub mod pipeline;
mod component;

pub use libkagami::*;
