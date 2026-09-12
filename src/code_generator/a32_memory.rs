//! Typed memory operand overloads used by Dynarmic's A32 and memory emitters.
//!
//! Upstream: oaknut/impl/mnemonics_generic_v8.0.inc.hpp and
//! oaknut/impl/mnemonics_fpsimd_v8.0.inc.hpp. Register-offset overloads here
//! use an unscaled byte offset: X registers select LSL #0, W registers
//! select UXTW #0. The offset type carries the extension, not a raw flag.

use super::CodeGenerator;
use crate::inst;
use crate::reg::{GpReg, QReg, WReg, XReg, XRegSp};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockOfCode, Q7, SP, W7, W9, X7, X8, X9, XZR};

    fn words(block: &BlockOfCode) -> Vec<u32> {
        (0..block.code_size())
            .step_by(4)
            .map(|offset| unsafe {
                block
                    .code_base_ptr()
                    .add(offset)
                    .cast::<u32>()
                    .read_unaligned()
            })
            .collect()
    }

    #[test]
    fn register_offset_types_select_oaknut_extension_bits_without_scaling() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.ldr_reg(W7, X8, X9).unwrap();
        code.ldr_reg(W7, X8, W9).unwrap();
        code.ldr_reg(X7, X8, X9).unwrap();
        code.ldr_reg(X7, X8, W9).unwrap();
        code.ldr_q_reg(Q7, X8, X9).unwrap();
        code.ldr_q_reg(Q7, X8, W9).unwrap();
        code.str_reg(W7, X8, X9).unwrap();
        code.str_reg(W7, X8, W9).unwrap();
        code.str_reg(X7, X8, X9).unwrap();
        code.str_reg(X7, X8, W9).unwrap();
        code.str_q_reg(Q7, X8, X9).unwrap();
        code.str_q_reg(Q7, X8, W9).unwrap();
        // Oaknut's register-offset templates with S=0, option=011/010.
        assert_eq!(
            words(&block),
            [
                0xb869_6907,
                0xb869_4907,
                0xf869_6907,
                0xf869_4907,
                0x3ce9_6907,
                0x3ce9_4907,
                0xb829_6907,
                0xb829_4907,
                0xf829_6907,
                0xf829_4907,
                0x3ca9_6907,
                0x3ca9_4907,
            ]
        );
    }

    #[test]
    fn narrow_memory_and_acquire_release_match_oaknut_templates() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.ldrb_reg(W7, X8, X9).unwrap();
        code.ldrb_reg(W7, X8, W9).unwrap();
        code.ldrh_reg(W7, X8, X9).unwrap();
        code.ldrh_reg(W7, X8, W9).unwrap();
        code.strb_reg(W7, X8, X9).unwrap();
        code.strb_reg(W7, X8, W9).unwrap();
        code.strh_reg(W7, X8, X9).unwrap();
        code.strh_reg(W7, X8, W9).unwrap();
        code.ldarb(W7, X8).unwrap();
        code.ldarh(W7, X8).unwrap();
        code.stlrb(W7, X8).unwrap();
        code.stlrh(W7, X8).unwrap();
        code.stlr(W7, X8).unwrap();
        code.stlr(X7, X8).unwrap();
        assert_eq!(
            words(&block),
            [
                0x3869_6907,
                0x3869_4907,
                0x7869_6907,
                0x7869_4907,
                0x3829_6907,
                0x3829_4907,
                0x7829_6907,
                0x7829_4907,
                0x08df_fd07,
                0x48df_fd07,
                0x089f_fd07,
                0x489f_fd07,
                0x889f_fd07,
                0xc89f_fd07,
            ]
        );
    }

    #[test]
    fn unscaled_offsets_and_sp_keep_a32_pc_upper_descriptor_layout() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.stur(X7, X8, 60).unwrap();
        code.ldur(X7, X8, 60).unwrap();
        code.stur(XZR, SP, -256).unwrap();
        code.ldur(X7, SP, 255).unwrap();
        code.add_uxtw(X7, SP, W9).unwrap();
        assert_eq!(
            words(&block),
            [
                0xf803_c107,
                0xf843_c107,
                0xf810_03ff,
                0xf84f_f3e7,
                0x8b29_43e7,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "out of imm9 range")]
    fn unscaled_offset_rejects_value_above_signed_imm9_limit() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        CodeGenerator::new(&mut block).stur(X7, X8, 256).unwrap();
    }
}

impl CodeGenerator<'_> {
    /// `LDARB(Wt, [Xn|SP])`.
    pub fn ldarb(&mut self, rt: WReg, rn: impl Into<XRegSp>) -> Result<(), String> {
        self.emit(inst::ldarb_w(rt.index(), rn.into().index()))
    }

    /// `LDARH(Wt, [Xn|SP])`.
    pub fn ldarh(&mut self, rt: WReg, rn: impl Into<XRegSp>) -> Result<(), String> {
        self.emit(inst::ldarh_w(rt.index(), rn.into().index()))
    }

    /// `STLRB(Wt, [Xn|SP])`.
    pub fn stlrb(&mut self, rt: WReg, rn: impl Into<XRegSp>) -> Result<(), String> {
        self.emit(inst::stlrb_w(rt.index(), rn.into().index()))
    }

    /// `STLRH(Wt, [Xn|SP])`.
    pub fn stlrh(&mut self, rt: WReg, rn: impl Into<XRegSp>) -> Result<(), String> {
        self.emit(inst::stlrh_w(rt.index(), rn.into().index()))
    }

    /// `STLR(Rt, [Xn|SP])`: release store, with width selected by Rt.
    pub fn stlr<R: GpReg>(&mut self, rt: R, rn: impl Into<XRegSp>) -> Result<(), String> {
        let rn = rn.into().index();
        self.emit(if R::SF {
            inst::stlr_x(rt.index(), rn)
        } else {
            inst::stlr_w(rt.index(), rn)
        })
    }

    /// `ADD(Xd|SP, Xn|SP, Wm, UXTW #0)`.
    pub fn add_uxtw(
        &mut self,
        rd: impl Into<XRegSp>,
        rn: impl Into<XRegSp>,
        rm: WReg,
    ) -> Result<(), String> {
        self.emit(inst::add_x_reg_uxtw(
            rd.into().index(),
            rn.into().index(),
            rm.index(),
        ))
    }

    /// `LDUR(Xt, [Xn|SP, #simm9])`, including unaligned byte offsets.
    pub fn ldur(&mut self, rt: XReg, rn: impl Into<XRegSp>, imm_bytes: i32) -> Result<(), String> {
        self.emit(inst::ldur_x(rt.index(), rn.into().index(), imm_bytes))
    }

    /// `STUR(Xt, [Xn|SP, #simm9])`, including unaligned byte offsets.
    pub fn stur(&mut self, rt: XReg, rn: impl Into<XRegSp>, imm_bytes: i32) -> Result<(), String> {
        self.emit(inst::stur_x(rt.index(), rn.into().index(), imm_bytes))
    }

    /// `LDR(Rt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn ldr_reg<R: GpReg, O: GpReg>(
        &mut self,
        rt: R,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(match (R::SF, O::SF) {
            (false, false) => inst::ldr_w_reg_uxtw(rt, rn, rm),
            (false, true) => inst::ldr_w_reg_lsl(rt, rn, rm),
            (true, false) => inst::ldr_x_reg_uxtw(rt, rn, rm),
            (true, true) => inst::ldr_x_reg_lsl(rt, rn, rm),
        })
    }

    /// SIMD `LDR(Qt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn ldr_q_reg<O: GpReg>(
        &mut self,
        rt: QReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::ldr_q_reg_lsl(rt, rn, rm)
        } else {
            inst::ldr_q_reg_uxtw(rt, rn, rm)
        })
    }

    /// `STR(Rt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn str_reg<R: GpReg, O: GpReg>(
        &mut self,
        rt: R,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(match (R::SF, O::SF) {
            (false, false) => inst::str_w_reg_uxtw(rt, rn, rm),
            (false, true) => inst::str_w_reg_lsl(rt, rn, rm),
            (true, false) => inst::str_x_reg_uxtw(rt, rn, rm),
            (true, true) => inst::str_x_reg_lsl(rt, rn, rm),
        })
    }

    /// SIMD `STR(Qt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn str_q_reg<O: GpReg>(
        &mut self,
        rt: QReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::str_q_reg_lsl(rt, rn, rm)
        } else {
            inst::str_q_reg_uxtw(rt, rn, rm)
        })
    }

    /// `LDRB(Wt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn ldrb_reg<O: GpReg>(
        &mut self,
        rt: WReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::ldrb_w_reg_lsl(rt, rn, rm)
        } else {
            inst::ldrb_w_reg_uxtw(rt, rn, rm)
        })
    }

    /// `LDRH(Wt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn ldrh_reg<O: GpReg>(
        &mut self,
        rt: WReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::ldrh_w_reg_lsl(rt, rn, rm)
        } else {
            inst::ldrh_w_reg_uxtw(rt, rn, rm)
        })
    }

    /// `STRB(Wt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn strb_reg<O: GpReg>(
        &mut self,
        rt: WReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::strb_w_reg_lsl(rt, rn, rm)
        } else {
            inst::strb_w_reg_uxtw(rt, rn, rm)
        })
    }

    /// `STRH(Wt, [Xn|SP, Rm, LSL|UXTW #0])`.
    pub fn strh_reg<O: GpReg>(
        &mut self,
        rt: WReg,
        rn: impl Into<XRegSp>,
        rm: O,
    ) -> Result<(), String> {
        let (rt, rn, rm) = (rt.index(), rn.into().index(), rm.index());
        self.emit(if O::SF {
            inst::strh_w_reg_lsl(rt, rn, rm)
        } else {
            inst::strh_w_reg_uxtw(rt, rn, rm)
        })
    }
}
