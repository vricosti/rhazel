// SPDX-FileCopyrightText: Copyright 2023 yuzu Emulator Project
// SPDX-License-Identifier: GPL-2.0-or-later

//! AArch64 register types — mirrors oaknut's XReg, WReg, QReg, SystemReg.

/// 64-bit general-purpose register (X0–X30, XZR encoded as index 31).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct XReg(pub u8);

/// 32-bit general-purpose register (W0–W30, WZR encoded as index 31).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct WReg(pub u8);

/// 128-bit SIMD/FP register (Q0–Q31).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct QReg(pub u8);

impl XReg {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl WReg {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl QReg {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

// Convenience constants matching oaknut::util names.
pub const X0: XReg = XReg(0);
pub const X1: XReg = XReg(1);
pub const X2: XReg = XReg(2);
pub const X3: XReg = XReg(3);
pub const X4: XReg = XReg(4);
pub const X5: XReg = XReg(5);
pub const X6: XReg = XReg(6);
pub const X7: XReg = XReg(7);
pub const X8: XReg = XReg(8);
pub const X9: XReg = XReg(9);
pub const X10: XReg = XReg(10);
pub const X11: XReg = XReg(11);
pub const X12: XReg = XReg(12);
pub const X13: XReg = XReg(13);
pub const X14: XReg = XReg(14);
pub const X15: XReg = XReg(15);
pub const X16: XReg = XReg(16);
pub const X17: XReg = XReg(17);
pub const X18: XReg = XReg(18);
pub const X19: XReg = XReg(19);
pub const X20: XReg = XReg(20);
pub const X21: XReg = XReg(21);
pub const X22: XReg = XReg(22);
pub const X23: XReg = XReg(23);
pub const X24: XReg = XReg(24);
pub const X25: XReg = XReg(25);
pub const X26: XReg = XReg(26);
pub const X27: XReg = XReg(27);
pub const X28: XReg = XReg(28);
pub const X29: XReg = XReg(29);
pub const X30: XReg = XReg(30);
/// XZR — zero register, encoded as index 31 in most positions.
pub const XZR: XReg = XReg(31);
/// SP — stack pointer, encoded as index 31 in base-register positions.
/// Distinct from XZR only by context; we use the same index.
pub const SP: XReg = XReg(31);

pub const W0: WReg = WReg(0);
pub const W1: WReg = WReg(1);
pub const W2: WReg = WReg(2);
pub const W3: WReg = WReg(3);
pub const WZR: WReg = WReg(31);

pub const Q8: QReg = QReg(8);
pub const Q9: QReg = QReg(9);
pub const Q10: QReg = QReg(10);
pub const Q11: QReg = QReg(11);
pub const Q12: QReg = QReg(12);
pub const Q13: QReg = QReg(13);
pub const Q14: QReg = QReg(14);
pub const Q15: QReg = QReg(15);

/// AArch64 system registers — subset used by the NCE patcher.
///
/// Values match the AArch64 encoding `op0:op1:CRn:CRm:op2` packed into 15 bits,
/// as used in MRS/MSR instruction encoding (bits [20:5]).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SystemReg {
    /// TPIDR_EL0 — user read/write TLS register. Encoding: 3,3,13,0,2
    TpidrEl0 = 0b11_011_1101_0000_010,
    /// TPIDRRO_EL0 — user read-only TLS register. Encoding: 3,3,13,0,3
    TpidrroEl0 = 0b11_011_1101_0000_011,
    /// NZCV — condition flags. Encoding: 3,3,4,2,0
    Nzcv = 0b11_011_0100_0010_000,
    /// FPSR — floating-point status. Encoding: 3,3,4,4,1
    Fpsr = 0b11_011_0100_0100_001,
    /// FPCR — floating-point control. Encoding: 3,3,4,4,0
    Fpcr = 0b11_011_0100_0100_000,
    /// CNTVCT_EL0 — virtual counter. Encoding: 3,3,14,0,2
    CntvctEl0 = 0b11_011_1110_0000_010,
}

impl SystemReg {
    /// Returns the 15-bit system register encoding used in MRS/MSR instructions.
    pub fn encoding(self) -> u32 {
        self as u32
    }
}
