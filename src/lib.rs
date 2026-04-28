// ViewKit のホスト向け Shim を公開する
// 簡潔に：Wayland ベースの最小 API を提供し、Kagami の surface/buffer/input をホスト上で模倣する

pub mod libkagami;

pub use libkagami::*;
