//! Operand enumerations for `MRS`/`MSR` and the barrier instructions, the
//! counterpart of `oaknut/impl/enum.hpp` (`Cond` lives in `cond.rs`), limited
//! to the values the encoders in `inst.rs` cover.

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemReg {
    FPCR,
    FPSR,
    NZCV,
}

/// `DMB`/`DSB` option; the value is the instruction's `CRm` field.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BarrierOp {
    OSHLD = 0b0001,
    OSHST = 0b0010,
    OSH = 0b0011,
    NSHLD = 0b0101,
    NSHST = 0b0110,
    NSH = 0b0111,
    ISHLD = 0b1001,
    ISHST = 0b1010,
    ISH = 0b1011,
    LD = 0b1101,
    ST = 0b1110,
    SY = 0b1111,
}
