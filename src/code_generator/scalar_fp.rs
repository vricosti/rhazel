//! Scalar FP overloads of Oaknut's `impl/mnemonics_fpsimd_v8.0.inc.hpp`.
//!
//! The `_fp` suffix separates scalar operands from the vector overloads.
//! Register allocation and FPCR/FPSR lifecycle remain in Dynarmic's emitter.

use super::CodeGenerator;
use crate::inst;
use crate::reg::{DReg, FpReg, GpReg, HReg, SReg, WReg};

mod sealed {
    pub trait Scalar {}
    impl Scalar for crate::reg::SReg {}
    impl Scalar for crate::reg::DReg {}
}

/// The S/D overload set used by scalar arithmetic and GP conversions.
pub trait ScalarFp: FpReg + sealed::Scalar {
    const DOUBLE: bool;
}
impl ScalarFp for SReg {
    const DOUBLE: bool = false;
}
impl ScalarFp for DReg {
    const DOUBLE: bool = true;
}

impl CodeGenerator<'_> {
    /// Oaknut `FCMP`, scalar form.
    pub fn fcmp_zero_fp<F: ScalarFp>(&mut self, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fcmp_s_zero(rn.index()),
            true => inst::fcmp_d_zero(rn.index()),
        })
    }

    /// Oaknut `FCMPE`, scalar form.
    pub fn fcmpe_zero_fp<F: ScalarFp>(&mut self, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fcmpe_s_zero(rn.index()),
            true => inst::fcmpe_d_zero(rn.index()),
        })
    }

    /// Oaknut `FCMP`, scalar form.
    pub fn fcmp_fp<F: ScalarFp>(&mut self, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fcmp_s(rn.index(), rm.index()),
            true => inst::fcmp_d(rn.index(), rm.index()),
        })
    }

    /// Oaknut `FCMPE`, scalar form.
    pub fn fcmpe_fp<F: ScalarFp>(&mut self, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fcmpe_s(rn.index(), rm.index()),
            true => inst::fcmpe_d(rn.index(), rm.index()),
        })
    }

    /// Oaknut `FCVTZU`, scaled fixed-point form.
    pub fn fcvtzu_fixed_fp<G: GpReg, F: ScalarFp>(
        &mut self,
        rd: G,
        rn: F,
        fbits: u8,
    ) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtzu_w_from_s_fixed(rd.index(), rn.index(), fbits),
            (false, true) => inst::fcvtzu_w_from_d_fixed(rd.index(), rn.index(), fbits),
            (true, false) => inst::fcvtzu_x_from_s_fixed(rd.index(), rn.index(), fbits),
            (true, true) => inst::fcvtzu_x_from_d_fixed(rd.index(), rn.index(), fbits),
        })
    }

    /// Oaknut `FCVTZS`, scaled fixed-point form.
    pub fn fcvtzs_fixed_fp<G: GpReg, F: ScalarFp>(
        &mut self,
        rd: G,
        rn: F,
        fbits: u8,
    ) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtzs_w_from_s_fixed(rd.index(), rn.index(), fbits),
            (false, true) => inst::fcvtzs_w_from_d_fixed(rd.index(), rn.index(), fbits),
            (true, false) => inst::fcvtzs_x_from_s_fixed(rd.index(), rn.index(), fbits),
            (true, true) => inst::fcvtzs_x_from_d_fixed(rd.index(), rn.index(), fbits),
        })
    }

    /// Oaknut `FCVTNU`, scalar form.
    pub fn fcvtnu_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtnu_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtnu_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtnu_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtnu_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTPU`, scalar form.
    pub fn fcvtpu_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtpu_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtpu_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtpu_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtpu_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTMU`, scalar form.
    pub fn fcvtmu_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtmu_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtmu_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtmu_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtmu_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTZU`, scalar form.
    pub fn fcvtzu_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtzu_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtzu_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtzu_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtzu_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTAU`, scalar form.
    pub fn fcvtau_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtau_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtau_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtau_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtau_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTNS`, scalar form.
    pub fn fcvtns_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtns_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtns_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtns_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtns_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTPS`, scalar form.
    pub fn fcvtps_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtps_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtps_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtps_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtps_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTMS`, scalar form.
    pub fn fcvtms_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtms_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtms_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtms_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtms_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTZS`, scalar form.
    pub fn fcvtzs_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtzs_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtzs_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtzs_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtzs_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVTAS`, scalar form.
    pub fn fcvtas_fp<G: GpReg, F: ScalarFp>(&mut self, rd: G, rn: F) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::fcvtas_w_from_s(rd.index(), rn.index()),
            (false, true) => inst::fcvtas_w_from_d(rd.index(), rn.index()),
            (true, false) => inst::fcvtas_x_from_s(rd.index(), rn.index()),
            (true, true) => inst::fcvtas_x_from_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `ADD(Wd, Wn, Wm, LSR, shift)`.
    pub fn add_lsr_fp(&mut self, rd: WReg, rn: WReg, rm: WReg, shift: u8) -> Result<(), String> {
        self.emit(inst::add_w_reg_lsr(
            rd.index(),
            rn.index(),
            rm.index(),
            shift,
        ))
    }

    /// Oaknut `LSR(Wd, Wn, shift)`.
    pub fn lsr_fp(&mut self, rd: WReg, rn: WReg, shift: u8) -> Result<(), String> {
        self.emit(inst::lsr_w_imm(rd.index(), rn.index(), shift))
    }

    /// Oaknut `UCVTF`, scalar form.
    pub fn ucvtf_fp<G: GpReg, F: ScalarFp>(&mut self, rd: F, rn: G) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::ucvtf_s_from_w(rd.index(), rn.index()),
            (false, true) => inst::ucvtf_d_from_w(rd.index(), rn.index()),
            (true, false) => inst::ucvtf_s_from_x(rd.index(), rn.index()),
            (true, true) => inst::ucvtf_d_from_x(rd.index(), rn.index()),
        })
    }

    /// Oaknut `SCVTF`, scalar form.
    pub fn scvtf_fp<G: GpReg, F: ScalarFp>(&mut self, rd: F, rn: G) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::scvtf_s_from_w(rd.index(), rn.index()),
            (false, true) => inst::scvtf_d_from_w(rd.index(), rn.index()),
            (true, false) => inst::scvtf_s_from_x(rd.index(), rn.index()),
            (true, true) => inst::scvtf_d_from_x(rd.index(), rn.index()),
        })
    }

    /// Oaknut `UCVTF`, scaled fixed-point form.
    pub fn ucvtf_fixed_fp<G: GpReg, F: ScalarFp>(
        &mut self,
        rd: F,
        rn: G,
        fbits: u8,
    ) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::ucvtf_s_from_w_fixed(rd.index(), rn.index(), fbits),
            (false, true) => inst::ucvtf_d_from_w_fixed(rd.index(), rn.index(), fbits),
            (true, false) => inst::ucvtf_s_from_x_fixed(rd.index(), rn.index(), fbits),
            (true, true) => inst::ucvtf_d_from_x_fixed(rd.index(), rn.index(), fbits),
        })
    }

    /// Oaknut `SCVTF`, scaled fixed-point form.
    pub fn scvtf_fixed_fp<G: GpReg, F: ScalarFp>(
        &mut self,
        rd: F,
        rn: G,
        fbits: u8,
    ) -> Result<(), String> {
        self.emit(match (G::SF, F::DOUBLE) {
            (false, false) => inst::scvtf_s_from_w_fixed(rd.index(), rn.index(), fbits),
            (false, true) => inst::scvtf_d_from_w_fixed(rd.index(), rn.index(), fbits),
            (true, false) => inst::scvtf_s_from_x_fixed(rd.index(), rn.index(), fbits),
            (true, true) => inst::scvtf_d_from_x_fixed(rd.index(), rn.index(), fbits),
        })
    }

    /// Oaknut `FMUL`, scalar form.
    pub fn fmul_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmul_s(rd.index(), rn.index(), rm.index()),
            true => inst::fmul_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FMULX`, scalar form.
    pub fn fmulx_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmulx_s(rd.index(), rn.index(), rm.index()),
            true => inst::fmulx_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FADD`, scalar form.
    pub fn fadd_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fadd_s(rd.index(), rn.index(), rm.index()),
            true => inst::fadd_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FSUB`, scalar form.
    pub fn fsub_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fsub_s(rd.index(), rn.index(), rm.index()),
            true => inst::fsub_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FDIV`, scalar form.
    pub fn fdiv_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fdiv_s(rd.index(), rn.index(), rm.index()),
            true => inst::fdiv_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FABS`, scalar form.
    pub fn fabs_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fabs_s(rd.index(), rn.index()),
            true => inst::fabs_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FMAXNM`, scalar form.
    pub fn fmaxnm_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmaxnm_s(rd.index(), rn.index(), rm.index()),
            true => inst::fmaxnm_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FMAX`, scalar form.
    pub fn fmax_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmax_s(rd.index(), rn.index(), rm.index()),
            true => inst::fmax_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FMADD`, scalar form.
    pub fn fmadd_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F, ra: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmadd_s(rd.index(), rn.index(), rm.index(), ra.index()),
            true => inst::fmadd_d(rd.index(), rn.index(), rm.index(), ra.index()),
        })
    }

    /// Oaknut `FMSUB`, scalar form.
    pub fn fmsub_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F, ra: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmsub_s(rd.index(), rn.index(), rm.index(), ra.index()),
            true => inst::fmsub_d(rd.index(), rn.index(), rm.index(), ra.index()),
        })
    }

    /// Oaknut `FMINNM`, scalar form.
    pub fn fminnm_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fminnm_s(rd.index(), rn.index(), rm.index()),
            true => inst::fminnm_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FMIN`, scalar form.
    pub fn fmin_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fmin_s(rd.index(), rn.index(), rm.index()),
            true => inst::fmin_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FNEG`, scalar form.
    pub fn fneg_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fneg_s(rd.index(), rn.index()),
            true => inst::fneg_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRECPE`, scalar form.
    pub fn frecpe_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frecpe_s(rd.index(), rn.index()),
            true => inst::frecpe_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRECPX`, scalar form.
    pub fn frecpx_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frecpx_s(rd.index(), rn.index()),
            true => inst::frecpx_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRECPS`, scalar form.
    pub fn frecps_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frecps_s(rd.index(), rn.index(), rm.index()),
            true => inst::frecps_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FRSQRTE`, scalar form.
    pub fn frsqrte_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frsqrte_s(rd.index(), rn.index()),
            true => inst::frsqrte_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRSQRTS`, scalar form.
    pub fn frsqrts_fp<F: ScalarFp>(&mut self, rd: F, rn: F, rm: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frsqrts_s(rd.index(), rn.index(), rm.index()),
            true => inst::frsqrts_d(rd.index(), rn.index(), rm.index()),
        })
    }

    /// Oaknut `FRINTX`, scalar form.
    pub fn frintx_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frintx_s(rd.index(), rn.index()),
            true => inst::frintx_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRINTN`, scalar form.
    pub fn frintn_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frintn_s(rd.index(), rn.index()),
            true => inst::frintn_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRINTP`, scalar form.
    pub fn frintp_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frintp_s(rd.index(), rn.index()),
            true => inst::frintp_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRINTM`, scalar form.
    pub fn frintm_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frintm_s(rd.index(), rn.index()),
            true => inst::frintm_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRINTZ`, scalar form.
    pub fn frintz_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frintz_s(rd.index(), rn.index()),
            true => inst::frintz_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FRINTA`, scalar form.
    pub fn frinta_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::frinta_s(rd.index(), rn.index()),
            true => inst::frinta_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FSQRT`, scalar form.
    pub fn fsqrt_fp<F: ScalarFp>(&mut self, rd: F, rn: F) -> Result<(), String> {
        self.emit(match F::DOUBLE {
            false => inst::fsqrt_s(rd.index(), rn.index()),
            true => inst::fsqrt_d(rd.index(), rn.index()),
        })
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_d_from_s_fp(&mut self, rd: DReg, rn: SReg) -> Result<(), String> {
        self.emit(inst::fcvt_d_from_s(rd.index(), rn.index()))
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_s_from_h_fp(&mut self, rd: SReg, rn: HReg) -> Result<(), String> {
        self.emit(inst::fcvt_s_from_h(rd.index(), rn.index()))
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_d_from_h_fp(&mut self, rd: DReg, rn: HReg) -> Result<(), String> {
        self.emit(inst::fcvt_d_from_h(rd.index(), rn.index()))
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_h_from_s_fp(&mut self, rd: HReg, rn: SReg) -> Result<(), String> {
        self.emit(inst::fcvt_h_from_s(rd.index(), rn.index()))
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_h_from_d_fp(&mut self, rd: HReg, rn: DReg) -> Result<(), String> {
        self.emit(inst::fcvt_h_from_d(rd.index(), rn.index()))
    }

    /// Oaknut `FCVTXN`, scalar form.
    pub fn fcvtxn_s_from_d_fp(&mut self, rd: SReg, rn: DReg) -> Result<(), String> {
        self.emit(inst::fcvtxn_s_from_d(rd.index(), rn.index()))
    }

    /// Oaknut `FCVT`, scalar form.
    pub fn fcvt_s_from_d_fp(&mut self, rd: SReg, rn: DReg) -> Result<(), String> {
        self.emit(inst::fcvt_s_from_d(rd.index(), rn.index()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_of_code::BlockOfCode;
    use crate::reg::*;

    fn words(block: &BlockOfCode) -> Vec<u32> {
        (0..block.code_size() / 4)
            .map(|i| unsafe { (block.code_base_ptr().add(i * 4) as *const u32).read_unaligned() })
            .collect()
    }

    // Constants independently transcribed from Oaknut v2.0.3 scalar bit patterns.

    #[test]
    fn register_edges_and_signed16_truncation_match_oaknut() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.fadd_fp(S31, S0, S31).unwrap();
        code.fmadd_fp(D31, D30, D29, D28).unwrap();
        code.fcvt_h_from_d_fp(H31, D0).unwrap();
        code.fcvtxn_s_from_d_fp(S0, D31).unwrap();
        code.fcvtzs_fixed_fp(W1, S2, 16).unwrap();
        code.asr(W16, W1, 31).unwrap();
        code.add_lsr_fp(W1, W1, W16, 16).unwrap();
        code.lsr_fp(W1, W1, 16).unwrap();
        code.add_lsr_fp(WZR, W0, WZR, 0).unwrap();
        code.add_lsr_fp(W0, WZR, W0, 31).unwrap();
        code.lsr_fp(WZR, W0, 0).unwrap();
        code.lsr_fp(W0, WZR, 31).unwrap();
        assert_eq!(
            words(&block),
            [
                0x1e3f281f, 0x1f5d73df, 0x1e63c01f, 0x7e616be0, 0x1e18c041, 0x131f7c30, 0x0b504021,
                0x53107c21, 0x0b5f001f, 0x0b407fe0, 0x53007c1f, 0x531f7fe0,
            ]
        );
    }

    #[test]
    fn fixed_scales_reject_zero_and_out_of_range() {
        use std::panic::{catch_unwind, AssertUnwindSafe};
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzs_fixed_fp(W1, S2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzs_fixed_fp(W1, D2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzs_fixed_fp(X1, S2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzs_fixed_fp(X1, D2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzu_fixed_fp(W1, S2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzu_fixed_fp(W1, D2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzu_fixed_fp(X1, S2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.fcvtzu_fixed_fp(X1, D2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.scvtf_fixed_fp(S1, W2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.scvtf_fixed_fp(D1, W2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.scvtf_fixed_fp(S1, X2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.scvtf_fixed_fp(D1, X2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.ucvtf_fixed_fp(S1, W2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 33] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.ucvtf_fixed_fp(D1, W2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.ucvtf_fixed_fp(S1, X2, fbits).unwrap();
            }))
            .is_err());
        }
        for fbits in [0, 65] {
            assert!(catch_unwind(AssertUnwindSafe(|| {
                code.ucvtf_fixed_fp(D1, X2, fbits).unwrap();
            }))
            .is_err());
        }
        assert_eq!(block.code_size(), 0);
    }

    #[test]
    fn integer_shift_helpers_reject_32() {
        use std::panic::{catch_unwind, AssertUnwindSafe};
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        assert!(catch_unwind(AssertUnwindSafe(|| {
            code.add_lsr_fp(W0, W1, W2, 32).unwrap();
        }))
        .is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| {
            code.lsr_fp(W0, W1, 32).unwrap();
        }))
        .is_err());
        assert_eq!(block.code_size(), 0);
    }

    // Constants independently transcribed from Oaknut v2.0.3 scalar bit patterns.
    // Every migrated scalar overload is exercised; scales cover 1 and W/X maxima.
    #[test]
    fn scalar_overloads_match_oaknut_words() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut code = CodeGenerator::new(&mut block);
        code.fcmp_zero_fp(S2).unwrap();
        code.fcmpe_zero_fp(S2).unwrap();
        code.fcmp_zero_fp(D2).unwrap();
        code.fcmpe_zero_fp(D2).unwrap();
        code.fcmp_fp(S2, S3).unwrap();
        code.fcmpe_fp(S2, S3).unwrap();
        code.fcmp_fp(D2, D3).unwrap();
        code.fcmpe_fp(D2, D3).unwrap();
        code.fcvtzu_fixed_fp(W1, S2, 1).unwrap();
        code.fcvtzu_fixed_fp(W1, S2, 32).unwrap();
        code.fcvtzu_fixed_fp(W1, D2, 1).unwrap();
        code.fcvtzu_fixed_fp(W1, D2, 32).unwrap();
        code.fcvtzs_fixed_fp(W1, S2, 1).unwrap();
        code.fcvtzs_fixed_fp(W1, S2, 32).unwrap();
        code.fcvtzs_fixed_fp(W1, D2, 1).unwrap();
        code.fcvtzs_fixed_fp(W1, D2, 32).unwrap();
        code.fcvtnu_fp(W1, S2).unwrap();
        code.fcvtnu_fp(W1, D2).unwrap();
        code.fcvtpu_fp(W1, S2).unwrap();
        code.fcvtpu_fp(W1, D2).unwrap();
        code.fcvtmu_fp(W1, S2).unwrap();
        code.fcvtmu_fp(W1, D2).unwrap();
        code.fcvtzu_fp(W1, S2).unwrap();
        code.fcvtzu_fp(W1, D2).unwrap();
        code.fcvtau_fp(W1, S2).unwrap();
        code.fcvtau_fp(W1, D2).unwrap();
        code.fcvtns_fp(W1, S2).unwrap();
        code.fcvtns_fp(W1, D2).unwrap();
        code.fcvtps_fp(W1, S2).unwrap();
        code.fcvtps_fp(W1, D2).unwrap();
        code.fcvtms_fp(W1, S2).unwrap();
        code.fcvtms_fp(W1, D2).unwrap();
        code.fcvtzs_fp(W1, S2).unwrap();
        code.fcvtzs_fp(W1, D2).unwrap();
        code.fcvtas_fp(W1, S2).unwrap();
        code.fcvtas_fp(W1, D2).unwrap();
        code.ucvtf_fp(S1, W2).unwrap();
        code.ucvtf_fp(D1, W2).unwrap();
        code.scvtf_fp(S1, W2).unwrap();
        code.scvtf_fp(D1, W2).unwrap();
        code.ucvtf_fixed_fp(S1, W2, 1).unwrap();
        code.ucvtf_fixed_fp(S1, W2, 32).unwrap();
        code.ucvtf_fixed_fp(D1, W2, 1).unwrap();
        code.ucvtf_fixed_fp(D1, W2, 32).unwrap();
        code.scvtf_fixed_fp(S1, W2, 1).unwrap();
        code.scvtf_fixed_fp(S1, W2, 32).unwrap();
        code.scvtf_fixed_fp(D1, W2, 1).unwrap();
        code.scvtf_fixed_fp(D1, W2, 32).unwrap();
        code.fcvtnu_fp(X1, S2).unwrap();
        code.fcvtnu_fp(X1, D2).unwrap();
        code.fcvtpu_fp(X1, S2).unwrap();
        code.fcvtpu_fp(X1, D2).unwrap();
        code.fcvtmu_fp(X1, S2).unwrap();
        code.fcvtmu_fp(X1, D2).unwrap();
        code.fcvtzu_fp(X1, S2).unwrap();
        code.fcvtzu_fp(X1, D2).unwrap();
        code.fcvtau_fp(X1, S2).unwrap();
        code.fcvtau_fp(X1, D2).unwrap();
        code.fcvtns_fp(X1, S2).unwrap();
        code.fcvtns_fp(X1, D2).unwrap();
        code.fcvtps_fp(X1, S2).unwrap();
        code.fcvtps_fp(X1, D2).unwrap();
        code.fcvtms_fp(X1, S2).unwrap();
        code.fcvtms_fp(X1, D2).unwrap();
        code.fcvtzs_fp(X1, S2).unwrap();
        code.fcvtzs_fp(X1, D2).unwrap();
        code.fcvtas_fp(X1, S2).unwrap();
        code.fcvtas_fp(X1, D2).unwrap();
        code.fcvtzu_fixed_fp(X1, S2, 1).unwrap();
        code.fcvtzu_fixed_fp(X1, S2, 64).unwrap();
        code.fcvtzu_fixed_fp(X1, D2, 1).unwrap();
        code.fcvtzu_fixed_fp(X1, D2, 64).unwrap();
        code.fcvtzs_fixed_fp(X1, S2, 1).unwrap();
        code.fcvtzs_fixed_fp(X1, S2, 64).unwrap();
        code.fcvtzs_fixed_fp(X1, D2, 1).unwrap();
        code.fcvtzs_fixed_fp(X1, D2, 64).unwrap();
        code.ucvtf_fp(S1, X2).unwrap();
        code.ucvtf_fp(D1, X2).unwrap();
        code.scvtf_fp(S1, X2).unwrap();
        code.scvtf_fp(D1, X2).unwrap();
        code.ucvtf_fixed_fp(S1, X2, 1).unwrap();
        code.ucvtf_fixed_fp(S1, X2, 64).unwrap();
        code.ucvtf_fixed_fp(D1, X2, 1).unwrap();
        code.ucvtf_fixed_fp(D1, X2, 64).unwrap();
        code.scvtf_fixed_fp(S1, X2, 1).unwrap();
        code.scvtf_fixed_fp(S1, X2, 64).unwrap();
        code.scvtf_fixed_fp(D1, X2, 1).unwrap();
        code.scvtf_fixed_fp(D1, X2, 64).unwrap();
        code.fmul_fp(S1, S2, S3).unwrap();
        code.fmul_fp(D1, D2, D3).unwrap();
        code.fmulx_fp(S1, S2, S3).unwrap();
        code.fmulx_fp(D1, D2, D3).unwrap();
        code.fadd_fp(S1, S2, S3).unwrap();
        code.fadd_fp(D1, D2, D3).unwrap();
        code.fsub_fp(S1, S2, S3).unwrap();
        code.fsub_fp(D1, D2, D3).unwrap();
        code.fdiv_fp(S1, S2, S3).unwrap();
        code.fdiv_fp(D1, D2, D3).unwrap();
        code.fabs_fp(S1, S2).unwrap();
        code.fabs_fp(D1, D2).unwrap();
        code.fmaxnm_fp(S1, S2, S3).unwrap();
        code.fmax_fp(S1, S2, S3).unwrap();
        code.fmaxnm_fp(D1, D2, D3).unwrap();
        code.fmax_fp(D1, D2, D3).unwrap();
        code.fmadd_fp(S1, S2, S3, S4).unwrap();
        code.fmadd_fp(D1, D2, D3, D4).unwrap();
        code.fmsub_fp(S1, S2, S3, S4).unwrap();
        code.fmsub_fp(D1, D2, D3, D4).unwrap();
        code.fminnm_fp(S1, S2, S3).unwrap();
        code.fmin_fp(S1, S2, S3).unwrap();
        code.fminnm_fp(D1, D2, D3).unwrap();
        code.fmin_fp(D1, D2, D3).unwrap();
        code.fneg_fp(S1, S2).unwrap();
        code.fneg_fp(D1, D2).unwrap();
        code.frecpe_fp(S1, S2).unwrap();
        code.frecpe_fp(D1, D2).unwrap();
        code.frecpx_fp(S1, S2).unwrap();
        code.frecpx_fp(D1, D2).unwrap();
        code.frecps_fp(S1, S2, S3).unwrap();
        code.frecps_fp(D1, D2, D3).unwrap();
        code.frsqrte_fp(S1, S2).unwrap();
        code.frsqrte_fp(D1, D2).unwrap();
        code.frsqrts_fp(S1, S2, S3).unwrap();
        code.frsqrts_fp(D1, D2, D3).unwrap();
        code.frintx_fp(S1, S2).unwrap();
        code.frintx_fp(D1, D2).unwrap();
        code.frintn_fp(S1, S2).unwrap();
        code.frintn_fp(D1, D2).unwrap();
        code.frintp_fp(S1, S2).unwrap();
        code.frintp_fp(D1, D2).unwrap();
        code.frintm_fp(S1, S2).unwrap();
        code.frintm_fp(D1, D2).unwrap();
        code.frintz_fp(S1, S2).unwrap();
        code.frintz_fp(D1, D2).unwrap();
        code.frinta_fp(S1, S2).unwrap();
        code.frinta_fp(D1, D2).unwrap();
        code.fsqrt_fp(S1, S2).unwrap();
        code.fsqrt_fp(D1, D2).unwrap();
        code.fcvt_d_from_s_fp(D1, S2).unwrap();
        code.fcvt_s_from_h_fp(S1, H2).unwrap();
        code.fcvt_d_from_h_fp(D1, H2).unwrap();
        code.fcvt_h_from_s_fp(H1, S2).unwrap();
        code.fcvt_h_from_d_fp(H1, D2).unwrap();
        code.fcvtxn_s_from_d_fp(S1, D2).unwrap();
        code.fcvt_s_from_d_fp(S1, D2).unwrap();
        assert_eq!(
            words(&block),
            [
                0x1e202048, // fcmp_s_zero
                0x1e202058, // fcmpe_s_zero
                0x1e602048, // fcmp_d_zero
                0x1e602058, // fcmpe_d_zero
                0x1e232040, // fcmp_s
                0x1e232050, // fcmpe_s
                0x1e632040, // fcmp_d
                0x1e632050, // fcmpe_d
                0x1e19fc41, // fcvtzu_w_from_s_fixed #1
                0x1e198041, // fcvtzu_w_from_s_fixed #32
                0x1e59fc41, // fcvtzu_w_from_d_fixed #1
                0x1e598041, // fcvtzu_w_from_d_fixed #32
                0x1e18fc41, // fcvtzs_w_from_s_fixed #1
                0x1e188041, // fcvtzs_w_from_s_fixed #32
                0x1e58fc41, // fcvtzs_w_from_d_fixed #1
                0x1e588041, // fcvtzs_w_from_d_fixed #32
                0x1e210041, // fcvtnu_w_from_s
                0x1e610041, // fcvtnu_w_from_d
                0x1e290041, // fcvtpu_w_from_s
                0x1e690041, // fcvtpu_w_from_d
                0x1e310041, // fcvtmu_w_from_s
                0x1e710041, // fcvtmu_w_from_d
                0x1e390041, // fcvtzu_w_from_s
                0x1e790041, // fcvtzu_w_from_d
                0x1e250041, // fcvtau_w_from_s
                0x1e650041, // fcvtau_w_from_d
                0x1e200041, // fcvtns_w_from_s
                0x1e600041, // fcvtns_w_from_d
                0x1e280041, // fcvtps_w_from_s
                0x1e680041, // fcvtps_w_from_d
                0x1e300041, // fcvtms_w_from_s
                0x1e700041, // fcvtms_w_from_d
                0x1e380041, // fcvtzs_w_from_s
                0x1e780041, // fcvtzs_w_from_d
                0x1e240041, // fcvtas_w_from_s
                0x1e640041, // fcvtas_w_from_d
                0x1e230041, // ucvtf_s_from_w
                0x1e630041, // ucvtf_d_from_w
                0x1e220041, // scvtf_s_from_w
                0x1e620041, // scvtf_d_from_w
                0x1e03fc41, // ucvtf_s_from_w_fixed #1
                0x1e038041, // ucvtf_s_from_w_fixed #32
                0x1e43fc41, // ucvtf_d_from_w_fixed #1
                0x1e438041, // ucvtf_d_from_w_fixed #32
                0x1e02fc41, // scvtf_s_from_w_fixed #1
                0x1e028041, // scvtf_s_from_w_fixed #32
                0x1e42fc41, // scvtf_d_from_w_fixed #1
                0x1e428041, // scvtf_d_from_w_fixed #32
                0x9e210041, // fcvtnu_x_from_s
                0x9e610041, // fcvtnu_x_from_d
                0x9e290041, // fcvtpu_x_from_s
                0x9e690041, // fcvtpu_x_from_d
                0x9e310041, // fcvtmu_x_from_s
                0x9e710041, // fcvtmu_x_from_d
                0x9e390041, // fcvtzu_x_from_s
                0x9e790041, // fcvtzu_x_from_d
                0x9e250041, // fcvtau_x_from_s
                0x9e650041, // fcvtau_x_from_d
                0x9e200041, // fcvtns_x_from_s
                0x9e600041, // fcvtns_x_from_d
                0x9e280041, // fcvtps_x_from_s
                0x9e680041, // fcvtps_x_from_d
                0x9e300041, // fcvtms_x_from_s
                0x9e700041, // fcvtms_x_from_d
                0x9e380041, // fcvtzs_x_from_s
                0x9e780041, // fcvtzs_x_from_d
                0x9e240041, // fcvtas_x_from_s
                0x9e640041, // fcvtas_x_from_d
                0x9e19fc41, // fcvtzu_x_from_s_fixed #1
                0x9e190041, // fcvtzu_x_from_s_fixed #64
                0x9e59fc41, // fcvtzu_x_from_d_fixed #1
                0x9e590041, // fcvtzu_x_from_d_fixed #64
                0x9e18fc41, // fcvtzs_x_from_s_fixed #1
                0x9e180041, // fcvtzs_x_from_s_fixed #64
                0x9e58fc41, // fcvtzs_x_from_d_fixed #1
                0x9e580041, // fcvtzs_x_from_d_fixed #64
                0x9e230041, // ucvtf_s_from_x
                0x9e630041, // ucvtf_d_from_x
                0x9e220041, // scvtf_s_from_x
                0x9e620041, // scvtf_d_from_x
                0x9e03fc41, // ucvtf_s_from_x_fixed #1
                0x9e030041, // ucvtf_s_from_x_fixed #64
                0x9e43fc41, // ucvtf_d_from_x_fixed #1
                0x9e430041, // ucvtf_d_from_x_fixed #64
                0x9e02fc41, // scvtf_s_from_x_fixed #1
                0x9e020041, // scvtf_s_from_x_fixed #64
                0x9e42fc41, // scvtf_d_from_x_fixed #1
                0x9e420041, // scvtf_d_from_x_fixed #64
                0x1e230841, // fmul_s
                0x1e630841, // fmul_d
                0x5e23dc41, // fmulx_s
                0x5e63dc41, // fmulx_d
                0x1e232841, // fadd_s
                0x1e632841, // fadd_d
                0x1e233841, // fsub_s
                0x1e633841, // fsub_d
                0x1e231841, // fdiv_s
                0x1e631841, // fdiv_d
                0x1e20c041, // fabs_s
                0x1e60c041, // fabs_d
                0x1e236841, // fmaxnm_s
                0x1e234841, // fmax_s
                0x1e636841, // fmaxnm_d
                0x1e634841, // fmax_d
                0x1f031041, // fmadd_s
                0x1f431041, // fmadd_d
                0x1f039041, // fmsub_s
                0x1f439041, // fmsub_d
                0x1e237841, // fminnm_s
                0x1e235841, // fmin_s
                0x1e637841, // fminnm_d
                0x1e635841, // fmin_d
                0x1e214041, // fneg_s
                0x1e614041, // fneg_d
                0x5ea1d841, // frecpe_s
                0x5ee1d841, // frecpe_d
                0x5ea1f841, // frecpx_s
                0x5ee1f841, // frecpx_d
                0x5e23fc41, // frecps_s
                0x5e63fc41, // frecps_d
                0x7ea1d841, // frsqrte_s
                0x7ee1d841, // frsqrte_d
                0x5ea3fc41, // frsqrts_s
                0x5ee3fc41, // frsqrts_d
                0x1e274041, // frintx_s
                0x1e674041, // frintx_d
                0x1e244041, // frintn_s
                0x1e644041, // frintn_d
                0x1e24c041, // frintp_s
                0x1e64c041, // frintp_d
                0x1e254041, // frintm_s
                0x1e654041, // frintm_d
                0x1e25c041, // frintz_s
                0x1e65c041, // frintz_d
                0x1e264041, // frinta_s
                0x1e664041, // frinta_d
                0x1e21c041, // fsqrt_s
                0x1e61c041, // fsqrt_d
                0x1e22c041, // fcvt_d_from_s
                0x1ee24041, // fcvt_s_from_h
                0x1ee2c041, // fcvt_d_from_h
                0x1e23c041, // fcvt_h_from_s
                0x1e63c041, // fcvt_h_from_d
                0x7e616841, // fcvtxn_s_from_d
                0x1e624041, // fcvt_s_from_d
            ]
        );
    }
}
