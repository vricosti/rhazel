//! Integer mnemonic extensions used by Dynarmic's emit_arm64_data_processing.cpp.
//! Operand forms follow oaknut/impl/mnemonics_generic_v8.0.inc.hpp.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn integer_widths_and_flags_match_oaknut_words() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        // Independent literals from Oaknut v2.0.3 emit templates.
        macro_rules! check {
            ($method:ident($($arg:expr),*), $expected:expr) => {{
                let offset = block.code_size();
                CodeGenerator::new(&mut block).$method($($arg),*).unwrap();
                assert_eq!(block.code_size(), offset + 4);
                let actual = unsafe {
                    (block.code_base_ptr().add(offset) as *const u32).read_unaligned()
                };
                assert_eq!(actual, $expected, stringify!($method($($arg),*)));
            }};
        }
        check!(adc(W29, W30, WZR), 0x1a1f03dd);
        check!(adc(X29, X30, XZR), 0x9a1f03dd);
        check!(adcs(W29, W30, WZR), 0x3a1f03dd);
        check!(adcs(X29, X30, XZR), 0xba1f03dd);
        check!(ands(W29, W30, WZR), 0x6a1f03dd);
        check!(ands(X29, X30, XZR), 0xea1f03dd);
        check!(asrv(W29, W30, WZR), 0x1adf2bdd);
        check!(asrv(X29, X30, XZR), 0x9adf2bdd);
        check!(bic(W29, W30, WZR), 0x0a3f03dd);
        check!(bic(X29, X30, XZR), 0x8a3f03dd);
        check!(bics(W29, W30, WZR), 0x6a3f03dd);
        check!(bics(X29, X30, XZR), 0xea3f03dd);
        check!(clz(W29, W30), 0x5ac013dd);
        check!(clz(X29, X30), 0xdac013dd);
        check!(lslv(W29, W30, WZR), 0x1adf23dd);
        check!(lslv(X29, X30, XZR), 0x9adf23dd);
        check!(lsrv(W29, W30, WZR), 0x1adf27dd);
        check!(lsrv(X29, X30, XZR), 0x9adf27dd);
        check!(mul(W29, W30, WZR), 0x1b1f7fdd);
        check!(mul(X29, X30, XZR), 0x9b1f7fdd);
        check!(mvn(W29, W30), 0x2a3e03fd);
        check!(mvn(X29, X30), 0xaa3e03fd);
        check!(neg(W29, W30), 0x4b1e03fd);
        check!(rev(W29, W30), 0x5ac00bdd);
        check!(rev(X29, X30), 0xdac00fdd);
        check!(rev16(W29, W30), 0x5ac007dd);
        check!(rorv(W29, W30, WZR), 0x1adf2fdd);
        check!(rorv(X29, X30, XZR), 0x9adf2fdd);
        check!(sbc(W29, W30, WZR), 0x5a1f03dd);
        check!(sbc(X29, X30, XZR), 0xda1f03dd);
        check!(sbcs(W29, W30, WZR), 0x7a1f03dd);
        check!(sbcs(X29, X30, XZR), 0xfa1f03dd);
        check!(sdiv(W29, W30, WZR), 0x1adf0fdd);
        check!(sdiv(X29, X30, XZR), 0x9adf0fdd);
        check!(smulh(X29, X30, XZR), 0x9b5f7fdd);
        check!(sxtb(W29, W30), 0x13001fdd);
        check!(sxtb(X29, W30), 0x93401fdd);
        check!(sxth(W29, W30), 0x13003fdd);
        check!(sxth(X29, W30), 0x93403fdd);
        check!(sxtw(X29, W30), 0x93407fdd);
        check!(tst(W30, WZR), 0x6a1f03df);
        check!(tst(X30, XZR), 0xea1f03df);
        check!(udiv(W29, W30, WZR), 0x1adf0bdd);
        check!(udiv(X29, X30, XZR), 0x9adf0bdd);
        check!(umulh(X29, X30, XZR), 0x9bdf7fdd);
        check!(lsr(W29, W30, 31), 0x531f7fdd);
        check!(lsr(X29, X30, 63), 0xd37fffdd);
        check!(ror(W29, W30, 31), 0x139e7fdd);
        check!(ror(X29, X30, 63), 0x93deffdd);
        check!(extr(W29, W30, WZR, 31), 0x139f7fdd);
        check!(extr(X29, X30, XZR, 63), 0x93dfffdd);
        check!(ubfx(W29, W30, 31, 1), 0x531f7fdd);
        check!(ubfx(X29, X30, 63, 1), 0xd37fffdd);
        check!(ubfiz(W29, W30, 29, 1), 0x530303dd);
        check!(ands_imm(W29, W30, 1 << 29), 0x720303dd);
        check!(orr_imm(W29, W30, 1 << 31), 0x320103dd);
        check!(adds_imm_shift(W29, W30, 4095, 12), 0x317fffdd);
        check!(adds_imm_shift(X29, X30, 4095, 12), 0xb17fffdd);
        check!(subs_imm_shift(W29, W30, 4095, 12), 0x717fffdd);
        check!(subs_imm_shift(X29, X30, 4095, 12), 0xf17fffdd);
    }

    #[test]
    fn invalid_bitfields_and_immediates_do_not_emit() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        macro_rules! rejects {
            ($method:ident($($arg:expr),*)) => {
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    CodeGenerator::new(&mut block).$method($($arg),*).unwrap();
                })).is_err());
                assert_eq!(block.code_size(), 0);
            };
        }
        rejects!(ubfx(W0, W1, 31, 2));
        rejects!(ubfx(X0, X1, 0, 0));
        rejects!(ubfiz(W0, W1, 32, 1));
        rejects!(extr(W0, W1, W2, 32));
        rejects!(lsr(X0, X1, 64));
        rejects!(adds_imm_shift(W0, W1, 1, 1));
        rejects!(subs_imm_shift(X0, X1, 4096, 0));
        rejects!(ands_imm(W0, W1, 0));
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn sign_extended_variable_shift_keeps_64_bit_shift_range() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        // Upstream ArithmeticShiftRight32 with carry clamps at 63 and uses
        // X operands here, then narrows the result with MOV W. W ASRV would
        // incorrectly wrap shift amounts 32..63 modulo 32.
        code.sxtw(X0, W0).unwrap();
        code.asrv(X0, X0, X1).unwrap();
        code.mov(W0, W0).unwrap();
        code.ret().unwrap();
        drop(code);
        block.seal();
        let run: unsafe extern "C" fn(u32, u64) -> u32 =
            unsafe { std::mem::transmute(block.code_base_ptr()) };
        for input in [0, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff] {
            for shift in [0, 1, 31, 32, 33, 63] {
                assert_eq!(
                    unsafe { run(input, shift) },
                    ((input as i32 as i64) >> shift) as u32
                );
            }
        }
    }
}

use super::CodeGenerator;
use crate::{inst, GpReg, WReg, XReg};

macro_rules! integer_three {
    ($name:ident, $w:ident, $x:ident) => {
        pub fn $name<R: GpReg>(&mut self, rd: R, rn: R, rm: R) -> Result<(), String> {
            self.emit(if R::SF {
                inst::$x(rd.index(), rn.index(), rm.index())
            } else {
                inst::$w(rd.index(), rn.index(), rm.index())
            })
        }
    };
}

macro_rules! integer_two {
    ($name:ident, $w:ident, $x:ident) => {
        pub fn $name<R: GpReg>(&mut self, rd: R, rn: R) -> Result<(), String> {
            self.emit(if R::SF {
                inst::$x(rd.index(), rn.index())
            } else {
                inst::$w(rd.index(), rn.index())
            })
        }
    };
}

impl CodeGenerator<'_> {
    integer_three!(adc, adc_w, adc_x);
    integer_three!(adcs, adcs_w, adcs_x);
    integer_three!(sbc, sbc_w, sbc_x);
    integer_three!(sbcs, sbcs_w, sbcs_x);
    integer_three!(ands, ands_w_reg, ands_x_reg);
    integer_three!(bic, bic_w, bic_x);
    integer_three!(bics, bics_w, bics_x);
    integer_three!(mul, mul_w, mul_x);
    integer_three!(sdiv, sdiv_w, sdiv_x);
    integer_three!(udiv, udiv_w, udiv_x);
    integer_three!(lslv, lslv_w, lslv_x);
    integer_three!(lsrv, lsrv_w, lsrv_x);
    integer_three!(asrv, asrv_w, asrv_x);
    integer_three!(rorv, rorv_w, rorv_x);
    integer_two!(clz, clz_w, clz_x);
    integer_two!(mvn, mvn_w, mvn_x);
    integer_two!(rev, rev_w, rev_x);

    pub fn lsr<R: GpReg>(&mut self, rd: R, rn: R, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::lsr_x_imm(rd.index(), rn.index(), shift)
        } else {
            inst::lsr_w_imm(rd.index(), rn.index(), shift)
        })
    }

    pub fn ror<R: GpReg>(&mut self, rd: R, rn: R, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::ror_x_imm(rd.index(), rn.index(), shift)
        } else {
            inst::ror_w_imm(rd.index(), rn.index(), shift)
        })
    }

    pub fn adds_imm_shift<R: GpReg>(
        &mut self,
        rd: R,
        rn: R,
        imm12: u32,
        shift: u8,
    ) -> Result<(), String> {
        let shift12 = super::arith_imm_shift12(shift);
        self.emit(if R::SF {
            inst::adds_x_imm_shift(rd.index(), rn.index(), imm12, shift12)
        } else {
            inst::adds_w_imm_shift(rd.index(), rn.index(), imm12, shift12)
        })
    }

    pub fn subs_imm_shift<R: GpReg>(
        &mut self,
        rd: R,
        rn: R,
        imm12: u32,
        shift: u8,
    ) -> Result<(), String> {
        let shift12 = super::arith_imm_shift12(shift);
        self.emit(if R::SF {
            inst::subs_x_imm_shift(rd.index(), rn.index(), imm12, shift12)
        } else {
            inst::subs_w_imm_shift(rd.index(), rn.index(), imm12, shift12)
        })
    }

    pub fn sxtb<R: GpReg>(&mut self, rd: R, rn: WReg) -> Result<(), String> {
        self.emit(if R::SF {
            inst::sxtb_x(rd.index(), rn.index())
        } else {
            inst::sxtb_w(rd.index(), rn.index())
        })
    }

    pub fn sxth<R: GpReg>(&mut self, rd: R, rn: WReg) -> Result<(), String> {
        self.emit(if R::SF {
            inst::sxth_x(rd.index(), rn.index())
        } else {
            inst::sxth_w(rd.index(), rn.index())
        })
    }

    pub fn sxtw(&mut self, rd: XReg, rn: WReg) -> Result<(), String> {
        self.emit(inst::sxtw_x(rd.index(), rn.index()))
    }

    pub fn ubfx<R: GpReg>(&mut self, rd: R, rn: R, lsb: u8, width: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::ubfx_x(rd.index(), rn.index(), lsb, width)
        } else {
            inst::ubfx_w(rd.index(), rn.index(), lsb, width)
        })
    }

    pub fn ubfiz(&mut self, rd: WReg, rn: WReg, lsb: u8, width: u8) -> Result<(), String> {
        self.emit(inst::ubfiz_w(rd.index(), rn.index(), lsb, width))
    }

    pub fn extr<R: GpReg>(&mut self, rd: R, rn: R, rm: R, lsb: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::extr_x(rd.index(), rn.index(), rm.index(), lsb)
        } else {
            inst::extr_w(rd.index(), rn.index(), rm.index(), lsb)
        })
    }

    pub fn tst<R: GpReg>(&mut self, rn: R, rm: R) -> Result<(), String> {
        self.emit(if R::SF {
            inst::tst_x_reg(rn.index(), rm.index())
        } else {
            inst::tst_w_reg(rn.index(), rm.index())
        })
    }

    pub fn smulh(&mut self, rd: XReg, rn: XReg, rm: XReg) -> Result<(), String> {
        self.emit(inst::smulh_x(rd.index(), rn.index(), rm.index()))
    }

    pub fn umulh(&mut self, rd: XReg, rn: XReg, rm: XReg) -> Result<(), String> {
        self.emit(inst::umulh_x(rd.index(), rn.index(), rm.index()))
    }

    pub fn rev16(&mut self, rd: WReg, rn: WReg) -> Result<(), String> {
        self.emit(inst::rev16_w(rd.index(), rn.index()))
    }

    pub fn neg(&mut self, rd: WReg, rn: WReg) -> Result<(), String> {
        self.emit(inst::neg_w(rd.index(), rn.index()))
    }

    pub fn ands_imm(&mut self, rd: WReg, rn: WReg, imm: u32) -> Result<(), String> {
        self.emit(inst::ands_w_imm(rd.index(), rn.index(), imm))
    }

    pub fn orr_imm(&mut self, rd: WReg, rn: WReg, imm: u32) -> Result<(), String> {
        self.emit(inst::orr_w_imm(rd.index(), rn.index(), imm))
    }
}
