// SPDX-FileCopyrightText: Copyright 2023 yuzu Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

//! Label and fixup mechanism — mirrors oaknut::Label.
//!
//! A Label is a forward/backward reference to a code offset.
//! When a branch is emitted before the label is bound, a fixup entry is
//! recorded.  When `bind` is called the fixup list is resolved.

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
struct LabelInner {
    /// Byte offset from the start of the code buffer, or None if not yet bound.
    offset: Option<usize>,
    /// List of (fixup_byte_offset, fixup_kind) waiting for this label to be bound.
    fixups: Vec<(usize, FixupKind)>,
}

/// Kind of fixup to apply when the label is resolved.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FixupKind {
    /// 26-bit PC-relative branch offset in bits [25:0], scaled by 4 (B / BL).
    Branch26,
    /// 19-bit PC-relative offset in bits [23:5], scaled by 4 (CBZ/CBNZ/LDR literal).
    Imm19At5,
    /// Raw 64-bit little-endian constant (for `dx`). Reserved for future use.
    #[allow(dead_code)]
    Data64,
}

/// A code label — cheap to clone (backed by `Rc<RefCell<>>`).
#[derive(Clone, Debug, Default)]
pub struct Label(Rc<RefCell<LabelInner>>);

impl Label {
    pub fn new() -> Self {
        Label(Rc::new(RefCell::new(LabelInner::default())))
    }

    /// Returns the bound offset, or None if not yet bound.
    pub fn offset(&self) -> Option<usize> {
        self.0.borrow().offset
    }

    /// Bind the label to `offset` and return the pending fixups.
    pub(crate) fn bind(&self, offset: usize) -> Vec<(usize, FixupKind)> {
        let mut inner = self.0.borrow_mut();
        assert!(inner.offset.is_none(), "label bound twice");
        inner.offset = Some(offset);
        std::mem::take(&mut inner.fixups)
    }

    /// Record a pending fixup at `at_offset` of the given kind.
    pub(crate) fn add_fixup(&self, at_offset: usize, kind: FixupKind) {
        let mut inner = self.0.borrow_mut();
        if let Some(bound) = inner.offset {
            // Label already bound — caller must apply immediately.
            // We store it anyway so the caller can drain it right after.
            inner.fixups.push((at_offset, kind));
            let _ = bound;
        } else {
            inner.fixups.push((at_offset, kind));
        }
    }

    /// True if already bound.
    pub fn is_bound(&self) -> bool {
        self.0.borrow().offset.is_some()
    }
}
