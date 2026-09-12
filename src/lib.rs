// SPDX-License-Identifier: GPL-2.0-or-later

//! **rhazel** — AArch64 assembler for ruzu, the counterpart of the `oaknut`
//! library that upstream dynarmic's arm64 backend links.
//!
//! Like oaknut for dynarmic, this crate owns only instruction encoding, code
//! buffers and labels; every emitter that turns IR into these instructions
//! lives in rdynarmic's `backend/arm64`.

pub mod block_of_code;
pub mod cond;
pub mod inst;
pub mod label;

pub use block_of_code::BlockOfCode;
pub use cond::Cond;
pub use label::Label;
