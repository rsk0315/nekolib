//! ポインタ系データ構造。
//!
//! ## Contents
//!
//! 説明書き：
//!
//! - [生ポインタ](rawptr/index.html)
//! - [variance](variance/index.html)
//! - TODO: [Stacked Borrows](sb/index.html)
//!
//! サンプル：
//!
//! - [handle](sample_handle/index.html)
//! - TODO: [node-ref](sample_noderef/index.html)

pub mod maybe_uninit;
pub mod rawptr;
pub mod sb;
pub mod variance;

pub mod sample_handle;
pub mod sample_noderef;

pub mod draft;
