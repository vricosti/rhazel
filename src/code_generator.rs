// SPDX-FileCopyrightText: Copyright 2023 yuzu Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

//! AArch64 code generator — Rust port of `oaknut::CodeGenerator`.
//!
//! Covers exactly the 21 instructions used by the NCE patcher:
//!   STR, LDR, STP, LDP, MRS, MSR, MOV, ADD, ORR,
//!   BL, B, RET, CBZ, CBNZ, LDAXR, STLXR, STXR, STLR, CLREX, UMULH, MADD
//! plus data directives dw/dx and label binding.
//!
//! Encoding references: Arm Architecture Reference Manual (ARMv8-A), section C4.

use crate::label::{FixupKind, Label};
use crate::regs::{QReg, SystemReg, WReg, XReg};

// ── Addressing mode sentinels (match oaknut API) ─────────────────────────────

/// Pre-indexed addressing: `[Xn, #imm]!`
pub const PRE_INDEXED: IndexMode = IndexMode::Pre;
/// Post-indexed addressing: `[Xn], #imm`
pub const POST_INDEXED: IndexMode = IndexMode::Post;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexMode {
    Pre,
    Post,
    /// Signed offset (default, used for two-argument forms).
    Offset,
}

// ── CodeGenerator ─────────────────────────────────────────────────────────────

/// AArch64 code generator that appends u32 words to an output buffer.
///
/// Mirrors `oaknut::CodeGenerator` constructor that takes `std::vector<u32>&`.
pub struct CodeGenerator<'a> {
    buf: &'a mut Vec<u32>,
}

impl<'a> CodeGenerator<'a> {
    /// Create a generator writing into `buf`.
    pub fn new(buf: &'a mut Vec<u32>) -> Self {
        Self { buf }
    }

    /// Current byte offset from the start of the buffer.
    pub fn offset(&self) -> usize {
        self.buf.len() * 4
    }

    // ── Raw data directives ───────────────────────────────────────────────────

    /// Emit a raw 32-bit word (matches `c.dw(v)`).
    pub fn dw(&mut self, v: u32) {
        self.buf.push(v);
    }

    /// Emit a raw 64-bit value as two little-endian words (matches `c.dx(v)`).
    pub fn dx(&mut self, v: u64) {
        self.buf.push(v as u32);
        self.buf.push((v >> 32) as u32);
    }

    // ── Label management ─────────────────────────────────────────────────────

    /// Bind `label` to the current offset and apply any pending fixups.
    ///
    /// Matches `c.l(label)`.
    pub fn l(&mut self, label: &Label) {
        let here = self.offset();
        let fixups = label.bind(here);
        for (fix_offset, kind) in fixups {
            self.apply_fixup(fix_offset, here, kind);
        }
    }

    fn apply_fixup(&mut self, fix_offset: usize, target_offset: usize, kind: FixupKind) {
        let word_idx = fix_offset / 4;
        match kind {
            FixupKind::Branch26 => {
                let delta = (target_offset as i64 - fix_offset as i64) / 4;
                assert!(
                    (-1 << 25..1 << 25).contains(&delta),
                    "branch target out of range: delta={delta}"
                );
                let old = self.buf[word_idx];
                self.buf[word_idx] = (old & !0x03FF_FFFF) | ((delta as u32) & 0x03FF_FFFF);
            }
            FixupKind::Imm19At5 => {
                let delta = (target_offset as i64 - fix_offset as i64) / 4;
                assert!(
                    (-1 << 18..1 << 18).contains(&delta),
                    "CBZ/CBNZ/LDR target out of range: delta={delta}"
                );
                let old = self.buf[word_idx];
                self.buf[word_idx] =
                    (old & !(0x7FFFF << 5)) | (((delta as u32) & 0x7FFFF) << 5);
            }
            FixupKind::Data64 => {
                let lo = target_offset as u32;
                let hi = (target_offset >> 32) as u32;
                self.buf[word_idx] = lo;
                if word_idx + 1 < self.buf.len() {
                    self.buf[word_idx + 1] = hi;
                }
            }
        }
    }

    // ── B  (unconditional branch) ─────────────────────────────────────────────

    /// Branch to an immediate byte offset (signed, scaled by 4).
    ///
    /// `c.B(offset_bytes)` — matches `rc.B(rel.patch_offset - …)`.
    pub fn b_offset(&mut self, offset_bytes: i64) {
        let delta = offset_bytes / 4;
        assert!(
            (-1 << 25..1 << 25).contains(&delta),
            "B offset out of range: {offset_bytes}"
        );
        self.buf.push(0x1400_0000 | ((delta as u32) & 0x03FF_FFFF));
    }

    /// Branch to a label (forward or backward).
    pub fn b(&mut self, label: &Label) {
        let here = self.offset();
        if let Some(target) = label.offset() {
            let delta = (target as i64 - here as i64) / 4;
            self.buf.push(0x1400_0000 | ((delta as u32) & 0x03FF_FFFF));
        } else {
            // Forward reference — emit placeholder and register fixup.
            label.add_fixup(here, FixupKind::Branch26);
            self.buf.push(0x1400_0000);
        }
    }

    // ── BL (branch with link) ─────────────────────────────────────────────────

    /// Branch with link to a label (`c.BL(label)`).
    pub fn bl(&mut self, label: &Label) {
        let here = self.offset();
        if let Some(target) = label.offset() {
            let delta = (target as i64 - here as i64) / 4;
            self.buf.push(0x9400_0000 | ((delta as u32) & 0x03FF_FFFF));
        } else {
            label.add_fixup(here, FixupKind::Branch26);
            self.buf.push(0x9400_0000);
        }
    }

    // ── RET ───────────────────────────────────────────────────────────────────

    /// Return via X30 (`c.RET()`).
    pub fn ret(&mut self) {
        // RET = 0xD65F_03C0 (Rn = X30)
        self.buf.push(0xD65F_03C0);
    }

    // ── CBZ / CBNZ ───────────────────────────────────────────────────────────

    /// Compare and branch if zero — 64-bit (`c.CBZ(Xn, label)`).
    pub fn cbz_x(&mut self, rn: XReg, label: &Label) {
        self.emit_cbz_cbnz(1, 0, rn.index() as u32, label);
    }

    /// Compare and branch if not-zero — 32-bit (`c.CBNZ(Wn, label)`).
    pub fn cbnz_w(&mut self, rn: WReg, label: &Label) {
        self.emit_cbz_cbnz(0, 1, rn.index() as u32, label);
    }

    fn emit_cbz_cbnz(&mut self, sf: u32, op: u32, rn: u32, label: &Label) {
        // sf:op:011010:imm19:Rt  — imm19 at [23:5]
        let here = self.offset();
        let base = (sf << 31) | (op << 24) | 0x3400_0000 | rn;
        if let Some(target) = label.offset() {
            let delta = (target as i64 - here as i64) / 4;
            assert!((-1 << 18..1 << 18).contains(&delta));
            self.buf.push(base | (((delta as u32) & 0x7FFFF) << 5));
        } else {
            label.add_fixup(here, FixupKind::Imm19At5);
            self.buf.push(base);
        }
    }

    // ── MRS / MSR ─────────────────────────────────────────────────────────────

    /// `MRS Xn, <sysreg>` — move from system register to X register.
    pub fn mrs(&mut self, dest: XReg, sysreg: SystemReg) {
        // 1101 0101 0011 1 op1 CRn CRm op2 Rt
        // Fixed: 0xD53B_0000 with sysreg encoding at [20:5], Rt at [4:0]
        let enc = sysreg.encoding();
        self.buf
            .push(0xD530_0000 | (enc << 5) | dest.index() as u32);
    }

    /// `MSR <sysreg>, Xn` — move from X register to system register.
    pub fn msr(&mut self, sysreg: SystemReg, src: XReg) {
        // 1101 0101 0001 1 op1 CRn CRm op2 Rt
        let enc = sysreg.encoding();
        self.buf
            .push(0xD510_0000 | (enc << 5) | src.index() as u32);
    }

    // ── MOV ───────────────────────────────────────────────────────────────────

    /// `MOV SP, Xn` — move X register to SP.
    pub fn mov_sp_from_x(&mut self, src: XReg) {
        // ADD SP, Xn, #0 (canonical encoding)
        self.add_x_imm(XReg(31), src, 0);
    }

    /// `MOV Wn, #imm16` — move immediate to W register (MOVZ, no shift).
    pub fn mov_w_imm(&mut self, rd: WReg, imm: u16) {
        // MOVZ: sf=0, opc=10, hw=00
        // 0 10 100101 00 <imm16> Rd
        self.buf
            .push(0x5280_0000 | ((imm as u32) << 5) | rd.index() as u32);
    }

    /// `MOV Xn, #imm16` — MOVZ 64-bit with zero shift.
    pub fn mov_x_imm(&mut self, rd: XReg, imm: u64) {
        assert!(imm <= 0xFFFF, "use MOVZ+MOVK for wider immediates");
        // MOVZ sf=1: 1 10 100101 00 <imm16> Rd
        self.buf
            .push(0xD280_0000 | ((imm as u32) << 5) | rd.index() as u32);
    }

    // ── ADD ───────────────────────────────────────────────────────────────────

    /// `ADD Xd, Xn, #imm12` (LSL #0).
    pub fn add_x_imm(&mut self, rd: XReg, rn: XReg, imm: u32) {
        assert!(imm < 4096, "ADD immediate out of 12-bit range");
        // sf=1, op=0, S=0 → 0x9100_0000
        // [21:10] = imm12, [9:5] = Rn, [4:0] = Rd
        self.buf.push(
            0x9100_0000 | (imm << 10) | ((rn.index() as u32) << 5) | rd.index() as u32,
        );
    }

    // ── ORR ───────────────────────────────────────────────────────────────────

    /// `ORR Xd, Xn, #imm` — ORR with bitmask immediate.
    ///
    /// `imm` is the 64-bit bitmask value (must be a valid AArch64 bitmask immediate).
    pub fn orr_x_imm(&mut self, rd: XReg, rn: XReg, imm: u64) {
        let enc = encode_bitmask_immediate(imm, true)
            .expect("orr_x_imm: value is not a valid bitmask immediate");
        // sf=1, opc=01, N:immr:imms at [22:10]
        // 1 01 100100 N immr imms Rn Rd
        self.buf.push(
            0xB200_0000
                | (enc << 10)
                | ((rn.index() as u32) << 5)
                | rd.index() as u32,
        );
    }

    // ── STR / LDR (single register, base + offset) ───────────────────────────

    /// `STR Xn, [Xbase, #imm]` — unsigned offset, scaled by 8.
    pub fn str_x_off(&mut self, rt: XReg, rbase: XReg, imm_bytes: u32) {
        assert!(imm_bytes % 8 == 0 && imm_bytes / 8 < 4096);
        let pimm = imm_bytes / 8;
        // STR Xt, [Xn, #pimm]  — size=11, V=0, opc=00
        // 1 11 111 00 00 <pimm12> Rn Rt
        self.buf.push(
            0xF900_0000 | (pimm << 10) | ((rbase.index() as u32) << 5) | rt.index() as u32,
        );
    }

    /// `STR Wn, [Xbase, #imm]` — unsigned offset, scaled by 4.
    pub fn str_w_off(&mut self, rt: WReg, rbase: XReg, imm_bytes: u32) {
        assert!(imm_bytes % 4 == 0 && imm_bytes / 4 < 4096);
        let pimm = imm_bytes / 4;
        // STR Wt, [Xn, #pimm]  — size=10, V=0, opc=00
        // 1 01 111 00 00 <pimm12> Rn Rt
        self.buf.push(
            0xB900_0000 | (pimm << 10) | ((rbase.index() as u32) << 5) | rt.index() as u32,
        );
    }

    /// `STR Xn, [SP, #simm9]!` (pre-indexed) or `STR Xn, [SP], #simm9` (post-indexed).
    pub fn str_x_indexed(&mut self, rt: XReg, rbase: XReg, simm: i16, mode: IndexMode) {
        let simm9 = (simm as u32) & 0x1FF;
        let (wb, post) = match mode {
            IndexMode::Pre => (0b11u32, 0u32),
            IndexMode::Post => (0b01u32, 0u32),
            IndexMode::Offset => unreachable!("use str_x_off for plain offset"),
        };
        // STR Xt pre/post: size=11, V=0, opc=00, [20:12]=simm9, [11:10]=mode
        // 1 11 111 00 00 0 <simm9> <01|11> Rn Rt
        let mode_bits = if mode == IndexMode::Pre { 0b11 } else { 0b01 };
        let _ = (wb, post);
        self.buf.push(
            0xF800_0000
                | (simm9 << 12)
                | (mode_bits << 10)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `LDR Xn, [Xbase, #imm]` — unsigned offset, scaled by 8.
    pub fn ldr_x_off(&mut self, rt: XReg, rbase: XReg, imm_bytes: u32) {
        assert!(imm_bytes % 8 == 0 && imm_bytes / 8 < 4096);
        let pimm = imm_bytes / 8;
        // LDR Xt, [Xn, #pimm]  — size=11, V=0, opc=01
        self.buf.push(
            0xF940_0000 | (pimm << 10) | ((rbase.index() as u32) << 5) | rt.index() as u32,
        );
    }

    /// `LDR Wn, [Xbase, #imm]` — unsigned offset, scaled by 4.
    pub fn ldr_w_off(&mut self, rt: WReg, rbase: XReg, imm_bytes: u32) {
        assert!(imm_bytes % 4 == 0 && imm_bytes / 4 < 4096);
        let pimm = imm_bytes / 4;
        // LDR Wt, [Xn, #pimm]  — size=10, V=0, opc=01
        self.buf.push(
            0xB940_0000 | (pimm << 10) | ((rbase.index() as u32) << 5) | rt.index() as u32,
        );
    }

    /// `LDR Xn, [SP, #simm9]` post-indexed or pre-indexed.
    pub fn ldr_x_indexed(&mut self, rt: XReg, rbase: XReg, simm: i16, mode: IndexMode) {
        let simm9 = (simm as u32) & 0x1FF;
        let mode_bits: u32 = match mode {
            IndexMode::Pre => 0b11,
            IndexMode::Post => 0b01,
            IndexMode::Offset => unreachable!(),
        };
        // LDR Xt pre/post: size=11, V=0, opc=01
        // 1 11 111 00 01 0 <simm9> <mode> Rn Rt
        self.buf.push(
            0xF840_0000
                | (simm9 << 12)
                | (mode_bits << 10)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `LDR Xn, label` — PC-relative load (literal).
    pub fn ldr_x_literal(&mut self, rt: XReg, label: &Label) {
        // LDR Xt, <literal>: opc=01, V=0, imm19 at [23:5]
        let here = self.offset();
        if let Some(target) = label.offset() {
            let delta = (target as i64 - here as i64) / 4;
            assert!((-1 << 18..1 << 18).contains(&delta));
            self.buf.push(
                0x5800_0000 | (((delta as u32) & 0x7FFFF) << 5) | rt.index() as u32,
            );
        } else {
            label.add_fixup(here, FixupKind::Imm19At5);
            self.buf.push(0x5800_0000 | rt.index() as u32);
        }
    }

    // ── STP / LDP (register pair) ─────────────────────────────────────────────

    /// `STP Xm, Xn, [Xbase, #imm7*8]` — signed offset pair, scaled by 8.
    pub fn stp_x_off(&mut self, rt1: XReg, rt2: XReg, rbase: XReg, imm_bytes: i32) {
        assert!(imm_bytes % 8 == 0);
        let simm7 = (imm_bytes / 8) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        // STP Xt1, Xt2, [Xn, #imm] — sf=1, opc=10, L=0
        // 1 0 101 0010 <imm7> Rt2 Rn Rt1
        self.buf.push(
            0xA900_0000
                | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `STP Xm, Xn, [SP, #simm7*8]!` or `[SP], #simm7*8`.
    pub fn stp_x_indexed(&mut self, rt1: XReg, rt2: XReg, rbase: XReg, imm_bytes: i32, mode: IndexMode) {
        assert!(imm_bytes % 8 == 0);
        let simm7 = (imm_bytes / 8) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        let opc: u32 = match mode {
            IndexMode::Pre => 0xA980_0000,   // STP pre-indexed
            IndexMode::Post => 0xA880_0000,  // STP post-indexed
            IndexMode::Offset => 0xA900_0000,
        };
        self.buf.push(
            opc | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `STP Qm, Qn, [Xbase, #imm7*16]` — SIMD pair, signed offset.
    pub fn stp_q_off(&mut self, rt1: QReg, rt2: QReg, rbase: XReg, imm_bytes: i32) {
        assert!(imm_bytes % 16 == 0);
        let simm7 = (imm_bytes / 16) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        // STP Qt1, Qt2, [Xn, #imm] — opc=10, V=1, L=0
        // 10 101 1010 <imm7> Rt2 Rn Rt1
        self.buf.push(
            0xAD00_0000
                | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `STP Qm, Qn, [SP, #imm]!` or `[SP], #imm`.
    pub fn stp_q_indexed(&mut self, rt1: QReg, rt2: QReg, rbase: XReg, imm_bytes: i32, mode: IndexMode) {
        assert!(imm_bytes % 16 == 0);
        let simm7 = (imm_bytes / 16) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        let opc: u32 = match mode {
            IndexMode::Pre => 0xAD80_0000,
            IndexMode::Post => 0xAC80_0000,
            IndexMode::Offset => 0xAD00_0000,
        };
        self.buf.push(
            opc | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `LDP Xm, Xn, [Xbase, #imm7*8]` — signed offset pair.
    pub fn ldp_x_off(&mut self, rt1: XReg, rt2: XReg, rbase: XReg, imm_bytes: i32) {
        assert!(imm_bytes % 8 == 0);
        let simm7 = (imm_bytes / 8) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        // LDP Xt1, Xt2, [Xn, #imm] — sf=1, opc=10, L=1
        self.buf.push(
            0xA940_0000
                | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `LDP Xm, Xn, [SP, #simm7*8]!` or `[SP], #simm7*8`.
    pub fn ldp_x_indexed(&mut self, rt1: XReg, rt2: XReg, rbase: XReg, imm_bytes: i32, mode: IndexMode) {
        assert!(imm_bytes % 8 == 0);
        let simm7 = (imm_bytes / 8) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        let opc: u32 = match mode {
            IndexMode::Pre => 0xA9C0_0000,
            IndexMode::Post => 0xA8C0_0000,
            IndexMode::Offset => 0xA940_0000,
        };
        self.buf.push(
            opc | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `LDP Qm, Qn, [Xbase, #imm7*16]`.
    pub fn ldp_q_off(&mut self, rt1: QReg, rt2: QReg, rbase: XReg, imm_bytes: i32) {
        assert!(imm_bytes % 16 == 0);
        let simm7 = (imm_bytes / 16) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        // LDP Qt1, Qt2, [Xn, #imm] — opc=10, V=1, L=1
        self.buf.push(
            0xAD40_0000
                | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    /// `LDP Qm, Qn, [SP, #imm]!` or `[SP], #imm`.
    pub fn ldp_q_indexed(&mut self, rt1: QReg, rt2: QReg, rbase: XReg, imm_bytes: i32, mode: IndexMode) {
        assert!(imm_bytes % 16 == 0);
        let simm7 = (imm_bytes / 16) as i32;
        assert!((-64..64).contains(&simm7));
        let simm7u = (simm7 as u32) & 0x7F;
        let opc: u32 = match mode {
            IndexMode::Pre => 0xADC0_0000,
            IndexMode::Post => 0xACC0_0000,
            IndexMode::Offset => 0xAD40_0000,
        };
        self.buf.push(
            opc | (simm7u << 15)
                | ((rt2.index() as u32) << 10)
                | ((rbase.index() as u32) << 5)
                | rt1.index() as u32,
        );
    }

    // ── Exclusive memory operations ───────────────────────────────────────────

    /// `LDAXR Xn, [Xbase]` — load-acquire exclusive, 64-bit.
    pub fn ldaxr_x(&mut self, rt: XReg, rbase: XReg) {
        // size=11, L=1, o0=1: 0xC85F_FC00 | (Rn<<5) | Rt
        self.buf
            .push(0xC85F_FC00 | ((rbase.index() as u32) << 5) | rt.index() as u32);
    }

    /// `LDAXR Wn, [Xbase]` — load-acquire exclusive, 32-bit.
    pub fn ldaxr_w(&mut self, rt: WReg, rbase: XReg) {
        // size=10, L=1, o0=1
        self.buf
            .push(0x885F_FC00 | ((rbase.index() as u32) << 5) | rt.index() as u32);
    }

    /// `STLXR Ws, Xn, [Xbase]` — store-release exclusive, 64-bit.
    pub fn stlxr_x(&mut self, rs: WReg, rt: XReg, rbase: XReg) {
        // size=11, L=0, o0=1: 0xC800_FC00 | (Rs<<16) | (Rn<<5) | Rt
        self.buf.push(
            0xC800_FC00
                | ((rs.index() as u32) << 16)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `STLXR Ws, Wn, [Xbase]` — store-release exclusive, 32-bit.
    pub fn stlxr_w(&mut self, rs: WReg, rt: WReg, rbase: XReg) {
        // size=10, L=0, o0=1
        self.buf.push(
            0x8800_FC00
                | ((rs.index() as u32) << 16)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `STXR Ws, Xn, [Xbase]` — store exclusive, 64-bit.
    pub fn stxr_x(&mut self, rs: WReg, rt: XReg, rbase: XReg) {
        // size=11, L=0, o0=0: 0xC800_7C00
        self.buf.push(
            0xC800_7C00
                | ((rs.index() as u32) << 16)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `STXR Ws, Wn, [Xbase]` — store exclusive, 32-bit.
    pub fn stxr_w(&mut self, rs: WReg, rt: WReg, rbase: XReg) {
        // size=10
        self.buf.push(
            0x8800_7C00
                | ((rs.index() as u32) << 16)
                | ((rbase.index() as u32) << 5)
                | rt.index() as u32,
        );
    }

    /// `STLR Wn, [Xbase]` — store-release, 32-bit.
    pub fn stlr_w(&mut self, rt: WReg, rbase: XReg) {
        // size=10, L=0, o0=1, Rs=XZR, Rt2=XZR
        // 0x8900_FC1F | (Rn<<5) | Rt
        self.buf
            .push(0x8900_FC1F | ((rbase.index() as u32) << 5) | rt.index() as u32);
    }

    /// `CLREX` — clear exclusive monitor.
    pub fn clrex(&mut self) {
        self.buf.push(0xD503_305F);
    }

    // ── Multiply ──────────────────────────────────────────────────────────────

    /// `UMULH Xd, Xn, Xm` — unsigned multiply high.
    pub fn umulh(&mut self, rd: XReg, rn: XReg, rm: XReg) {
        // 1001 1011 1100 0000 0111 11 Rm Rn Rd
        // 0x9BC0_7C00 | (Rm<<16) | (Rn<<5) | Rd
        self.buf.push(
            0x9BC0_7C00
                | ((rm.index() as u32) << 16)
                | ((rn.index() as u32) << 5)
                | rd.index() as u32,
        );
    }

    /// `MADD Xd, Xn, Xm, Xa` — multiply-add: `Xd = Xn * Xm + Xa`.
    pub fn madd(&mut self, rd: XReg, rn: XReg, rm: XReg, ra: XReg) {
        // 1001 1011 000 Rm 0 Ra Rn Rd
        // 0x9B00_0000 | (Rm<<16) | (Ra<<10) | (Rn<<5) | Rd
        self.buf.push(
            0x9B00_0000
                | ((rm.index() as u32) << 16)
                | ((ra.index() as u32) << 10)
                | ((rn.index() as u32) << 5)
                | rd.index() as u32,
        );
    }
}

// ── Bitmask immediate encoder ─────────────────────────────────────────────────

/// Encode a 64-bit value as an AArch64 bitmask immediate (N:immr:imms, 13 bits).
/// Returns None if the value cannot be represented.
fn encode_bitmask_immediate(value: u64, sf: bool) -> Option<u32> {
    if value == 0 || (sf && value == u64::MAX) || (!sf && value == 0xFFFF_FFFF) {
        return None;
    }

    let size = if sf { 64usize } else { 32 };

    // Find element size: smallest power-of-two len such that the pattern repeats.
    let mut len = 64;
    while len > 1 {
        let half = len / 2;
        let mask = (1u64 << half) - 1;
        let lo = value & mask;
        let hi = (value >> half) & mask;
        if lo != hi {
            break;
        }
        len = half;
    }

    if len > size {
        return None;
    }

    let mask = if len == 64 { u64::MAX } else { (1u64 << len) - 1 };
    let element = value & mask;

    // Count trailing zeros and trailing ones.
    let tz = element.trailing_zeros() as usize;
    let element_rotated = if tz > 0 { element.rotate_right(tz as u32) } else { element };
    let ones = element_rotated.trailing_ones() as usize;

    // Check that the element is (ones 1-bits) rotated left by tz.
    let expected = if ones == 64 {
        u64::MAX
    } else {
        ((1u64 << ones) - 1).rotate_left((len - tz) as u32)
    } & mask;
    if element != expected {
        return None;
    }

    let n = if len == 64 { 1u32 } else { 0u32 };
    let immr = ((len - tz) & (len - 1)) as u32;
    let imms = (ones - 1) as u32 | (!((len << 1) - 1) as u32 & 0x3F);

    Some((n << 12) | (immr << 6) | imms)
}
