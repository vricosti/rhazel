//! Register operand types, the counterpart of `oaknut/impl/reg.hpp`.
//!
//! oaknut distinguishes operand classes by C++ type so that one mnemonic
//! overload set (`ADD(WReg, …)` / `ADD(XReg, …)` / `ADD(VReg_4S, …)`) picks
//! the encoding. Rust has no overloading, so `CodeGenerator` mnemonics are
//! generic over the small traits below, which carry the same information:
//! width (`sf`) for general-purpose registers, element size and `Q` for
//! arranged vector registers, and the scalar FP width for `B`/`H`/`S`/`D`/`Q`.
//!
//! Register 31 keeps oaknut's meaning: it is `XZR`/`WZR` through `XReg`/`WReg`
//! and `SP`/`WSP` through `XRegSp`/`WRegWsp`; the mnemonic's parameter type,
//! not the value, selects which one an encoding means.

/// A general-purpose register operand; `SF` is the AArch64 `sf` bit.
pub trait GpReg: Copy {
    const SF: bool;
    fn index(self) -> u8;
}

/// A general-purpose register operand in a position where 31 means `SP`.
pub trait GpRegSp: Copy {
    const SF: bool;
    fn index(self) -> u8;
}

/// A scalar FP/SIMD register operand; `SIZE` is the `size`/`ftype` field
/// (0 = B, 1 = H, 2 = S, 3 = D, 4 = Q as oaknut's bitsize order).
pub trait FpReg: Copy {
    const SIZE: u8;
    fn index(self) -> u8;
}

/// An arranged vector register operand (`Vn.8B`, `Vn.4S`, …).
pub trait VRegArranged: Copy {
    /// Element size in bits (8, 16, 32 or 64), the form the encoders take.
    const SIZE: u8;
    /// `Q` bit: 128-bit arrangement.
    const Q: bool;
    fn index(self) -> u8;
    /// The arrangement's `VRegSelector` accessor, for code generic over the
    /// arrangement (upstream's `EmitThreeOpArranged<fsize>` pattern).
    fn from_vreg(reg: VReg) -> Self;
}

/// The register classes `LDR`/`STR` accept; oaknut overloads them over every
/// general-purpose and FP/SIMD width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LdStKind {
    W,
    X,
    B,
    H,
    S,
    D,
    Q,
}

/// A register operand of a load or store.
pub trait LdStReg: Copy {
    const KIND: LdStKind;
    fn index(self) -> u8;
}

/// A byte-arranged vector register (`Vn.8B` or `Vn.16B`), the operand class
/// of the logical/permute mnemonics oaknut declares only for those two.
pub trait VRegBytes: VRegArranged {}

macro_rules! register_type {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name(u8);

        impl $name {
            /// Build from a register number (0..=31). Panics otherwise, as
            /// oaknut's `Reg` constructor asserts.
            pub const fn new(index: u8) -> Self {
                assert!(index < 32, "register index out of range");
                Self(index)
            }

            pub const fn index(self) -> u8 {
                self.0
            }
        }
    };
}

register_type!(/// 64-bit general-purpose register; 31 is `XZR`.
    XReg);
register_type!(/// 32-bit general-purpose register; 31 is `WZR`.
    WReg);
register_type!(/// 64-bit general-purpose register or `SP` (31).
    XRegSp);
register_type!(/// 32-bit general-purpose register or `WSP` (31).
    WRegWsp);
register_type!(/// 8-bit scalar FP/SIMD register.
    BReg);
register_type!(/// 16-bit scalar FP/SIMD register.
    HReg);
register_type!(/// 32-bit scalar FP/SIMD register.
    SReg);
register_type!(/// 64-bit scalar FP/SIMD register.
    DReg);
register_type!(/// 128-bit scalar FP/SIMD register.
    QReg);
register_type!(/// A vector register before its arrangement is chosen
    /// (`oaknut::VRegSelector`).
    VReg);
register_type!(/// `Vn.8B`
    VReg8B);
register_type!(/// `Vn.16B`
    VReg16B);
register_type!(/// `Vn.4H`
    VReg4H);
register_type!(/// `Vn.8H`
    VReg8H);
register_type!(/// `Vn.2S`
    VReg2S);
register_type!(/// `Vn.4S`
    VReg4S);
register_type!(/// `Vn.1D`
    VReg1D);
register_type!(/// `Vn.2D`
    VReg2D);

impl GpReg for WReg {
    const SF: bool = false;
    fn index(self) -> u8 {
        self.0
    }
}

impl GpReg for XReg {
    const SF: bool = true;
    fn index(self) -> u8 {
        self.0
    }
}

impl GpRegSp for WRegWsp {
    const SF: bool = false;
    fn index(self) -> u8 {
        self.0
    }
}

impl GpRegSp for XRegSp {
    const SF: bool = true;
    fn index(self) -> u8 {
        self.0
    }
}

// oaknut's `XRegSp(XReg)` converting constructor is implicit, so an `XReg`
// is accepted wherever an `SP`-capable operand is expected, with the same
// hazard: register 31 encodes `SP` in that position, not `XZR`.
impl GpRegSp for XReg {
    const SF: bool = true;
    fn index(self) -> u8 {
        self.0
    }
}

impl GpRegSp for WReg {
    const SF: bool = false;
    fn index(self) -> u8 {
        self.0
    }
}

macro_rules! ldst_reg {
    ($name:ident, $kind:ident) => {
        impl LdStReg for $name {
            const KIND: LdStKind = LdStKind::$kind;
            fn index(self) -> u8 {
                self.0
            }
        }
    };
}

ldst_reg!(WReg, W);
ldst_reg!(XReg, X);
ldst_reg!(BReg, B);
ldst_reg!(HReg, H);
ldst_reg!(SReg, S);
ldst_reg!(DReg, D);
ldst_reg!(QReg, Q);

/// oaknut converts `XReg` to `XRegSp` implicitly; a register number that is
/// not 31 means the same thing in both positions.
impl From<XReg> for XRegSp {
    fn from(reg: XReg) -> Self {
        XRegSp(reg.0)
    }
}

impl From<WReg> for WRegWsp {
    fn from(reg: WReg) -> Self {
        WRegWsp(reg.0)
    }
}

impl XReg {
    /// `oaknut::RReg::toW`.
    pub const fn to_w(self) -> WReg {
        WReg(self.0)
    }
}

impl WReg {
    /// `oaknut::RReg::toX`.
    pub const fn to_x(self) -> XReg {
        XReg(self.0)
    }
}

macro_rules! fp_reg {
    ($name:ident, $size:expr) => {
        impl FpReg for $name {
            const SIZE: u8 = $size;
            fn index(self) -> u8 {
                self.0
            }
        }
    };
}

fp_reg!(BReg, 0);
fp_reg!(HReg, 1);
fp_reg!(SReg, 2);
fp_reg!(DReg, 3);
fp_reg!(QReg, 4);

macro_rules! arranged {
    ($name:ident, $size:expr, $q:expr, $selector:ident) => {
        impl VRegArranged for $name {
            const SIZE: u8 = $size;
            const Q: bool = $q;
            fn index(self) -> u8 {
                self.0
            }
            fn from_vreg(reg: VReg) -> Self {
                $name(reg.0)
            }
        }

        impl VReg {
            /// oaknut's `VRegSelector` arrangement accessor.
            pub const fn $selector(self) -> $name {
                $name(self.0)
            }
        }
    };
}

arranged!(VReg8B, 8, false, b8);
arranged!(VReg16B, 8, true, b16);
arranged!(VReg4H, 16, false, h4);
arranged!(VReg8H, 16, true, h8);
arranged!(VReg2S, 32, false, s2);
arranged!(VReg4S, 32, true, s4);
arranged!(VReg1D, 64, false, d1);
arranged!(VReg2D, 64, true, d2);

impl VRegBytes for VReg8B {}
impl VRegBytes for VReg16B {}

impl VReg {
    pub const fn b(self) -> BReg {
        BReg(self.0)
    }
    pub const fn h(self) -> HReg {
        HReg(self.0)
    }
    pub const fn s(self) -> SReg {
        SReg(self.0)
    }
    pub const fn d(self) -> DReg {
        DReg(self.0)
    }
    pub const fn q(self) -> QReg {
        QReg(self.0)
    }
}

macro_rules! named_registers {
    ($type:ident, $prefix:ident: $($n:literal),*) => {
        paste::paste! {
            $(pub const [<$prefix $n>]: $type = $type::new($n);)*
        }
    };
}

// oaknut::util names. Register 31 has its own names below.
named_registers!(XReg, X: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30);
named_registers!(WReg, W: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30);
named_registers!(VReg, V: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
named_registers!(QReg, Q: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
named_registers!(DReg, D: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
named_registers!(SReg, S: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
named_registers!(HReg, H: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);
named_registers!(BReg, B: 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31);

pub const XZR: XReg = XReg::new(31);
pub const WZR: WReg = WReg::new(31);
pub const SP: XRegSp = XRegSp::new(31);
pub const WSP: WRegWsp = WRegWsp::new(31);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_31_is_zr_or_sp_by_type() {
        assert_eq!(XZR.index(), 31);
        assert_eq!(SP.index(), 31);
        assert_eq!(XRegSp::from(X3).index(), 3);
        assert_eq!(W7.to_x(), X7);
        assert_eq!(X7.to_w(), W7);
    }

    #[test]
    fn arrangements_carry_size_and_q() {
        assert_eq!(<VReg4S as VRegArranged>::SIZE, 32);
        assert!(<VReg4S as VRegArranged>::Q);
        assert!(!<VReg2S as VRegArranged>::Q);
        assert_eq!(V5.s4().index(), 5);
        assert_eq!(V5.q().index(), 5);
        assert_eq!(<QReg as FpReg>::SIZE, 4);
        assert_eq!(<VReg2D as VRegArranged>::from_vreg(V9), V9.d2());
    }
}
