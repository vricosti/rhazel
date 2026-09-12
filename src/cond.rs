//! AArch64 condition codes, `oaknut::Cond`'s counterpart.
//!
//! The variants carry the architectural 4-bit encoding, the same values as
//! rdynarmic's `IR::Cond`; the backend converts between the two at its
//! boundary, as upstream does with `static_cast<oaknut::Cond>`.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Cond {
    EQ = 0,
    NE = 1,
    CS = 2,
    CC = 3,
    MI = 4,
    PL = 5,
    VS = 6,
    VC = 7,
    HI = 8,
    LS = 9,
    GE = 10,
    LT = 11,
    GT = 12,
    LE = 13,
    AL = 14,
    NV = 15,
}

impl Cond {
    pub const HS: Cond = Cond::CS;
    pub const LO: Cond = Cond::CC;

    /// Invert the condition code (`oaknut::invert`).
    pub fn invert(self) -> Cond {
        Cond::from_u8(self as u8 ^ 1)
    }

    /// Create from the raw 4-bit encoding.
    pub fn from_u8(val: u8) -> Cond {
        match val & 0xF {
            0 => Cond::EQ, 1 => Cond::NE, 2 => Cond::CS, 3 => Cond::CC,
            4 => Cond::MI, 5 => Cond::PL, 6 => Cond::VS, 7 => Cond::VC,
            8 => Cond::HI, 9 => Cond::LS, 10 => Cond::GE, 11 => Cond::LT,
            12 => Cond::GT, 13 => Cond::LE, 14 => Cond::AL, _ => Cond::NV,
        }
    }
}

impl fmt::Display for Cond {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Cond::EQ => "eq", Cond::NE => "ne", Cond::CS => "cs", Cond::CC => "cc",
            Cond::MI => "mi", Cond::PL => "pl", Cond::VS => "vs", Cond::VC => "vc",
            Cond::HI => "hi", Cond::LS => "ls", Cond::GE => "ge", Cond::LT => "lt",
            Cond::GT => "gt", Cond::LE => "le", Cond::AL => "al", Cond::NV => "nv",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Cond;

    #[test]
    fn invert_flips_the_low_bit_and_round_trips() {
        assert_eq!(Cond::EQ.invert(), Cond::NE);
        assert_eq!(Cond::LT.invert(), Cond::GE);
        for raw in 0..16u8 {
            assert_eq!(Cond::from_u8(raw) as u8, raw);
        }
    }
}
