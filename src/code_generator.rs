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
    DReg, FpReg, GpReg, GpRegSp, LdStKind, LdStReg, QReg, VReg16B, VReg2D, VReg4H, VReg4S,
    VReg8H, VRegArranged, VRegBytes, WReg, XReg, XRegSp,
};
use crate::enums::{BarrierOp, SystemReg};

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

// FP vector mnemonics: the encoders exist per arrangement (`fadd_v4s`,
// `fadd_v2d`), so the generic method dispatches on the arrangement's size
// and `Q`; other arrangements have no encoder yet and are a caller bug.
macro_rules! fp_vec_mnemonic3 {
    ($(#[$doc:meta])* $name:ident, $v4s:ident, $v2d:ident) => {
        $(#[$doc])*
        pub fn $name<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
            self.emit(match (V::SIZE, V::Q) {
                (32, true) => inst::$v4s(rd.index(), rn.index(), rm.index()),
                (64, true) => inst::$v2d(rd.index(), rn.index(), rm.index()),
                _ => panic!(concat!(stringify!($name), " has encoders only for the 4S and 2D arrangements")),
            })
        }
    };
}

macro_rules! fp_vec_mnemonic2 {
    ($(#[$doc:meta])* $name:ident, $v4s:ident, $v2d:ident) => {
        $(#[$doc])*
        pub fn $name<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
            self.emit(match (V::SIZE, V::Q) {
                (32, true) => inst::$v4s(rd.index(), rn.index()),
                (64, true) => inst::$v2d(rd.index(), rn.index()),
                _ => panic!(concat!(stringify!($name), " has encoders only for the 4S and 2D arrangements")),
            })
        }
    };
}

macro_rules! fp_vec_mnemonic2_fbits {
    ($(#[$doc:meta])* $name:ident, $v4s:ident, $v2d:ident) => {
        $(#[$doc])*
        pub fn $name<V: VRegArranged>(&mut self, rd: V, rn: V, fbits: u8) -> Result<(), String> {
            self.emit(match (V::SIZE, V::Q) {
                (32, true) => inst::$v4s(rd.index(), rn.index(), fbits),
                (64, true) => inst::$v2d(rd.index(), rn.index(), fbits),
                _ => panic!(concat!(stringify!($name), " has encoders only for the 4S and 2D arrangements")),
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

/// Arithmetic immediates shift by 0 or 12 only.
fn arith_imm_shift12(shift: u8) -> bool {
    match shift {
        0 => false,
        12 => true,
        _ => panic!("arithmetic immediate shift must be 0 or 12, got {shift}"),
    }
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
    pub fn cbz<R: GpReg>(&mut self, rt: R, label: &mut Label) -> Result<(), String> {
        if R::SF {
            label.cbz_x(self.code, rt.index())?;
        } else {
            label.cbz_w(self.code, rt.index())?;
        }
        Ok(())
    }

    /// `CBNZ(Rt, label)`
    pub fn cbnz<R: GpReg>(&mut self, rt: R, label: &mut Label) -> Result<(), String> {
        if R::SF {
            label.cbnz_x(self.code, rt.index())?;
        } else {
            label.cbnz_w(self.code, rt.index())?;
        }
        Ok(())
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

    /// `STR(Rt, [Xn|SP, #imm])` for every GP and FP/SIMD register width.
    pub fn str<R: LdStReg>(&mut self, rt: R, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        let (rt, rn) = (rt.index(), rn.into().index());
        self.emit(match R::KIND {
            LdStKind::W => inst::str_w_unsigned(rt, rn, imm_bytes),
            LdStKind::X => inst::str_x_unsigned(rt, rn, imm_bytes),
            LdStKind::B => inst::str_b_unsigned(rt, rn, imm_bytes),
            LdStKind::H => inst::str_h_unsigned(rt, rn, imm_bytes),
            LdStKind::S => inst::str_s_unsigned(rt, rn, imm_bytes),
            LdStKind::D => inst::str_d_unsigned(rt, rn, imm_bytes),
            LdStKind::Q => inst::str_q_unsigned(rt, rn, imm_bytes),
        })
    }

    /// `LDR(Rt, [Xn|SP, #imm])` for every GP and FP/SIMD register width.
    pub fn ldr<R: LdStReg>(&mut self, rt: R, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        let (rt, rn) = (rt.index(), rn.into().index());
        self.emit(match R::KIND {
            LdStKind::W => inst::ldr_w_unsigned(rt, rn, imm_bytes),
            LdStKind::X => inst::ldr_x_unsigned(rt, rn, imm_bytes),
            LdStKind::B => inst::ldr_b_unsigned(rt, rn, imm_bytes),
            LdStKind::H => inst::ldr_h_unsigned(rt, rn, imm_bytes),
            LdStKind::S => inst::ldr_s_unsigned(rt, rn, imm_bytes),
            LdStKind::D => inst::ldr_d_unsigned(rt, rn, imm_bytes),
            LdStKind::Q => inst::ldr_q_unsigned(rt, rn, imm_bytes),
        })
    }

    /// `ADD(Rd|SP, Rn|SP, #imm)`; `rd` accepts the `SP`-capable type or the
    /// plain register, as oaknut's implicit `XReg -> XRegSp` conversion does.
    pub fn add_imm<R: GpRegSp>(&mut self, rd: impl Into<R>, rn: R, imm: u32) -> Result<(), String> {
        let rd = rd.into();
        self.emit(if R::SF {
            inst::add_x_imm(rd.index(), rn.index(), imm)
        } else {
            inst::add_w_imm(rd.index(), rn.index(), imm)
        })
    }

    /// `SUB(Rd|SP, Rn|SP, #imm)`
    pub fn sub_imm<R: GpRegSp>(&mut self, rd: impl Into<R>, rn: R, imm: u32) -> Result<(), String> {
        let rd = rd.into();
        self.emit(if R::SF {
            inst::sub_x_imm(rd.index(), rn.index(), imm)
        } else {
            inst::sub_w_imm(rd.index(), rn.index(), imm)
        })
    }

    /// `BR(Xn)`
    pub fn br(&mut self, rn: XReg) -> Result<(), String> {
        self.emit(inst::br(rn.index()))
    }

    /// `LDAR(Rt, [Xn|SP])`
    pub fn ldar<R: GpReg>(&mut self, rt: R, rn: impl Into<XRegSp>) -> Result<(), String> {
        let rn = rn.into();
        self.emit(if R::SF {
            inst::ldar_x(rt.index(), rn.index())
        } else {
            inst::ldar_w(rt.index(), rn.index())
        })
    }

    /// `LDRB(Wt, [Xn|SP, #imm])`
    pub fn ldrb(&mut self, wt: WReg, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        self.emit(inst::ldrb_w_unsigned(wt.index(), rn.into().index(), imm_bytes))
    }

    /// `LDP(Rt, Rt2, [Xn|SP, #imm])`
    pub fn ldp<R: GpReg>(&mut self, rt: R, rt2: R, rn: impl Into<XRegSp>, imm_bytes: i32) -> Result<(), String> {
        let rn = rn.into();
        self.emit(if R::SF {
            inst::ldp_x_offset(rt.index(), rt2.index(), rn.index(), imm_bytes)
        } else {
            inst::ldp_w_offset(rt.index(), rt2.index(), rn.index(), imm_bytes)
        })
    }

    /// `STP(Rt, Rt2, [Xn|SP, #imm])`
    pub fn stp<R: GpReg>(&mut self, rt: R, rt2: R, rn: impl Into<XRegSp>, imm_bytes: i32) -> Result<(), String> {
        let rn = rn.into();
        self.emit(if R::SF {
            inst::stp_x_offset(rt.index(), rt2.index(), rn.index(), imm_bytes)
        } else {
            inst::stp_w_offset(rt.index(), rt2.index(), rn.index(), imm_bytes)
        })
    }

    /// `ADD(Xd|SP, Xn|SP, Xm)` — the extended-register form oaknut selects
    /// for `SP` operands (`UXTX #0`).
    pub fn add_ext(&mut self, rd: impl Into<XRegSp>, rn: impl Into<XRegSp>, rm: XReg) -> Result<(), String> {
        self.emit(inst::add_x_reg_sp(rd.into().index(), rn.into().index(), rm.index()))
    }

    /// `AND(Rd, Rn, #imm)` (bitmask immediate)
    pub fn and_imm<R: GpReg>(&mut self, rd: R, rn: R, imm: u64) -> Result<(), String> {
        self.emit(if R::SF {
            inst::and_x_imm(rd.index(), rn.index(), imm)
        } else {
            inst::and_w_imm(rd.index(), rn.index(), imm as u32)
        })
    }

    /// `TST(Rn, #imm)` (bitmask immediate)
    pub fn tst_imm<R: GpReg>(&mut self, rn: R, imm: u64) -> Result<(), String> {
        self.emit(if R::SF {
            inst::tst_x_imm(rn.index(), imm)
        } else {
            inst::tst_w_imm(rn.index(), imm as u32)
        })
    }

    /// `LSL(Rd, Rn, #shift)`
    pub fn lsl<R: GpReg>(&mut self, rd: R, rn: R, shift: u8) -> Result<(), String> {
        self.emit(if R::SF {
            inst::lsl_x_imm(rd.index(), rn.index(), shift)
        } else {
            inst::lsl_w_imm(rd.index(), rn.index(), shift)
        })
    }

    /// `BRK(#imm16)`
    pub fn brk(&mut self, imm16: u16) -> Result<(), String> {
        self.emit(inst::brk(imm16))
    }

    /// `DSB(option)`
    pub fn dsb(&mut self, option: BarrierOp) -> Result<(), String> {
        self.emit(inst::dsb(option as u8))
    }

    /// `DMB(option)`
    pub fn dmb(&mut self, option: BarrierOp) -> Result<(), String> {
        self.emit(inst::dmb(option as u8))
    }

    /// `STRB(Wt, [Xn|SP, #imm])`
    pub fn strb(&mut self, wt: WReg, rn: impl Into<XRegSp>, imm_bytes: u32) -> Result<(), String> {
        self.emit(inst::strb_w_unsigned(wt.index(), rn.into().index(), imm_bytes))
    }

    /// `ADD(Rd|SP, Rn|SP, #imm12, LSL #shift)` with `shift` 0 or 12.
    pub fn add_imm_shift<R: GpRegSp>(&mut self, rd: impl Into<R>, rn: R, imm12: u32, shift: u8) -> Result<(), String> {
        let rd = rd.into();
        let shift12 = arith_imm_shift12(shift);
        self.emit(if R::SF {
            inst::add_x_imm_shift(rd.index(), rn.index(), imm12, shift12)
        } else {
            inst::add_w_imm_shift(rd.index(), rn.index(), imm12, shift12)
        })
    }

    /// `SUB(Rd|SP, Rn|SP, #imm12, LSL #shift)` with `shift` 0 or 12.
    pub fn sub_imm_shift<R: GpRegSp>(&mut self, rd: impl Into<R>, rn: R, imm12: u32, shift: u8) -> Result<(), String> {
        let rd = rd.into();
        let shift12 = arith_imm_shift12(shift);
        self.emit(if R::SF {
            inst::sub_x_imm_shift(rd.index(), rn.index(), imm12, shift12)
        } else {
            inst::sub_w_imm_shift(rd.index(), rn.index(), imm12, shift12)
        })
    }

    /// `MSR(SystemReg, Xt)`
    pub fn msr(&mut self, sysreg: SystemReg, rt: XReg) -> Result<(), String> {
        self.emit(match sysreg {
            SystemReg::FPCR => inst::msr_fpcr(rt.index()),
            SystemReg::FPSR => inst::msr_fpsr(rt.index()),
            SystemReg::NZCV => inst::msr_nzcv(rt.index()),
        })
    }

    /// `MRS(Xt, SystemReg)`
    pub fn mrs(&mut self, rt: XReg, sysreg: SystemReg) -> Result<(), String> {
        self.emit(match sysreg {
            SystemReg::FPCR => inst::mrs_fpcr(rt.index()),
            SystemReg::FPSR => inst::mrs_fpsr(rt.index()),
            SystemReg::NZCV => inst::mrs_nzcv(rt.index()),
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
    gp_mnemonic3!(/// `ADD(Rd, Rn, Rm)`
        add, add_w_reg, add_x_reg);
    gp_mnemonic3!(/// `SUB(Rd, Rn, Rm)`
        sub, sub_w_reg, sub_x_reg);
    gp_mnemonic3!(/// `AND(Rd, Rn, Rm)`
        and, and_w_reg, and_x_reg);
    gp_mnemonic3!(/// `ORR(Rd, Rn, Rm)`
        orr, orr_w, orr_x);

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

    // Mnemonics oaknut overloads across general-purpose and SIMD operands
    // (`ADD`, `SUB`, `AND`, `EOR`, …) keep the bare name for the GP form and
    // take a `_v` suffix for the vector form; Rust has no overloading.

    vec_mnemonic3!(/// `ADD(Vd.T, Vn.T, Vm.T)`
        add_v, add_v);
    vec_mnemonic3!(/// `SUB(Vd.T, Vn.T, Vm.T)`
        sub_v, sub_v);
    vec_mnemonic3!(/// `CMHI(Vd.T, Vn.T, Vm.T)`
        cmhi, cmhi_v);
    vec_mnemonic3!(/// `SHADD(Vd.T, Vn.T, Vm.T)`
        shadd, shadd_v);
    vec_mnemonic3!(/// `SHSUB(Vd.T, Vn.T, Vm.T)`
        shsub, shsub_v);
    vec_mnemonic3!(/// `UHADD(Vd.T, Vn.T, Vm.T)`
        uhadd, uhadd_v);
    vec_mnemonic3!(/// `UHSUB(Vd.T, Vn.T, Vm.T)`
        uhsub, uhsub_v);
    vec_mnemonic3!(/// `UABD(Vd.T, Vn.T, Vm.T)`
        uabd, uabd_v);

    /// `CMEQ(Vd.T, Vn.T, #0)`
    pub fn cmeq_zero<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        self.emit(inst::cmeq_v_zero(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// `CMGE(Vd.T, Vn.T, #0)`
    pub fn cmge_zero<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        self.emit(inst::cmge_v_zero(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// `SSHR(Vd.T, Vn.T, #shift)`
    pub fn sshr<V: VRegArranged>(&mut self, rd: V, rn: V, shift: u8) -> Result<(), String> {
        self.emit(inst::sshr_v(rd.index(), rn.index(), V::SIZE, shift, V::Q))
    }

    /// `USHR(Vd.T, Vn.T, #shift)`
    pub fn ushr<V: VRegArranged>(&mut self, rd: V, rn: V, shift: u8) -> Result<(), String> {
        self.emit(inst::ushr_v(rd.index(), rn.index(), V::SIZE, shift, V::Q))
    }

    // Widening and narrowing forms name both arrangements as the assembler
    // syntax does (`SXTL Vd.4S, Vn.4H`); the source arrangement selects the
    // encoding, as in oaknut's per-arrangement overloads.

    /// `SXTL(Vd.Tw, Vn.Tn)`
    pub fn sxtl<D: VRegArranged, N: VRegArranged>(&mut self, rd: D, rn: N) -> Result<(), String> {
        self.emit(inst::sxtl_v(rd.index(), rn.index(), N::SIZE))
    }

    /// `UXTL(Vd.Tw, Vn.Tn)`
    pub fn uxtl<D: VRegArranged, N: VRegArranged>(&mut self, rd: D, rn: N) -> Result<(), String> {
        self.emit(inst::uxtl_v(rd.index(), rn.index(), N::SIZE))
    }

    /// `XTN(Vd.Tn, Vn.Tw)`
    pub fn xtn<D: VRegArranged, N: VRegArranged>(&mut self, rd: D, rn: N) -> Result<(), String> {
        self.emit(inst::xtn_v(rd.index(), rn.index(), N::SIZE))
    }

    /// `SHRN(Vd.Tn, Vn.Tw, #shift)`
    pub fn shrn<D: VRegArranged, N: VRegArranged>(&mut self, rd: D, rn: N, shift: u8) -> Result<(), String> {
        self.emit(inst::shrn_v(rd.index(), rn.index(), N::SIZE, shift))
    }

    /// `UADDLV(Rd, Vn.T)` — scalar destination one size wider than the lanes.
    pub fn uaddlv<F: FpReg, V: VRegArranged>(&mut self, rd: F, rn: V) -> Result<(), String> {
        self.emit(inst::uaddlv_from_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// `AND(Vd.8B|16B, Vn, Vm)`
    pub fn and_v<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        self.emit(if V::Q {
            inst::and_v16b(rd.index(), rn.index(), rm.index())
        } else {
            inst::and_v8b(rd.index(), rn.index(), rm.index())
        })
    }

    /// `EOR(Vd.8B|16B, Vn, Vm)`
    pub fn eor_v<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        self.emit(if V::Q {
            inst::eor_v16b(rd.index(), rn.index(), rm.index())
        } else {
            inst::eor_v8b(rd.index(), rn.index(), rm.index())
        })
    }

    /// `BSL(Vd.8B|16B, Vn, Vm)`
    pub fn bsl<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        self.emit(if V::Q {
            inst::bsl_v16b(rd.index(), rn.index(), rm.index())
        } else {
            inst::bsl_v8b(rd.index(), rn.index(), rm.index())
        })
    }

    /// `EXT(Vd.8B|16B, Vn, Vm, #index)`
    pub fn ext<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V, index: u8) -> Result<(), String> {
        self.emit(inst::ext_v16b(rd.index(), rn.index(), rm.index(), index, V::Q))
    }

    /// `MOVI(Vd.8B|16B, #imm8)`
    pub fn movi<V: VRegBytes>(&mut self, rd: V, imm: u8) -> Result<(), String> {
        self.emit(if V::Q {
            inst::movi_v16b_imm(rd.index(), imm)
        } else {
            inst::movi_v8b_imm(rd.index(), imm)
        })
    }

    /// `FMOV(Sd, Sn)` / `FMOV(Dd, Dn)`
    pub fn fmov<F: FpReg>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::SIZE {
            2 => inst::fmov_s(rd.index(), rn.index()),
            3 => inst::fmov_d(rd.index(), rn.index()),
            size => panic!("FMOV register-to-register has no encoding for FP size {size}"),
        })
    }

    vec_mnemonic3!(/// `ZIP1(Vd.T, Vn.T, Vm.T)`
        zip1, zip1_v);

    /// `BIC(Vd.8H, #imm8, LSL #shift)` (vector, immediate)
    pub fn bic_imm(&mut self, rd: VReg8H, imm8: u8, lsl: u8) -> Result<(), String> {
        self.emit(inst::bic_v8h_imm(rd.index(), imm8, lsl))
    }

    fp_vec_mnemonic3!(/// `FADD(Vd.T, Vn.T, Vm.T)`
        fadd, fadd_v4s, fadd_v2d);
    fp_vec_mnemonic3!(/// `FSUB(Vd.T, Vn.T, Vm.T)`
        fsub, fsub_v4s, fsub_v2d);
    fp_vec_mnemonic3!(/// `FMUL(Vd.T, Vn.T, Vm.T)`
        fmul, fmul_v4s, fmul_v2d);
    fp_vec_mnemonic3!(/// `FMULX(Vd.T, Vn.T, Vm.T)`
        fmulx, fmulx_v4s, fmulx_v2d);
    fp_vec_mnemonic3!(/// `FDIV(Vd.T, Vn.T, Vm.T)`
        fdiv, fdiv_v4s, fdiv_v2d);
    fp_vec_mnemonic3!(/// `FMAX(Vd.T, Vn.T, Vm.T)`
        fmax, fmax_v4s, fmax_v2d);
    fp_vec_mnemonic3!(/// `FMAXNM(Vd.T, Vn.T, Vm.T)`
        fmaxnm, fmaxnm_v4s, fmaxnm_v2d);
    fp_vec_mnemonic3!(/// `FMIN(Vd.T, Vn.T, Vm.T)`
        fmin, fmin_v4s, fmin_v2d);
    fp_vec_mnemonic3!(/// `FMINNM(Vd.T, Vn.T, Vm.T)`
        fminnm, fminnm_v4s, fminnm_v2d);
    fp_vec_mnemonic3!(/// `FCMEQ(Vd.T, Vn.T, Vm.T)`
        fcmeq, fcmeq_v4s, fcmeq_v2d);
    fp_vec_mnemonic3!(/// `FCMGT(Vd.T, Vn.T, Vm.T)`
        fcmgt, fcmgt_v4s, fcmgt_v2d);
    fp_vec_mnemonic3!(/// `FCMGE(Vd.T, Vn.T, Vm.T)`
        fcmge, fcmge_v4s, fcmge_v2d);
    fp_vec_mnemonic3!(/// `FMLA(Vd.T, Vn.T, Vm.T)`
        fmla, fmla_v4s, fmla_v2d);
    fp_vec_mnemonic3!(/// `FADDP(Vd.T, Vn.T, Vm.T)`
        faddp, faddp_v4s, faddp_v2d);
    fp_vec_mnemonic3!(/// `FRECPS(Vd.T, Vn.T, Vm.T)`
        frecps, frecps_v4s, frecps_v2d);
    fp_vec_mnemonic3!(/// `FRSQRTS(Vd.T, Vn.T, Vm.T)`
        frsqrts, frsqrts_v4s, frsqrts_v2d);

    fp_vec_mnemonic2!(/// `FABS(Vd.T, Vn.T)`
        fabs, fabs_v4s, fabs_v2d);
    fp_vec_mnemonic2!(/// `FNEG(Vd.T, Vn.T)`
        fneg, fneg_v4s, fneg_v2d);
    fp_vec_mnemonic2!(/// `FSQRT(Vd.T, Vn.T)`
        fsqrt, fsqrt_v4s, fsqrt_v2d);
    fp_vec_mnemonic2!(/// `FRECPE(Vd.T, Vn.T)`
        frecpe, frecpe_v4s, frecpe_v2d);
    fp_vec_mnemonic2!(/// `FRSQRTE(Vd.T, Vn.T)`
        frsqrte, frsqrte_v4s, frsqrte_v2d);
    fp_vec_mnemonic2!(/// `FRINTN(Vd.T, Vn.T)`
        frintn, frintn_v4s, frintn_v2d);
    fp_vec_mnemonic2!(/// `FRINTP(Vd.T, Vn.T)`
        frintp, frintp_v4s, frintp_v2d);
    fp_vec_mnemonic2!(/// `FRINTM(Vd.T, Vn.T)`
        frintm, frintm_v4s, frintm_v2d);
    fp_vec_mnemonic2!(/// `FRINTZ(Vd.T, Vn.T)`
        frintz, frintz_v4s, frintz_v2d);
    fp_vec_mnemonic2!(/// `FRINTA(Vd.T, Vn.T)`
        frinta, frinta_v4s, frinta_v2d);
    fp_vec_mnemonic2!(/// `FRINTX(Vd.T, Vn.T)`
        frintx, frintx_v4s, frintx_v2d);
    fp_vec_mnemonic2!(/// `SCVTF(Vd.T, Vn.T)`
        scvtf, scvtf_v4s, scvtf_v2d);
    fp_vec_mnemonic2!(/// `UCVTF(Vd.T, Vn.T)`
        ucvtf, ucvtf_v4s, ucvtf_v2d);
    fp_vec_mnemonic2!(/// `FCVTZS(Vd.T, Vn.T)`
        fcvtzs, fcvtzs_v4s, fcvtzs_v2d);
    fp_vec_mnemonic2!(/// `FCVTZU(Vd.T, Vn.T)`
        fcvtzu, fcvtzu_v4s, fcvtzu_v2d);
    fp_vec_mnemonic2!(/// `FCVTNS(Vd.T, Vn.T)`
        fcvtns, fcvtns_v4s, fcvtns_v2d);
    fp_vec_mnemonic2!(/// `FCVTPS(Vd.T, Vn.T)`
        fcvtps, fcvtps_v4s, fcvtps_v2d);
    fp_vec_mnemonic2!(/// `FCVTMS(Vd.T, Vn.T)`
        fcvtms, fcvtms_v4s, fcvtms_v2d);
    fp_vec_mnemonic2!(/// `FCVTAS(Vd.T, Vn.T)`
        fcvtas, fcvtas_v4s, fcvtas_v2d);
    fp_vec_mnemonic2!(/// `FCVTNU(Vd.T, Vn.T)`
        fcvtnu, fcvtnu_v4s, fcvtnu_v2d);
    fp_vec_mnemonic2!(/// `FCVTPU(Vd.T, Vn.T)`
        fcvtpu, fcvtpu_v4s, fcvtpu_v2d);
    fp_vec_mnemonic2!(/// `FCVTMU(Vd.T, Vn.T)`
        fcvtmu, fcvtmu_v4s, fcvtmu_v2d);
    fp_vec_mnemonic2!(/// `FCVTAU(Vd.T, Vn.T)`
        fcvtau, fcvtau_v4s, fcvtau_v2d);

    // oaknut overloads the fixed-point forms on arity (`SCVTF(Vd, Vn, #fbits)`).
    fp_vec_mnemonic2_fbits!(/// `SCVTF(Vd.T, Vn.T, #fbits)`
        scvtf_fixed, scvtf_v4s_fixed, scvtf_v2d_fixed);
    fp_vec_mnemonic2_fbits!(/// `UCVTF(Vd.T, Vn.T, #fbits)`
        ucvtf_fixed, ucvtf_v4s_fixed, ucvtf_v2d_fixed);
    fp_vec_mnemonic2_fbits!(/// `FCVTZS(Vd.T, Vn.T, #fbits)`
        fcvtzs_fixed, fcvtzs_v4s_fixed, fcvtzs_v2d_fixed);
    fp_vec_mnemonic2_fbits!(/// `FCVTZU(Vd.T, Vn.T, #fbits)`
        fcvtzu_fixed, fcvtzu_v4s_fixed, fcvtzu_v2d_fixed);

    /// `FCVTL(Vd.4S, Vn.4H)`
    pub fn fcvtl(&mut self, rd: VReg4S, rn: VReg4H) -> Result<(), String> {
        self.emit(inst::fcvtl_v4s_from_v4h(rd.index(), rn.index()))
    }

    /// `FCVTN(Vd.4H, Vn.4S)`
    pub fn fcvtn(&mut self, rd: VReg4H, rn: VReg4S) -> Result<(), String> {
        self.emit(inst::fcvtn_v4h_from_v4s(rd.index(), rn.index()))
    }

    /// `FADDP(Dd, Vn.2D)` — the scalar pairwise form.
    pub fn faddp_scalar(&mut self, rd: DReg, rn: VReg2D) -> Result<(), String> {
        self.emit(inst::faddp_d_from_v2d(rd.index(), rn.index()))
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
    fn packed_simd_mnemonics_match_the_encoders() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.add_v(V1.h4(), V2.h4(), V3.h4()).unwrap();
        code.sub_v(V1.s2(), V2.s2(), V3.s2()).unwrap();
        code.cmhi(V1.b8(), V2.b8(), V3.b8()).unwrap();
        code.uhadd(V1.h4(), V2.h4(), V3.h4()).unwrap();
        code.uabd(V1.b8(), V2.b8(), V3.b8()).unwrap();
        code.cmeq_zero(V4.h4(), V5.h4()).unwrap();
        code.cmge_zero(V4.s2(), V5.s2()).unwrap();
        code.sshr(V4.s2(), V5.s2(), 1).unwrap();
        code.ushr(V4.s2(), V5.s2(), 1).unwrap();
        code.sxtl(V0.s4(), V6.h4()).unwrap();
        code.uxtl(V0.h8(), V6.b8()).unwrap();
        code.xtn(V0.h4(), V0.s4()).unwrap();
        code.shrn(V0.h4(), V0.s4(), 16).unwrap();
        code.uaddlv(H7, V7.b8()).unwrap();
        code.and_v(V1.b8(), V1.b8(), V2.b8()).unwrap();
        code.eor_v(V1.b16(), V1.b16(), V2.b16()).unwrap();
        code.bsl(V1.b8(), V2.b8(), V3.b8()).unwrap();
        code.bsl(V1.b16(), V2.b16(), V3.b16()).unwrap();
        code.ext(V1.b8(), V1.b8(), V1.b8(), 4).unwrap();
        code.movi(V2.b8(), 0b1111_0000).unwrap();
        code.movi(V2.b16(), 0xff).unwrap();
        code.fmov(D1, D2).unwrap();
        code.fmov(S1, S2).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![
                inst::add_v(1, 2, 3, 16, false),
                inst::sub_v(1, 2, 3, 32, false),
                inst::cmhi_v(1, 2, 3, 8, false),
                inst::uhadd_v(1, 2, 3, 16, false),
                inst::uabd_v(1, 2, 3, 8, false),
                inst::cmeq_v_zero(4, 5, 16, false),
                inst::cmge_v_zero(4, 5, 32, false),
                inst::sshr_v(4, 5, 32, 1, false),
                inst::ushr_v(4, 5, 32, 1, false),
                inst::sxtl_v(0, 6, 16),
                inst::uxtl_v(0, 6, 8),
                inst::xtn_v(0, 0, 32),
                inst::shrn_v(0, 0, 32, 16),
                inst::uaddlv_from_v(7, 7, 8, false),
                inst::and_v8b(1, 1, 2),
                inst::eor_v16b(1, 1, 2),
                inst::bsl_v8b(1, 2, 3),
                inst::bsl_v16b(1, 2, 3),
                inst::ext_v16b(1, 1, 1, 4, false),
                inst::movi_v8b_imm(2, 0b1111_0000),
                inst::movi_v16b_imm(2, 0xff),
                inst::fmov_d(1, 2),
                inst::fmov_s(1, 2),
            ]
        );
    }

    #[test]
    fn fp_vector_mnemonics_match_the_encoders() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.fadd(V1.s4(), V2.s4(), V3.s4()).unwrap();
        code.fmla(V1.d2(), V2.d2(), V3.d2()).unwrap();
        code.frintx(V4.s4(), V5.s4()).unwrap();
        code.fcvtau(V4.d2(), V5.d2()).unwrap();
        code.scvtf_fixed(V4.s4(), V5.s4(), 7).unwrap();
        code.fcvtzu_fixed(V4.d2(), V5.d2(), 9).unwrap();
        code.fcvtl(V6.s4(), V7.h4()).unwrap();
        code.fcvtn(V6.h4(), V7.s4()).unwrap();
        code.faddp_scalar(D6, V7.d2()).unwrap();
        code.zip1(V0.d2(), V1.d2(), V2.d2()).unwrap();
        code.bic_imm(V8.h8(), 0b1000_0000, 8).unwrap();
        code.msr(SystemReg::FPCR, X16).unwrap();
        code.mrs(X17, SystemReg::FPSR).unwrap();
        code.add_imm(X0, SP, 16).unwrap();
        code.add_imm(X3, X28, 0x40).unwrap();
        code.sub_imm(W1, W2, 4).unwrap();
        code.str(Q1, X1, 0).unwrap();
        code.ldr(Q2, SP, 16).unwrap();
        code.str(D3, X4, 8).unwrap();
        code.ldr(B5, X6, 1).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![
                inst::fadd_v4s(1, 2, 3),
                inst::fmla_v2d(1, 2, 3),
                inst::frintx_v4s(4, 5),
                inst::fcvtau_v2d(4, 5),
                inst::scvtf_v4s_fixed(4, 5, 7),
                inst::fcvtzu_v2d_fixed(4, 5, 9),
                inst::fcvtl_v4s_from_v4h(6, 7),
                inst::fcvtn_v4h_from_v4s(6, 7),
                inst::faddp_d_from_v2d(6, 7),
                inst::zip1_v(0, 1, 2, 64, true),
                inst::bic_v8h_sign_bit(8),
                inst::msr_fpcr(16),
                inst::mrs_fpsr(17),
                inst::add_x_imm(0, 31, 16),
                inst::add_x_imm(3, 28, 0x40),
                inst::sub_w_imm(1, 2, 4),
                inst::str_q_unsigned(1, 1, 0),
                inst::ldr_q_unsigned(2, 31, 16),
                inst::str_d_unsigned(3, 4, 8),
                inst::ldr_b_unsigned(5, 6, 1),
            ]
        );
    }

    #[test]
    fn labels_bind_through_the_generator() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        let mut label = Label::new();
        code.cbz(X0, &mut label).unwrap();
        code.cbnz(W16, &mut label).unwrap();
        code.nop().unwrap();
        code.l(&mut label).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![inst::cbz_x(0, 12), inst::cbnz_w(16, 8), inst::nop()]
        );
    }

    #[test]
    fn terminal_mnemonics_match_the_encoders() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.br(X17).unwrap();
        code.ldar(W16, X27).unwrap();
        code.ldar(X16, X27).unwrap();
        code.ldrb(W16, SP, 0x30).unwrap();
        code.ldp(X16, X17, X2, 0x40).unwrap();
        code.stp(W1, W2, SP, 8).unwrap();
        code.add_ext(X2, SP, X30).unwrap();
        code.and_imm(X1, X1, 0x00ff_ffff_ffff_ffff).unwrap();
        code.and_imm(W30, W30, 0xff0).unwrap();
        code.tst_imm(X16, 0x10).unwrap();
        code.tst_imm(W16, 1).unwrap();
        code.lsl(X0, X0, 37).unwrap();
        code.add(X1, X2, X3).unwrap();
        code.sub(X1, X1, X26).unwrap();
        code.and(W0, W0, W16).unwrap();
        code.orr(X0, X0, X1).unwrap();
        code.msr(SystemReg::NZCV, X16).unwrap();
        code.brk(0).unwrap();
        code.dsb(BarrierOp::SY).unwrap();
        code.dmb(BarrierOp::ISH).unwrap();
        code.strb(W16, SP, 0x30).unwrap();
        code.sub_imm_shift(X26, X26, 1, 12).unwrap();
        code.add_imm_shift(W1, W2, 3, 0).unwrap();
        drop(code);
        assert_eq!(
            words(&block),
            vec![
                inst::br(17),
                inst::ldar_w(16, 27),
                inst::ldar_x(16, 27),
                inst::ldrb_w_unsigned(16, 31, 0x30),
                inst::ldp_x_offset(16, 17, 2, 0x40),
                inst::stp_w_offset(1, 2, 31, 8),
                inst::add_x_reg_sp(2, 31, 30),
                inst::and_x_imm(1, 1, 0x00ff_ffff_ffff_ffff),
                inst::and_w_imm(30, 30, 0xff0),
                inst::tst_x_imm(16, 0x10),
                inst::tst_w_imm(16, 1),
                inst::lsl_x_imm(0, 0, 37),
                inst::add_x_reg(1, 2, 3),
                inst::sub_x_reg(1, 1, 26),
                inst::and_w_reg(0, 0, 16),
                inst::orr_x(0, 0, 1),
                inst::msr_nzcv(16),
                inst::brk(0),
                inst::dsb_sy(),
                inst::dmb(0b1011),
                inst::strb_w_unsigned(16, 31, 0x30),
                inst::sub_x_imm_shift(26, 26, 1, true),
                inst::add_w_imm_shift(1, 2, 3, false),
            ]
        );
    }
}
