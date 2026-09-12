//! `oaknut::CodeGenerator`'s counterpart: one method per mnemonic, emitting
//! into a [`BlockOfCode`].
//!
//! Method names are oaknut's mnemonics in snake_case. Where oaknut overloads
//! one name across operand classes, the Rust method is generic over the
//! operand traits in [`crate::reg`], so `code.adds(w0, w1, w2)` and
//! `code.adds(x0, x1, x2)` read like the C++ while the type still selects
//! the encoding. Overloads that differ in operand *kind* rather than width
//! (register vs immediate) keep a suffix, e.g. `cmp` / `cmp_imm`, and
//! `B(Cond, Label&)` is `b_cond`.
//!
//! The generator dereferences to its `BlockOfCode`, so callers that still
//! emit raw words through `write_u32` or drive a [`Label`] directly keep
//! working while they migrate; the encoders themselves stay in [`crate::inst`].

use std::ops::{Deref, DerefMut};

use crate::block_of_code::BlockOfCode;
use crate::cond::Cond;
use crate::inst;
use crate::label::Label;
use crate::reg::{
    DReg, GpReg, QReg, VReg16B, VReg4S, VRegArranged, WReg, XReg, XRegSp,
};

pub struct CodeGenerator<'a> {
    code: &'a mut BlockOfCode,
}

impl<'a> Deref for CodeGenerator<'a> {
    type Target = BlockOfCode;
    fn deref(&self) -> &BlockOfCode {
        self.code
    }
}

impl<'a> DerefMut for CodeGenerator<'a> {
    fn deref_mut(&mut self) -> &mut BlockOfCode {
        self.code
    }
}

macro_rules! gp_mnemonic3 {
    ($(#[$doc:meta])* $name:ident, $w:ident, $x:ident) => {
        $(#[$doc])*
        pub fn $name<R: GpReg>(&mut self, rd: R, rn: R, rm: R) -> Result<(), String> {
            self.emit(if R::SF {
                inst::$x(rd.index(), rn.index(), rm.index())
            } else {
                inst::$w(rd.index(), rn.index(), rm.index())
            })
        }
    };
}

macro_rules! vec_mnemonic3 {
    ($(#[$doc:meta])* $name:ident, $enc:ident) => {
        $(#[$doc])*
        pub fn $name<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
            self.emit(inst::$enc(rd.index(), rn.index(), rm.index(), V::SIZE, V::Q))
        }
    };
}

impl<'a> CodeGenerator<'a> {
    pub fn new(code: &'a mut BlockOfCode) -> Self {
        Self { code }
    }

    fn emit(&mut self, word: u32) -> Result<(), String> {
        self.code.write_u32(word).map(|_| ())
    }

    /// Bind `label` here — `oaknut::BasicCodeGenerator::l`.
    pub fn l(&mut self, label: &mut Label) -> Result<(), String> {
        label.bind(self.code)
    }

    // --- Branches -----------------------------------------------------------

    /// `B(Label&)`
    pub fn b(&mut self, label: &mut Label) -> Result<(), String> {
        label.b(self.code).map(|_| ())
    }

    /// `B(Cond, Label&)`
    pub fn b_cond(&mut self, cond: impl Into<Cond>, label: &mut Label) -> Result<(), String> {
        label.b_cond(self.code, cond).map(|_| ())
    }

    /// `CBZ(XReg, Label&)`
    pub fn cbz(&mut self, rt: XReg, label: &mut Label) -> Result<(), String> {
        label.cbz_x(self.code, rt.index()).map(|_| ())
    }

    /// `CBNZ(XReg, Label&)`
    pub fn cbnz(&mut self, rt: XReg, label: &mut Label) -> Result<(), String> {
        label.cbnz_x(self.code, rt.index()).map(|_| ())
    }

    /// `TBNZ(XReg, Imm<6>, Label&)`; see [`Label::tbnz_x`] for the far-target
    /// fallback that oaknut does not have.
    pub fn tbnz(&mut self, rt: XReg, bit: u8, label: &mut Label) -> Result<(), String> {
        label.tbnz_x(self.code, rt.index(), bit).map(|_| ())
    }

    /// `NOP()`
    pub fn nop(&mut self) -> Result<(), String> {
        self.emit(inst::nop())
    }

    /// `BLR(Xn)`
    pub fn blr(&mut self, rn: XReg) -> Result<(), String> {
        self.emit(inst::blr(rn.index()))
    }

    // --- Loads and stores (unsigned immediate offset) -----------------------

    /// `STR(Rt, [Xn|SP, #imm])`
    pub fn str<R: GpReg>(&mut self, rt: R, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        let rn = rn.into();
        self.emit(if R::SF {
            inst::str_x_unsigned(rt.index(), rn.index(), imm_bytes)
        } else {
            inst::str_w_unsigned(rt.index(), rn.index(), imm_bytes)
        })
    }

    /// `LDR(Rt, [Xn|SP, #imm])`
    pub fn ldr<R: GpReg>(&mut self, rt: R, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        let rn = rn.into();
        self.emit(if R::SF {
            inst::ldr_x_unsigned(rt.index(), rn.index(), imm_bytes)
        } else {
            inst::ldr_w_unsigned(rt.index(), rn.index(), imm_bytes)
        })
    }

    /// `MOV(Rd, imm)` — oaknut's immediate-materializing overload.
    ///
    /// oaknut chooses the shortest MOVZ/MOVN/MOVK/ORR sequence. This port
    /// keeps the sequence the emitters produced before they moved to the
    /// generator: `MOVZ` of the low half-word, then a `MOVK` for every
    /// non-zero higher half-word, so the emitted code is unchanged.
    pub fn mov_imm<R: GpReg>(&mut self, rd: R, imm: u64) -> Result<(), String> {
        self.movz(rd, (imm & 0xffff) as u16, 0)?;
        let shifts: &[u8] = if R::SF { &[16, 32, 48] } else { &[16] };
        for &shift in shifts {
            let chunk = ((imm >> shift) & 0xffff) as u16;
            if chunk != 0 {
                self.movk(rd, chunk, shift)?;
            }
        }
        Ok(())
    }

    /// `BFI(Rd, Rn, #lsb, #width)`
    pub fn bfi<R: GpReg>(&mut self, rd: R, rn: R, lsb: u8, width: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::bfi_x(rd.index(), rn.index(), lsb, width)
        } else {
            inst::bfi_w(rd.index(), rn.index(), lsb, width)
        })
    }

    // --- Data processing ----------------------------------------------------

    gp_mnemonic3!(/// `ADDS(Rd, Rn, Rm)`
        adds, adds_w_reg, adds_x_reg);
    gp_mnemonic3!(/// `SUBS(Rd, Rn, Rm)`
        subs, subs_w_reg, subs_x_reg);
    gp_mnemonic3!(/// `EOR(Rd, Rn, Rm)`
        eor, eor_w_reg, eor_x_reg);

    /// `ASR(Rd, Rn, Imm<6>)`
    pub fn asr<R: GpReg>(&mut self, rd: R, rn: R, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::asr_x_imm(rd.index(), rn.index(), shift)
        } else {
            inst::asr_w_imm(rd.index(), rn.index(), shift)
        })
    }

    /// `CMP(Rn, Rm)`
    pub fn cmp<R: GpReg>(&mut self, rn: R, rm: R) -> Result<(), String> {
        self.emit(if R::SF {
            inst::cmp_x_reg(rn.index(), rm.index())
        } else {
            inst::cmp_w_reg(rn.index(), rm.index())
        })
    }

    /// `CMP(Rn, AddSubImm)`
    pub fn cmp_imm<R: GpReg>(&mut self, rn: R, imm: u32) -> Result<(), String> {
        self.emit(if R::SF {
            inst::cmp_x_imm(rn.index(), imm)
        } else {
            inst::cmp_w_imm(rn.index(), imm)
        })
    }

    /// `MOV(Rd, Rm)`
    pub fn mov<R: GpReg>(&mut self, rd: R, rm: R) -> Result<(), String> {
        self.emit(if R::SF {
            inst::mov_x(rd.index(), rm.index())
        } else {
            inst::mov_w(rd.index(), rm.index())
        })
    }

    /// `MOVZ(Rd, Imm<16>, LSL #shift)`
    pub fn movz<R: GpReg>(&mut self, rd: R, imm16: u16, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::movz_x(rd.index(), imm16, shift)
        } else {
            inst::movz_w(rd.index(), imm16, shift)
        })
    }

    /// `MOVK(Rd, Imm<16>, LSL #shift)`
    pub fn movk<R: GpReg>(&mut self, rd: R, imm16: u16, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::movk_x(rd.index(), imm16, shift)
        } else {
            inst::movk_w(rd.index(), imm16, shift)
        })
    }

    /// `CINC(Rd, Rn, Cond)`
    pub fn cinc<R: GpReg>(&mut self, rd: R, rn: R, cond: impl Into<Cond>) -> Result<(), String> {
        let cond: Cond = cond.into();
        self.emit(if R::SF {
            inst::cinc_x(rd.index(), rn.index(), cond)
        } else {
            inst::cinc_w(rd.index(), rn.index(), cond)
        })
    }

    /// `CSEL(Rd, Rn, Rm, Cond)`
    pub fn csel<R: GpReg>(&mut self, rd: R, rn: R, rm: R, cond: impl Into<Cond>) -> Result<(), String> {
        let cond: Cond = cond.into();
        self.emit(if R::SF {
            inst::csel_x(rd.index(), rn.index(), rm.index(), cond)
        } else {
            inst::csel_w(rd.index(), rn.index(), rm.index(), cond)
        })
    }

    // --- CRC32 --------------------------------------------------------------

    /// `CRC32B(Wd, Wn, Wm)`
    pub fn crc32b(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32b_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32H(Wd, Wn, Wm)`
    pub fn crc32h(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32h_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32W(Wd, Wn, Wm)`
    pub fn crc32w(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32w_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32X(Wd, Wn, Xm)`
    pub fn crc32x(&mut self, wd: WReg, wn: WReg, xm: XReg) -> Result<(), String> {
        self.emit(inst::crc32x_x(wd.index(), wn.index(), xm.index()))
    }
    /// `CRC32CB(Wd, Wn, Wm)`
    pub fn crc32cb(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32cb_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32CH(Wd, Wn, Wm)`
    pub fn crc32ch(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32ch_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32CW(Wd, Wn, Wm)`
    pub fn crc32cw(&mut self, wd: WReg, wn: WReg, wm: WReg) -> Result<(), String> {
        self.emit(inst::crc32cw_w(wd.index(), wn.index(), wm.index()))
    }
    /// `CRC32CX(Wd, Wn, Xm)`
    pub fn crc32cx(&mut self, wd: WReg, wn: WReg, xm: XReg) -> Result<(), String> {
        self.emit(inst::crc32cx_x(wd.index(), wn.index(), xm.index()))
    }

    // --- Cryptography -------------------------------------------------------

    /// `AESD(Vd.16B, Vn.16B)`
    pub fn aesd(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::aesd_v16b(rd.index(), rn.index()))
    }
    /// `AESE(Vd.16B, Vn.16B)`
    pub fn aese(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::aese_v16b(rd.index(), rn.index()))
    }
    /// `AESIMC(Vd.16B, Vn.16B)`
    pub fn aesimc(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::aesimc_v16b(rd.index(), rn.index()))
    }
    /// `AESMC(Vd.16B, Vn.16B)`
    pub fn aesmc(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::aesmc_v16b(rd.index(), rn.index()))
    }
    /// `SHA256H(Qd, Qn, Vm.4S)`
    pub fn sha256h(&mut self, qd: QReg, qn: QReg, vm: VReg4S) -> Result<(), String> {
        self.emit(inst::sha256h_q(qd.index(), qn.index(), vm.index()))
    }
    /// `SHA256H2(Qd, Qn, Vm.4S)`
    pub fn sha256h2(&mut self, qd: QReg, qn: QReg, vm: VReg4S) -> Result<(), String> {
        self.emit(inst::sha256h2_q(qd.index(), qn.index(), vm.index()))
    }
    /// `SHA256SU0(Vd.4S, Vn.4S)`
    pub fn sha256su0(&mut self, rd: VReg4S, rn: VReg4S) -> Result<(), String> {
        self.emit(inst::sha256su0_v4s(rd.index(), rn.index()))
    }
    /// `SHA256SU1(Vd.4S, Vn.4S, Vm.4S)`
    pub fn sha256su1(&mut self, rd: VReg4S, rn: VReg4S, rm: VReg4S) -> Result<(), String> {
        self.emit(inst::sha256su1_v4s(rd.index(), rn.index(), rm.index()))
    }

    // --- SIMD ---------------------------------------------------------------

    /// `MOVI(Dd, #0)` — the zeroing form the emitters use.
    pub fn movi_zero(&mut self, rd: DReg) -> Result<(), String> {
        self.emit(inst::movi_d_imm0(rd.index()))
    }

    vec_mnemonic3!(/// `SQADD(Vd.T, Vn.T, Vm.T)`
        sqadd, sqadd_v);
    vec_mnemonic3!(/// `SQSUB(Vd.T, Vn.T, Vm.T)`
        sqsub, sqsub_v);
    vec_mnemonic3!(/// `UQADD(Vd.T, Vn.T, Vm.T)`
        uqadd, uqadd_v);
    vec_mnemonic3!(/// `UQSUB(Vd.T, Vn.T, Vm.T)`
        uqsub, uqsub_v);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reg::*;

    fn words(code: &BlockOfCode) -> Vec<u32> {
        (0..code.code_size() / 4)
            .map(|i| {
                let p = unsafe { code.code_base_ptr().add(i * 4) as *const u32 };
                unsafe { p.read_unaligned() }
            })
            .collect()
    }

    #[test]
    fn typed_mnemonics_emit_the_same_words_as_the_encoders() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.crc32cx(W1, W2, X3).unwrap();
        code.adds(W4, W5, W6).unwrap();
        code.adds(X4, X5, X6).unwrap();
        code.cinc(X7, X8, Cond::NE).unwrap();
        code.csel(W7, W8, W9, Cond::LT).unwrap();
        code.sqadd(V1.s4(), V2.s4(), V3.s4()).unwrap();
        code.sqadd(V1.b8(), V2.b8(), V3.b8()).unwrap();
        code.sha256h(Q0, Q1, V2.s4()).unwrap();
        code.movi_zero(D9).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![
                inst::crc32cx_x(1, 2, 3),
                inst::adds_w_reg(4, 5, 6),
                inst::adds_x_reg(4, 5, 6),
                inst::cinc_x(7, 8, Cond::NE),
                inst::csel_w(7, 8, 9, Cond::LT),
                inst::sqadd_v(1, 2, 3, 32, true),
                inst::sqadd_v(1, 2, 3, 8, false),
                inst::sha256h_q(0, 1, 2),
                inst::movi_d_imm0(9),
            ]
        );
    }

    #[test]
    fn memory_and_immediate_mnemonics_match_the_encoders() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.str(WZR, X28, 0x40).unwrap();
        code.ldr(X3, X16, 0).unwrap();
        code.bfi(X3, X17, 32, 32).unwrap();
        code.bfi(W3, W17, 8, 8).unwrap();
        code.blr(X16).unwrap();
        code.mov_imm(W5, 0x8000_0000).unwrap();
        code.mov_imm(X6, 0x1234_0000_5678).unwrap();
        code.mov_imm(X7, 0).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![
                inst::str_w_unsigned(31, 28, 0x40),
                inst::ldr_x_unsigned(3, 16, 0),
                inst::bfi_x(3, 17, 32, 32),
                inst::bfi_w(3, 17, 8, 8),
                inst::blr(16),
                inst::movz_w(5, 0, 0),
                inst::movk_w(5, 0x8000, 16),
                inst::movz_x(6, 0x5678, 0),
                inst::movk_x(6, 0x1234, 32),
                inst::movz_x(7, 0, 0),
            ]
        );
    }

    #[test]
    fn labels_bind_through_the_generator() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        let mut label = Label::new();
        code.cbz(X0, &mut label).unwrap();
        code.nop().unwrap();
        code.l(&mut label).unwrap();
        drop(code);
        assert_eq!(words(&block), vec![inst::cbz_x(0, 8), inst::nop()]);
    }
}
