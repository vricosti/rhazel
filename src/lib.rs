// SPDX-FileCopyrightText: Copyright 2023 yuzu Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

//! **rhazel** — mini AArch64 code generator, Rust port of oaknut.
//!
//! Covers exactly the instruction subset used by the NCE patcher:
//! STR, LDR, STP, LDP, MRS, MSR, MOV, ADD, ORR,
//! BL, B, RET, CBZ, CBNZ, LDAXR, STLXR, STXR, STLR, CLREX, UMULH, MADD.

pub mod code_generator;
pub mod label;
pub mod regs;

pub use code_generator::{CodeGenerator, IndexMode, POST_INDEXED, PRE_INDEXED};
pub use label::Label;
pub use regs::{
    QReg, SystemReg, WReg, XReg,
    // X registers
    X0, X1, X2, X3, X4, X5, X6, X7, X8, X9, X10, X11, X12, X13, X14, X15,
    X16, X17, X18, X19, X20, X21, X22, X23, X24, X25, X26, X27, X28, X29,
    X30, XZR, SP,
    // W registers
    W0, W1, W2, W3, WZR,
    // Q registers
    Q8, Q9, Q10, Q11, Q12, Q13, Q14, Q15,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn gen() -> Vec<u32> {
        Vec::new()
    }

    // ── Branch / Label ────────────────────────────────────────────────────────

    #[test]
    fn test_ret() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.ret();
        assert_eq!(buf[0], 0xD65F_03C0);
    }

    #[test]
    fn test_b_backward() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        let lbl = Label::new();
        c.l(&lbl);          // bind at offset 0 (no word emitted)
        c.dw(0xD503_201F); // NOP at buf[0], offset 0
        c.b(&lbl);          // at offset 4, target=0, delta=-1 → buf[1]
        // B encoding: 0x1400_0000 | ((-1i32 as u32) & 0x03FF_FFFF) = 0x17FF_FFFF
        assert_eq!(buf[1], 0x17FF_FFFF);
    }

    #[test]
    fn test_bl_forward() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        let lbl = Label::new();
        c.bl(&lbl);         // at offset 0, target unknown
        c.dw(0);            // padding at offset 4
        c.l(&lbl);          // bind at offset 8 → delta = +2
        // BL 0x9400_0000 | 2
        assert_eq!(buf[0], 0x9400_0002);
    }

    #[test]
    fn test_b_offset_forward() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.b_offset(8); // delta = 8/4 = 2
        assert_eq!(buf[0], 0x1400_0002);
    }

    // ── CBZ / CBNZ ───────────────────────────────────────────────────────────

    #[test]
    fn test_cbnz_w_backward() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        let lbl = Label::new();
        c.l(&lbl);           // bind at offset 0 (no word emitted)
        c.dw(0);             // buf[0] at offset 0
        c.cbnz_w(W1, &lbl); // at offset 4, target=0, delta=-1 → buf[1]
        // CBNZ W1: sf=0, op=1 → base=0x3500_0001, imm19(-1)=0x7FFFF at [23:5]
        let delta: i32 = -1;
        let imm19 = (delta as u32) & 0x7FFFF;
        let expected = 0x3500_0000u32 | (imm19 << 5) | 1;
        assert_eq!(buf[1], expected);
    }

    // ── MRS / MSR ─────────────────────────────────────────────────────────────

    #[test]
    fn test_mrs_tpidr_el0() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.mrs(X30, SystemReg::TpidrEl0);
        // MRS X30, TPIDR_EL0: 0xD53B_D81E
        // enc = 0b11_011_1101_0000_010 = 0x6DA2  (but packed as sysreg field)
        // 0xD530_0000 | (enc << 5) | 30
        let enc = SystemReg::TpidrEl0.encoding();
        let expected = 0xD530_0000 | (enc << 5) | 30;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_msr_tpidr_el0() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.msr(SystemReg::TpidrEl0, X3);
        let enc = SystemReg::TpidrEl0.encoding();
        let expected = 0xD510_0000 | (enc << 5) | 3;
        assert_eq!(buf[0], expected);
    }

    // ── STR / LDR ─────────────────────────────────────────────────────────────

    #[test]
    fn test_str_x_offset() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.str_x_off(X30, X30, 8);
        // STR X30, [X30, #8]: pimm = 1
        // 0xF900_0000 | (1 << 10) | (30 << 5) | 30
        let expected = 0xF900_0000 | (1 << 10) | (30 << 5) | 30;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_str_x_pre_indexed() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.str_x_indexed(X30, SP, -16, PRE_INDEXED);
        // STR X30, [SP, #-16]! : simm9 = -16 = 0x1F0, mode=0b11
        // 0xF800_0000 | (0x1F0 << 12) | (0b11 << 10) | (31 << 5) | 30
        let simm9 = (-16i16 as u32) & 0x1FF;
        let expected = 0xF800_0000 | (simm9 << 12) | (0b11 << 10) | (31 << 5) | 30;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_ldr_x_offset() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.ldr_x_off(X30, X30, 0);
        // LDR X30, [X30, #0]
        let expected = 0xF940_0000 | (0 << 10) | (30 << 5) | 30;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_ldr_x_post_indexed() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.ldr_x_indexed(X30, SP, 16, POST_INDEXED);
        // LDR X30, [SP], #16 : simm9=16=0x010, mode=0b01
        let simm9 = (16i16 as u32) & 0x1FF;
        let expected = 0xF840_0000 | (simm9 << 12) | (0b01 << 10) | (31 << 5) | 30;
        assert_eq!(buf[0], expected);
    }

    // ── STP / LDP ─────────────────────────────────────────────────────────────

    #[test]
    fn test_stp_x_pre_indexed() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.stp_x_indexed(X0, X1, SP, -16, PRE_INDEXED);
        // STP X0, X1, [SP, #-16]! : simm7 = -16/8 = -2 → 0x7E
        let simm7 = ((-2i32) as u32) & 0x7F;
        let expected = 0xA980_0000 | (simm7 << 15) | (1 << 10) | (31 << 5) | 0;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_ldp_x_post_indexed() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.ldp_x_indexed(X0, X1, SP, 16, POST_INDEXED);
        // LDP X0, X1, [SP], #16 : simm7 = 16/8 = 2
        let simm7 = 2u32;
        let expected = 0xA8C0_0000 | (simm7 << 15) | (1 << 10) | (31 << 5) | 0;
        assert_eq!(buf[0], expected);
    }

    // ── MOV / ADD / ORR ───────────────────────────────────────────────────────

    #[test]
    fn test_mov_w_imm() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.mov_w_imm(W1, 42);
        // MOVZ W1, #42: 0x5280_0000 | (42 << 5) | 1
        assert_eq!(buf[0], 0x5280_0000 | (42 << 5) | 1);
    }

    #[test]
    fn test_add_x_imm() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.add_x_imm(X0, X0, 8);
        // ADD X0, X0, #8: 0x9100_0000 | (8 << 10) | (0 << 5) | 0
        assert_eq!(buf[0], 0x9100_0000 | (8 << 10));
    }

    // ── Exclusive / atomic ────────────────────────────────────────────────────

    #[test]
    fn test_ldaxr_x() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.ldaxr_x(X0, X2);
        // LDAXR X0, [X2]: 0xC85F_FC00 | (2 << 5) | 0
        assert_eq!(buf[0], 0xC85F_FC40);
    }

    #[test]
    fn test_stlxr_w_xzr() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.stlxr_x(W3, XZR, X2);
        // STLXR W3, XZR, [X2]: 0xC800_FC00 | (3<<16) | (2<<5) | 31
        let expected = 0xC800_FC00 | (3 << 16) | (2 << 5) | 31;
        assert_eq!(buf[0], expected);
    }

    #[test]
    fn test_clrex() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.clrex();
        assert_eq!(buf[0], 0xD503_305F);
    }

    // ── UMULH / MADD ─────────────────────────────────────────────────────────

    #[test]
    fn test_umulh() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.umulh(X0, X2, X0);
        // UMULH X0, X2, X0: 0x9BC0_7C00 | (0<<16) | (2<<5) | 0
        assert_eq!(buf[0], 0x9BC0_7C40);
    }

    #[test]
    fn test_madd() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.madd(X2, X2, X1, X0);
        // MADD X2, X2, X1, X0: 0x9B00_0000 | (1<<16) | (0<<10) | (2<<5) | 2
        let expected = 0x9B00_0000 | (1 << 16) | (0 << 10) | (2 << 5) | 2;
        assert_eq!(buf[0], expected);
    }

    // ── dw / dx ───────────────────────────────────────────────────────────────

    #[test]
    fn test_dx() {
        let mut buf = gen();
        let mut c = CodeGenerator::new(&mut buf);
        c.dx(0xDEAD_BEEF_CAFE_1234u64);
        assert_eq!(buf[0], 0xCAFE_1234);
        assert_eq!(buf[1], 0xDEAD_BEEF);
    }
}
