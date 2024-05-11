//! ポインタ系データ構造。
//!
//! ## Contents
//!
//! - [生ポインタ](rawptr/index.html)
//! - [未初期化の値](maybe_uninit/index.html)
//! - [variance](variance/index.html)
//! - [エイリアスモデル](alias_model/index.html)

#![allow(dead_code)]

pub mod alias_model;
pub mod maybe_uninit;
pub mod rawptr;
pub mod variance;

mod sample_handle;
mod sample_list;
mod sample_noderef;

mod draft;
