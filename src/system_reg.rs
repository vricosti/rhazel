//! System register names for `MRS`/`MSR`, the counterpart of
//! `oaknut::SystemReg` (`oaknut/impl/enum.hpp`), limited to the registers
//! the encoders in `inst.rs` cover.

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SystemReg {
    FPCR,
    FPSR,
    NZCV,
}
