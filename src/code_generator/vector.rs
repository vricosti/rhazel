//! Integer SIMD mnemonic extensions.
//! Upstream forms: oaknut/impl/mnemonics_fpsimd_v8.0.inc.hpp.
//! Shared/scalar FP mnemonics remain in the parent generator module.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn integer_simd_arrangements_match_oaknut_words() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        // Literal words are derived from Oaknut v2.0.3's per-arrangement
        // emit templates, not from rhazel's encoders.
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
        check!(abs_v(V29.b8(), V30.b8()), 0x0e20bbdd);
        check!(abs_v(V29.b16(), V30.b16()), 0x4e20bbdd);
        check!(abs_v(V29.h4(), V30.h4()), 0x0e60bbdd);
        check!(abs_v(V29.h8(), V30.h8()), 0x4e60bbdd);
        check!(abs_v(V29.s2(), V30.s2()), 0x0ea0bbdd);
        check!(abs_v(V29.s4(), V30.s4()), 0x4ea0bbdd);
        check!(abs_v(V29.d2(), V30.d2()), 0x4ee0bbdd);
        check!(addp_scalar(D29, V30.d2()), 0x5ef1bbdd);
        check!(addp_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3fbfdd);
        check!(addp_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3fbfdd);
        check!(addp_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7fbfdd);
        check!(addp_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7fbfdd);
        check!(addp_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebfbfdd);
        check!(addp_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebfbfdd);
        check!(addp_v(V29.d2(), V30.d2(), V31.d2()), 0x4effbfdd);
        check!(addv_scalar(B29, V30.b16()), 0x4e31bbdd);
        check!(addv_scalar(H29, V30.h8()), 0x4e71bbdd);
        check!(addv_scalar(S29, V30.s4()), 0x4eb1bbdd);
        check!(bic_v(V29.b8(), V30.b8(), V31.b8()), 0x0e7f1fdd);
        check!(bic_v(V29.b16(), V30.b16(), V31.b16()), 0x4e7f1fdd);
        check!(clz_v(V29.b8(), V30.b8()), 0x2e204bdd);
        check!(clz_v(V29.b16(), V30.b16()), 0x6e204bdd);
        check!(clz_v(V29.h4(), V30.h4()), 0x2e604bdd);
        check!(clz_v(V29.h8(), V30.h8()), 0x6e604bdd);
        check!(clz_v(V29.s2(), V30.s2()), 0x2ea04bdd);
        check!(clz_v(V29.s4(), V30.s4()), 0x6ea04bdd);
        check!(cmeq_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f8fdd);
        check!(cmeq_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f8fdd);
        check!(cmeq_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f8fdd);
        check!(cmeq_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f8fdd);
        check!(cmeq_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf8fdd);
        check!(cmeq_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf8fdd);
        check!(cmeq_v(V29.d2(), V30.d2(), V31.d2()), 0x6eff8fdd);
        check!(cmge_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f3fdd);
        check!(cmge_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f3fdd);
        check!(cmge_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f3fdd);
        check!(cmge_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f3fdd);
        check!(cmge_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf3fdd);
        check!(cmge_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf3fdd);
        check!(cmge_v(V29.d2(), V30.d2(), V31.d2()), 0x4eff3fdd);
        check!(cmgt_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f37dd);
        check!(cmgt_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f37dd);
        check!(cmgt_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f37dd);
        check!(cmgt_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f37dd);
        check!(cmgt_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf37dd);
        check!(cmgt_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf37dd);
        check!(cmgt_v(V29.d2(), V30.d2(), V31.d2()), 0x4eff37dd);
        check!(cnt_v(V29.b16(), V30.b16()), 0x4e205bdd);
        check!(mul_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f9fdd);
        check!(mul_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f9fdd);
        check!(mul_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f9fdd);
        check!(mul_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f9fdd);
        check!(mul_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf9fdd);
        check!(mul_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf9fdd);
        check!(not_v(V29.b16(), V30.b16()), 0x6e205bdd);
        check!(orr_v(V29.b8(), V30.b8(), V31.b8()), 0x0ebf1fdd);
        check!(orr_v(V29.b16(), V30.b16(), V31.b16()), 0x4ebf1fdd);
        check!(pmul_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f9fdd);
        check!(pmul_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f9fdd);
        check!(pmull_v(V29.h8(), V30.b8(), V31.b8()), 0x0e3fe3dd);
        check!(pmull_64(Q29, V30.d1(), V31.d1()), 0x0effe3dd);
        check!(rbit_v(V29.b16(), V30.b16()), 0x6e605bdd);
        check!(rev16_v(V29.b16(), V30.b16()), 0x4e201bdd);
        check!(rev32_v(V29.b8(), V30.b8()), 0x2e200bdd);
        check!(rev32_v(V29.b16(), V30.b16()), 0x6e200bdd);
        check!(rev32_v(V29.h4(), V30.h4()), 0x2e600bdd);
        check!(rev32_v(V29.h8(), V30.h8()), 0x6e600bdd);
        check!(rev64_v(V29.b8(), V30.b8()), 0x0e200bdd);
        check!(rev64_v(V29.b16(), V30.b16()), 0x4e200bdd);
        check!(rev64_v(V29.h4(), V30.h4()), 0x0e600bdd);
        check!(rev64_v(V29.h8(), V30.h8()), 0x4e600bdd);
        check!(rev64_v(V29.s2(), V30.s2()), 0x0ea00bdd);
        check!(rev64_v(V29.s4(), V30.s4()), 0x4ea00bdd);
        check!(sabd_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f77dd);
        check!(sabd_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f77dd);
        check!(sabd_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f77dd);
        check!(sabd_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f77dd);
        check!(sabd_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf77dd);
        check!(sabd_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf77dd);
        check!(saddlp_v(V29.h8(), V30.b16()), 0x4e202bdd);
        check!(saddlp_v(V29.s4(), V30.h8()), 0x4e602bdd);
        check!(saddlp_v(V29.d2(), V30.s4()), 0x4ea02bdd);
        check!(smax_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f67dd);
        check!(smax_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f67dd);
        check!(smax_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f67dd);
        check!(smax_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f67dd);
        check!(smax_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf67dd);
        check!(smax_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf67dd);
        check!(smaxp_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3fa7dd);
        check!(smaxp_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3fa7dd);
        check!(smaxp_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7fa7dd);
        check!(smaxp_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7fa7dd);
        check!(smaxp_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebfa7dd);
        check!(smaxp_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebfa7dd);
        check!(smin_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f6fdd);
        check!(smin_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f6fdd);
        check!(smin_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f6fdd);
        check!(smin_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f6fdd);
        check!(smin_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf6fdd);
        check!(smin_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf6fdd);
        check!(sminp_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3fafdd);
        check!(sminp_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3fafdd);
        check!(sminp_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7fafdd);
        check!(sminp_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7fafdd);
        check!(sminp_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebfafdd);
        check!(sminp_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebfafdd);
        check!(smull_v(V29.h8(), V30.b8(), V31.b8()), 0x0e3fc3dd);
        check!(smull_v(V29.s4(), V30.h4(), V31.h4()), 0x0e7fc3dd);
        check!(smull_v(V29.d2(), V30.s2(), V31.s2()), 0x0ebfc3dd);
        check!(sqabs_v(V29.b8(), V30.b8()), 0x0e207bdd);
        check!(sqabs_v(V29.b16(), V30.b16()), 0x4e207bdd);
        check!(sqabs_v(V29.h4(), V30.h4()), 0x0e607bdd);
        check!(sqabs_v(V29.h8(), V30.h8()), 0x4e607bdd);
        check!(sqabs_v(V29.s2(), V30.s2()), 0x0ea07bdd);
        check!(sqabs_v(V29.s4(), V30.s4()), 0x4ea07bdd);
        check!(sqabs_v(V29.d2(), V30.d2()), 0x4ee07bdd);
        check!(sqdmulh_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7fb7dd);
        check!(sqdmulh_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7fb7dd);
        check!(sqdmulh_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebfb7dd);
        check!(sqdmulh_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebfb7dd);
        check!(sqdmull_v(V29.s4(), V30.h4(), V31.h4()), 0x0e7fd3dd);
        check!(sqdmull_v(V29.d2(), V30.s2(), V31.s2()), 0x0ebfd3dd);
        check!(sqneg_v(V29.b8(), V30.b8()), 0x2e207bdd);
        check!(sqneg_v(V29.b16(), V30.b16()), 0x6e207bdd);
        check!(sqneg_v(V29.h4(), V30.h4()), 0x2e607bdd);
        check!(sqneg_v(V29.h8(), V30.h8()), 0x6e607bdd);
        check!(sqneg_v(V29.s2(), V30.s2()), 0x2ea07bdd);
        check!(sqneg_v(V29.s4(), V30.s4()), 0x6ea07bdd);
        check!(sqneg_v(V29.d2(), V30.d2()), 0x6ee07bdd);
        check!(sqrdmulh_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7fb7dd);
        check!(sqrdmulh_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7fb7dd);
        check!(sqrdmulh_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebfb7dd);
        check!(sqrdmulh_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebfb7dd);
        check!(sqshl_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f4fdd);
        check!(sqshl_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f4fdd);
        check!(sqshl_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f4fdd);
        check!(sqshl_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f4fdd);
        check!(sqshl_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf4fdd);
        check!(sqshl_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf4fdd);
        check!(sqshl_v(V29.d2(), V30.d2(), V31.d2()), 0x4eff4fdd);
        check!(sqxtn_v(V29.b8(), V30.h8()), 0x0e214bdd);
        check!(sqxtn_v(V29.h4(), V30.s4()), 0x0e614bdd);
        check!(sqxtn_v(V29.s2(), V30.d2()), 0x0ea14bdd);
        check!(sqxtun_v(V29.b8(), V30.h8()), 0x2e212bdd);
        check!(sqxtun_v(V29.h4(), V30.s4()), 0x2e612bdd);
        check!(sqxtun_v(V29.s2(), V30.d2()), 0x2ea12bdd);
        check!(srhadd_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f17dd);
        check!(srhadd_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f17dd);
        check!(srhadd_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f17dd);
        check!(srhadd_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f17dd);
        check!(srhadd_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf17dd);
        check!(srhadd_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf17dd);
        check!(srshl_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f57dd);
        check!(srshl_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f57dd);
        check!(srshl_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f57dd);
        check!(srshl_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f57dd);
        check!(srshl_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf57dd);
        check!(srshl_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf57dd);
        check!(srshl_v(V29.d2(), V30.d2(), V31.d2()), 0x4eff57dd);
        check!(sshl_v(V29.b8(), V30.b8(), V31.b8()), 0x0e3f47dd);
        check!(sshl_v(V29.b16(), V30.b16(), V31.b16()), 0x4e3f47dd);
        check!(sshl_v(V29.h4(), V30.h4(), V31.h4()), 0x0e7f47dd);
        check!(sshl_v(V29.h8(), V30.h8(), V31.h8()), 0x4e7f47dd);
        check!(sshl_v(V29.s2(), V30.s2(), V31.s2()), 0x0ebf47dd);
        check!(sshl_v(V29.s4(), V30.s4(), V31.s4()), 0x4ebf47dd);
        check!(sshl_v(V29.d2(), V30.d2(), V31.d2()), 0x4eff47dd);
        check!(suqadd_v(V29.b8(), V30.b8()), 0x0e203bdd);
        check!(suqadd_v(V29.b16(), V30.b16()), 0x4e203bdd);
        check!(suqadd_v(V29.h4(), V30.h4()), 0x0e603bdd);
        check!(suqadd_v(V29.h8(), V30.h8()), 0x4e603bdd);
        check!(suqadd_v(V29.s2(), V30.s2()), 0x0ea03bdd);
        check!(suqadd_v(V29.s4(), V30.s4()), 0x4ea03bdd);
        check!(suqadd_v(V29.d2(), V30.d2()), 0x4ee03bdd);
        check!(trn1_v(V29.b8(), V30.b8(), V31.b8()), 0x0e1f2bdd);
        check!(trn1_v(V29.b16(), V30.b16(), V31.b16()), 0x4e1f2bdd);
        check!(trn1_v(V29.h4(), V30.h4(), V31.h4()), 0x0e5f2bdd);
        check!(trn1_v(V29.h8(), V30.h8(), V31.h8()), 0x4e5f2bdd);
        check!(trn1_v(V29.s2(), V30.s2(), V31.s2()), 0x0e9f2bdd);
        check!(trn1_v(V29.s4(), V30.s4(), V31.s4()), 0x4e9f2bdd);
        check!(trn1_v(V29.d2(), V30.d2(), V31.d2()), 0x4edf2bdd);
        check!(trn2_v(V29.b8(), V30.b8(), V31.b8()), 0x0e1f6bdd);
        check!(trn2_v(V29.b16(), V30.b16(), V31.b16()), 0x4e1f6bdd);
        check!(trn2_v(V29.h4(), V30.h4(), V31.h4()), 0x0e5f6bdd);
        check!(trn2_v(V29.h8(), V30.h8(), V31.h8()), 0x4e5f6bdd);
        check!(trn2_v(V29.s2(), V30.s2(), V31.s2()), 0x0e9f6bdd);
        check!(trn2_v(V29.s4(), V30.s4(), V31.s4()), 0x4e9f6bdd);
        check!(trn2_v(V29.d2(), V30.d2(), V31.d2()), 0x4edf6bdd);
        check!(uaddlp_v(V29.h8(), V30.b16()), 0x6e202bdd);
        check!(uaddlp_v(V29.s4(), V30.h8()), 0x6e602bdd);
        check!(uaddlp_v(V29.d2(), V30.s4()), 0x6ea02bdd);
        check!(umax_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f67dd);
        check!(umax_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f67dd);
        check!(umax_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f67dd);
        check!(umax_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f67dd);
        check!(umax_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf67dd);
        check!(umax_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf67dd);
        check!(umaxp_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3fa7dd);
        check!(umaxp_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3fa7dd);
        check!(umaxp_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7fa7dd);
        check!(umaxp_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7fa7dd);
        check!(umaxp_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebfa7dd);
        check!(umaxp_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebfa7dd);
        check!(umin_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f6fdd);
        check!(umin_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f6fdd);
        check!(umin_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f6fdd);
        check!(umin_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f6fdd);
        check!(umin_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf6fdd);
        check!(umin_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf6fdd);
        check!(uminp_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3fafdd);
        check!(uminp_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3fafdd);
        check!(uminp_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7fafdd);
        check!(uminp_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7fafdd);
        check!(uminp_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebfafdd);
        check!(uminp_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebfafdd);
        check!(umull_v(V29.h8(), V30.b8(), V31.b8()), 0x2e3fc3dd);
        check!(umull_v(V29.s4(), V30.h4(), V31.h4()), 0x2e7fc3dd);
        check!(umull_v(V29.d2(), V30.s2(), V31.s2()), 0x2ebfc3dd);
        check!(uqshl_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f4fdd);
        check!(uqshl_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f4fdd);
        check!(uqshl_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f4fdd);
        check!(uqshl_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f4fdd);
        check!(uqshl_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf4fdd);
        check!(uqshl_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf4fdd);
        check!(uqshl_v(V29.d2(), V30.d2(), V31.d2()), 0x6eff4fdd);
        check!(uqxtn_v(V29.b8(), V30.h8()), 0x2e214bdd);
        check!(uqxtn_v(V29.h4(), V30.s4()), 0x2e614bdd);
        check!(uqxtn_v(V29.s2(), V30.d2()), 0x2ea14bdd);
        check!(urecpe_v(V29.s4(), V30.s4()), 0x4ea1cbdd);
        check!(urhadd_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f17dd);
        check!(urhadd_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f17dd);
        check!(urhadd_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f17dd);
        check!(urhadd_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f17dd);
        check!(urhadd_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf17dd);
        check!(urhadd_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf17dd);
        check!(urshl_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f57dd);
        check!(urshl_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f57dd);
        check!(urshl_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f57dd);
        check!(urshl_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f57dd);
        check!(urshl_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf57dd);
        check!(urshl_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf57dd);
        check!(urshl_v(V29.d2(), V30.d2(), V31.d2()), 0x6eff57dd);
        check!(ursqrte_v(V29.s4(), V30.s4()), 0x6ea1cbdd);
        check!(ushl_v(V29.b8(), V30.b8(), V31.b8()), 0x2e3f47dd);
        check!(ushl_v(V29.b16(), V30.b16(), V31.b16()), 0x6e3f47dd);
        check!(ushl_v(V29.h4(), V30.h4(), V31.h4()), 0x2e7f47dd);
        check!(ushl_v(V29.h8(), V30.h8(), V31.h8()), 0x6e7f47dd);
        check!(ushl_v(V29.s2(), V30.s2(), V31.s2()), 0x2ebf47dd);
        check!(ushl_v(V29.s4(), V30.s4(), V31.s4()), 0x6ebf47dd);
        check!(ushl_v(V29.d2(), V30.d2(), V31.d2()), 0x6eff47dd);
        check!(usqadd_v(V29.b8(), V30.b8()), 0x2e203bdd);
        check!(usqadd_v(V29.b16(), V30.b16()), 0x6e203bdd);
        check!(usqadd_v(V29.h4(), V30.h4()), 0x2e603bdd);
        check!(usqadd_v(V29.h8(), V30.h8()), 0x6e603bdd);
        check!(usqadd_v(V29.s2(), V30.s2()), 0x2ea03bdd);
        check!(usqadd_v(V29.s4(), V30.s4()), 0x6ea03bdd);
        check!(usqadd_v(V29.d2(), V30.d2()), 0x6ee03bdd);
        check!(uzp1_v(V29.b8(), V30.b8(), V31.b8()), 0x0e1f1bdd);
        check!(uzp1_v(V29.b16(), V30.b16(), V31.b16()), 0x4e1f1bdd);
        check!(uzp1_v(V29.h4(), V30.h4(), V31.h4()), 0x0e5f1bdd);
        check!(uzp1_v(V29.h8(), V30.h8(), V31.h8()), 0x4e5f1bdd);
        check!(uzp1_v(V29.s2(), V30.s2(), V31.s2()), 0x0e9f1bdd);
        check!(uzp1_v(V29.s4(), V30.s4(), V31.s4()), 0x4e9f1bdd);
        check!(uzp1_v(V29.d2(), V30.d2(), V31.d2()), 0x4edf1bdd);
        check!(uzp2_v(V29.b8(), V30.b8(), V31.b8()), 0x0e1f5bdd);
        check!(uzp2_v(V29.b16(), V30.b16(), V31.b16()), 0x4e1f5bdd);
        check!(uzp2_v(V29.h4(), V30.h4(), V31.h4()), 0x0e5f5bdd);
        check!(uzp2_v(V29.h8(), V30.h8(), V31.h8()), 0x4e5f5bdd);
        check!(uzp2_v(V29.s2(), V30.s2(), V31.s2()), 0x0e9f5bdd);
        check!(uzp2_v(V29.s4(), V30.s4(), V31.s4()), 0x4e9f5bdd);
        check!(uzp2_v(V29.d2(), V30.d2(), V31.d2()), 0x4edf5bdd);
        check!(zip2_v(V29.b8(), V30.b8(), V31.b8()), 0x0e1f7bdd);
        check!(zip2_v(V29.b16(), V30.b16(), V31.b16()), 0x4e1f7bdd);
        check!(zip2_v(V29.h4(), V30.h4(), V31.h4()), 0x0e5f7bdd);
        check!(zip2_v(V29.h8(), V30.h8(), V31.h8()), 0x4e5f7bdd);
        check!(zip2_v(V29.s2(), V30.s2(), V31.s2()), 0x0e9f7bdd);
        check!(zip2_v(V29.s4(), V30.s4(), V31.s4()), 0x4e9f7bdd);
        check!(zip2_v(V29.d2(), V30.d2(), V31.d2()), 0x4edf7bdd);
        check!(umov(W29, V30.b16(), 15), 0x0e1f3fdd);
        check!(umov(X29, V30.d2(), 1), 0x4e183fdd);
        check!(mov_to_element(V29.b16(), 15, W30), 0x4e1f1fdd);
        check!(mov_to_element(V29.d2(), 1, X30), 0x4e181fdd);
        check!(mov_d1_from_d0(V29.d2(), V30.d2()), 0x6e1807dd);
        check!(dup_from_gp(V29.b16(), W30), 0x4e010fdd);
        check!(dup_from_gp(V29.d2(), X30), 0x4e080fdd);
        check!(dup_element(V29.b8(), V30, 15), 0x0e1f07dd);
        check!(dup_element(V29.d2(), V30, 1), 0x4e1807dd);
        check!(tbl_v(V29.b8(), V30.b16(), V31.b8(), 4), 0x0e1f63dd);
        check!(tbx_v(V29.b8(), V30.b16(), V31.b8(), 4), 0x0e1f73dd);
        check!(tbl_v(V29.b16(), V30.b16(), V31.b16(), 4), 0x4e1f63dd);
        check!(tbx_v(V29.b16(), V30.b16(), V31.b16(), 4), 0x4e1f73dd);
        check!(shl_v(V29.d2(), V30.d2(), 63), 0x4f7f57dd);
        check!(sqshlu_v(V29.d2(), V30.d2(), 63), 0x6f7f67dd);
    }

    #[test]
    fn movi_rep_matches_oaknut_literal_words() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        // Independent words from Oaknut's DReg/RepImm encoding, not the encoder under test.
        for (rd, mask, expected) in [
            (D0, 0x00, 0x2f00_e400),
            (D2, 0x0f, 0x2f00_e5e2),
            (D2, 0xf0, 0x2f07_e602),
            (D31, 0xff, 0x2f07_e7ff),
            (D0, 0x81, 0x2f04_e420),
        ] {
            let offset = block.code_size();
            CodeGenerator::new(&mut block).movi_rep(rd, mask).unwrap();
            assert_eq!(block.code_size(), offset + 4);
            let actual =
                unsafe { (block.code_base_ptr().add(offset) as *const u32).read_unaligned() };
            assert_eq!(actual, expected);
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn movi_rep_native_expands_every_byte_mask() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut offsets = Vec::new();
        for mask in 0..=u8::MAX {
            offsets.push(block.code_size());
            let mut code = CodeGenerator::new(&mut block);
            code.movi_rep(D0, mask).unwrap();
            code.fmov_to_gp(X0, D0).unwrap();
            code.ret().unwrap();
        }
        block.seal();
        for (mask, offset) in offsets.into_iter().enumerate() {
            let run: unsafe extern "C" fn() -> u64 =
                unsafe { std::mem::transmute(block.code_base_ptr().add(offset)) };
            let expected = (0..8).fold(0u64, |value, byte| {
                value
                    | if mask & (1 << byte) != 0 {
                        0xff << (byte * 8)
                    } else {
                        0
                    }
            });
            assert_eq!(unsafe { run() }, expected, "mask {mask:#04x}");
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn packed_rep_masks_native_results_match_eden() {
        // Replay the upstream packed scratch sequences; emitter tests check these mask words.
        for add_is_hi in [false, true] {
            let mut block = BlockOfCode::with_size(4096).unwrap();
            {
                let mut code = CodeGenerator::new(&mut block);
                code.fmov_from_gp(D4, X0).unwrap();
                code.fmov_from_gp(D5, X1).unwrap();
                code.uxtl(V0.s4(), V4.h4()).unwrap();
                code.uxtl(V1.s4(), V5.h4()).unwrap();
                code.ext(V1.b8(), V1.b8(), V1.b8(), 4).unwrap();
                code.movi_rep(D2, if add_is_hi { 0xf0 } else { 0x0f })
                    .unwrap();
                code.eor_v(V1.b8(), V1.b8(), V2.b8()).unwrap();
                code.sub_v(V1.s2(), V1.s2(), V2.s2()).unwrap();
                code.sub_v(V3.s2(), V0.s2(), V1.s2()).unwrap();
                code.xtn(V3.h4(), V3.s4()).unwrap();
                code.fmov_to_gp(X0, D3).unwrap();
                code.ret().unwrap();
            }
            block.seal();
            let run: unsafe extern "C" fn(u64, u64) -> u64 =
                unsafe { std::mem::transmute(block.code_base_ptr()) };
            for (a, b) in [
                (0u32, 1u32),
                (0x1020_3040, 0x0102_0304),
                (u32::MAX, u32::MAX),
            ] {
                let (alo, ahi) = (a as u16, (a >> 16) as u16);
                let (blo, bhi) = (b as u16, (b >> 16) as u16);
                let (lo, hi) = if add_is_hi {
                    (alo.wrapping_sub(bhi), ahi.wrapping_add(blo))
                } else {
                    (alo.wrapping_add(bhi), ahi.wrapping_sub(blo))
                };
                let expected = u32::from(lo) | (u32::from(hi) << 16);
                assert_eq!(unsafe { run(a.into(), b.into()) } as u32, expected);
            }
        }

        let mut block = BlockOfCode::with_size(4096).unwrap();
        {
            let mut code = CodeGenerator::new(&mut block);
            code.fmov_from_gp(D4, X0).unwrap();
            code.fmov_from_gp(D5, X1).unwrap();
            code.movi_rep(D2, 0x0f).unwrap();
            code.uabd(V3.b8(), V4.b8(), V5.b8()).unwrap();
            code.and_v(V3.b8(), V3.b8(), V2.b8()).unwrap();
            code.uaddlv(V3.h(), V3.b8()).unwrap();
            code.fmov_to_gp(X0, D3).unwrap();
            code.ret().unwrap();
        }
        block.seal();
        let run: unsafe extern "C" fn(u64, u64) -> u64 =
            unsafe { std::mem::transmute(block.code_base_ptr()) };
        for (a, b, expected) in [
            (0x20, 0, 32),
            (0xffff_ffff, 0, 1020),
            (0xffff_ffff_0000_0020, 0, 32),
            (0, 0xffff_ffff_0000_0020, 32),
        ] {
            assert_eq!(unsafe { run(a, b) }, expected);
        }
    }

    #[test]
    fn invalid_lane_widths_and_indices_do_not_emit() {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        macro_rules! rejects {
            ($method:ident($($arg:expr),*)) => {
                assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    CodeGenerator::new(&mut block).$method($($arg),*).unwrap();
                })).is_err());
                assert_eq!(block.code_size(), 0);
            };
        }
        rejects!(umov(W0, V1.d2(), 0));
        rejects!(umov(X0, V1.b16(), 0));
        rejects!(umov(W0, V1.b16(), 16));
        rejects!(mov_to_element(V0.d2(), 0, W1));
        rejects!(mov_to_element(V0.d2(), 2, X1));
        rejects!(dup_from_gp(V0.d1(), X1));
        rejects!(dup_element(V0.b8(), V1, 16));
        rejects!(addv_scalar(H0, V1.b16()));
        rejects!(saddlp_v(V0.s4(), V1.b16()));
        rejects!(sqdmull_v(V0.h8(), V1.b8(), V2.b8()));
        rejects!(mul_v(V0.d2(), V1.d2(), V2.d2()));
        rejects!(shl_v(V0.s4(), V1.s4(), 32));
        rejects!(tbl_v(V0.b16(), V1.b16(), V2.b16(), 0));
        rejects!(tbx_v(V0.b16(), V1.b16(), V2.b16(), 5));
    }
}

use super::CodeGenerator;
use crate::{
    inst, DReg, FpReg, GpReg, NarrowingSource, QReg, VReg, VReg16B, VReg1D, VReg2D, VReg4S, VReg8B,
    VReg8H, VRegArranged, VRegBytes, WideningSource,
};

impl CodeGenerator<'_> {
    /// Oaknut `MOVI(DReg, RepImm)`: each mask bit expands to an FF/00 byte.
    pub fn movi_rep(&mut self, rd: DReg, encoded: u8) -> Result<(), String> {
        self.emit(inst::movi_d_rep_imm(rd.index(), encoded))
    }

    /// Oaknut `TRN2` with matching vector arrangements.
    pub fn trn2_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid TRN2 arrangement"
        );
        self.emit(inst::trn2_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `TRN1` with matching vector arrangements.
    pub fn trn1_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid TRN1 arrangement"
        );
        self.emit(inst::trn1_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `CMGE` with matching vector arrangements.
    pub fn cmge_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid CMGE arrangement"
        );
        self.emit(inst::cmge_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `ABS` with matching vector arrangements.
    pub fn abs_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid ABS arrangement"
        );
        self.emit(inst::abs_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `CLZ` with matching vector arrangements.
    pub fn clz_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid CLZ arrangement"
        );
        self.emit(inst::clz_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `REV32` with matching vector arrangements.
    pub fn rev32_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true)
            ),
            "invalid REV32 arrangement"
        );
        self.emit(inst::rev32_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `REV64` with matching vector arrangements.
    pub fn rev64_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid REV64 arrangement"
        );
        self.emit(inst::rev64_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `MUL` with matching vector arrangements.
    pub fn mul_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (16, false) | (16, true) | (32, false) | (32, true) | (8, false) | (8, true)
            ),
            "invalid MUL arrangement"
        );
        self.emit(inst::mul_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `CMEQ` with matching vector arrangements.
    pub fn cmeq_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid CMEQ arrangement"
        );
        self.emit(inst::cmeq_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `CMGT` with matching vector arrangements.
    pub fn cmgt_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid CMGT arrangement"
        );
        self.emit(inst::cmgt_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SMAX` with matching vector arrangements.
    pub fn smax_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SMAX arrangement"
        );
        self.emit(inst::smax_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UMAX` with matching vector arrangements.
    pub fn umax_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid UMAX arrangement"
        );
        self.emit(inst::umax_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SMIN` with matching vector arrangements.
    pub fn smin_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SMIN arrangement"
        );
        self.emit(inst::smin_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UMIN` with matching vector arrangements.
    pub fn umin_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid UMIN arrangement"
        );
        self.emit(inst::umin_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `ADDP` with matching vector arrangements.
    pub fn addp_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid ADDP arrangement"
        );
        self.emit(inst::addp_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SMAXP` with matching vector arrangements.
    pub fn smaxp_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SMAXP arrangement"
        );
        self.emit(inst::smaxp_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UMAXP` with matching vector arrangements.
    pub fn umaxp_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid UMAXP arrangement"
        );
        self.emit(inst::umaxp_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SMINP` with matching vector arrangements.
    pub fn sminp_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SMINP arrangement"
        );
        self.emit(inst::sminp_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UMINP` with matching vector arrangements.
    pub fn uminp_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid UMINP arrangement"
        );
        self.emit(inst::uminp_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `PMUL` with matching vector arrangements.
    pub fn pmul_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!((V::SIZE, V::Q), (8, false) | (8, true)),
            "invalid PMUL arrangement"
        );
        self.emit(inst::pmul_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SSHL` with matching vector arrangements.
    pub fn sshl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SSHL arrangement"
        );
        self.emit(inst::sshl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `USHL` with matching vector arrangements.
    pub fn ushl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid USHL arrangement"
        );
        self.emit(inst::ushl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SRSHL` with matching vector arrangements.
    pub fn srshl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SRSHL arrangement"
        );
        self.emit(inst::srshl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `URSHL` with matching vector arrangements.
    pub fn urshl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid URSHL arrangement"
        );
        self.emit(inst::urshl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SABD` with matching vector arrangements.
    pub fn sabd_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SABD arrangement"
        );
        self.emit(inst::sabd_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SRHADD` with matching vector arrangements.
    pub fn srhadd_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SRHADD arrangement"
        );
        self.emit(inst::srhadd_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `URHADD` with matching vector arrangements.
    pub fn urhadd_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false) | (8, true) | (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid URHADD arrangement"
        );
        self.emit(inst::urhadd_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SQABS` with matching vector arrangements.
    pub fn sqabs_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SQABS arrangement"
        );
        self.emit(inst::sqabs_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `SUQADD` with matching vector arrangements.
    pub fn suqadd_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SUQADD arrangement"
        );
        self.emit(inst::suqadd_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `SQDMULH` with matching vector arrangements.
    pub fn sqdmulh_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SQDMULH arrangement"
        );
        self.emit(inst::sqdmulh_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SQRDMULH` with matching vector arrangements.
    pub fn sqrdmulh_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (16, false) | (16, true) | (32, false) | (32, true)
            ),
            "invalid SQRDMULH arrangement"
        );
        self.emit(inst::sqrdmulh_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SQNEG` with matching vector arrangements.
    pub fn sqneg_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SQNEG arrangement"
        );
        self.emit(inst::sqneg_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `SQSHL` with matching vector arrangements.
    pub fn sqshl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SQSHL arrangement"
        );
        self.emit(inst::sqshl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SQSHLU` with matching vector arrangements.
    pub fn sqshlu_v<V: VRegArranged>(&mut self, rd: V, rn: V, shift: u8) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SQSHLU arrangement"
        );
        self.emit(inst::sqshlu_v(rd.index(), rn.index(), V::SIZE, shift, V::Q))
    }

    /// Oaknut `USQADD` with matching vector arrangements.
    pub fn usqadd_v<V: VRegArranged>(&mut self, rd: V, rn: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid USQADD arrangement"
        );
        self.emit(inst::usqadd_v(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// Oaknut `UQSHL` with matching vector arrangements.
    pub fn uqshl_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid UQSHL arrangement"
        );
        self.emit(inst::uqshl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `ZIP2` with matching vector arrangements.
    pub fn zip2_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid ZIP2 arrangement"
        );
        self.emit(inst::zip2_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UZP1` with matching vector arrangements.
    pub fn uzp1_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid UZP1 arrangement"
        );
        self.emit(inst::uzp1_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `UZP2` with matching vector arrangements.
    pub fn uzp2_v<V: VRegArranged>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid UZP2 arrangement"
        );
        self.emit(inst::uzp2_v(
            rd.index(),
            rn.index(),
            rm.index(),
            V::SIZE,
            V::Q,
        ))
    }

    /// Oaknut `SHL` with matching vector arrangements.
    pub fn shl_v<V: VRegArranged>(&mut self, rd: V, rn: V, shift: u8) -> Result<(), String> {
        assert!(
            matches!(
                (V::SIZE, V::Q),
                (8, false)
                    | (8, true)
                    | (16, false)
                    | (16, true)
                    | (32, false)
                    | (32, true)
                    | (64, true)
            ),
            "invalid SHL arrangement"
        );
        self.emit(inst::shl_v(rd.index(), rn.index(), V::SIZE, shift, V::Q))
    }

    pub fn not_v(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::not_v16b(rd.index(), rn.index()))
    }

    pub fn cnt_v(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::cnt_v16b(rd.index(), rn.index()))
    }

    pub fn rbit_v(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::rbit_v16b(rd.index(), rn.index()))
    }

    pub fn rev16_v(&mut self, rd: VReg16B, rn: VReg16B) -> Result<(), String> {
        self.emit(inst::rev16_v16b(rd.index(), rn.index()))
    }

    pub fn bic_v<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        self.emit(if V::Q {
            inst::bic_v16b(rd.index(), rn.index(), rm.index())
        } else {
            inst::bic_v8b(rd.index(), rn.index(), rm.index())
        })
    }

    pub fn orr_v<V: VRegBytes>(&mut self, rd: V, rn: V, rm: V) -> Result<(), String> {
        self.emit(if V::Q {
            inst::orr_v16b(rd.index(), rn.index(), rm.index())
        } else {
            inst::orr_v8b(rd.index(), rn.index(), rm.index())
        })
    }

    pub fn urecpe_v(&mut self, rd: VReg4S, rn: VReg4S) -> Result<(), String> {
        self.emit(inst::urecpe_v4s(rd.index(), rn.index()))
    }

    pub fn ursqrte_v(&mut self, rd: VReg4S, rn: VReg4S) -> Result<(), String> {
        self.emit(inst::ursqrte_v4s(rd.index(), rn.index()))
    }

    pub fn smull_v<N: WideningSource>(&mut self, rd: N::Wide, rn: N, rm: N) -> Result<(), String> {
        self.emit(inst::smull_v(rd.index(), rn.index(), rm.index(), N::SIZE))
    }

    pub fn umull_v<N: WideningSource>(&mut self, rd: N::Wide, rn: N, rm: N) -> Result<(), String> {
        self.emit(inst::umull_v(rd.index(), rn.index(), rm.index(), N::SIZE))
    }

    pub fn sqdmull_v<N: WideningSource>(
        &mut self,
        rd: N::Wide,
        rn: N,
        rm: N,
    ) -> Result<(), String> {
        assert!(matches!(N::SIZE, 16 | 32), "SQDMULL requires H or S lanes");
        self.emit(inst::sqdmull_v(rd.index(), rn.index(), rm.index(), N::SIZE))
    }

    pub fn sqxtn_v<N: NarrowingSource>(&mut self, rd: N::Narrow, rn: N) -> Result<(), String> {
        self.emit(inst::sqxtn_v(rd.index(), rn.index(), N::SIZE))
    }

    pub fn sqxtun_v<N: NarrowingSource>(&mut self, rd: N::Narrow, rn: N) -> Result<(), String> {
        self.emit(inst::sqxtun_v(rd.index(), rn.index(), N::SIZE))
    }

    pub fn uqxtn_v<N: NarrowingSource>(&mut self, rd: N::Narrow, rn: N) -> Result<(), String> {
        self.emit(inst::uqxtn_v(rd.index(), rn.index(), N::SIZE))
    }

    pub fn saddlp_v<D: VRegArranged, N: VRegArranged>(
        &mut self,
        rd: D,
        rn: N,
    ) -> Result<(), String> {
        assert!(
            D::Q && N::Q && matches!(N::SIZE, 8 | 16 | 32) && D::SIZE == 2 * N::SIZE,
            "pairwise widening requires matching full-width arrangements"
        );
        self.emit(inst::saddlp_v(rd.index(), rn.index(), N::SIZE))
    }

    pub fn uaddlp_v<D: VRegArranged, N: VRegArranged>(
        &mut self,
        rd: D,
        rn: N,
    ) -> Result<(), String> {
        assert!(
            D::Q && N::Q && matches!(N::SIZE, 8 | 16 | 32) && D::SIZE == 2 * N::SIZE,
            "pairwise widening requires matching full-width arrangements"
        );
        self.emit(inst::uaddlp_v(rd.index(), rn.index(), N::SIZE))
    }

    pub fn pmull_v(&mut self, rd: VReg8H, rn: VReg8B, rm: VReg8B) -> Result<(), String> {
        self.emit(inst::pmull_v(rd.index(), rn.index(), rm.index(), 8))
    }

    /// Oaknut VReg_1Q is represented by QReg; no shared register type is added.
    pub fn pmull_64(&mut self, rd: QReg, rn: VReg1D, rm: VReg1D) -> Result<(), String> {
        self.emit(inst::pmull_v(rd.index(), rn.index(), rm.index(), 64))
    }

    pub fn addv_scalar<F: FpReg, V: VRegArranged>(&mut self, rd: F, rn: V) -> Result<(), String> {
        assert!(
            V::Q && matches!(V::SIZE, 8 | 16 | 32) && (8u16 << F::SIZE) == V::SIZE as u16,
            "ADDV scalar width must match source lanes"
        );
        self.emit(inst::addv_from_v(rd.index(), rn.index(), V::SIZE))
    }

    pub fn addp_scalar(&mut self, rd: DReg, rn: VReg2D) -> Result<(), String> {
        self.emit(inst::addp_d_from_v2d(rd.index(), rn.index()))
    }

    pub fn umov<R: GpReg, V: VRegArranged>(
        &mut self,
        rd: R,
        rn: V,
        index: u8,
    ) -> Result<(), String> {
        assert!(
            R::SF == (V::SIZE == 64),
            "UMOV requires X for D lanes, W otherwise"
        );
        self.emit(inst::umov_from_v(rd.index(), rn.index(), V::SIZE, index))
    }

    pub fn mov_to_element<V: VRegArranged, R: GpReg>(
        &mut self,
        rd: V,
        index: u8,
        rn: R,
    ) -> Result<(), String> {
        assert!(
            R::SF == (V::SIZE == 64),
            "MOV element requires X for D lanes, W otherwise"
        );
        self.emit(inst::mov_to_v_element(
            rd.index(),
            rn.index(),
            V::SIZE,
            index,
        ))
    }

    /// `MOV(Vd.D[1], Vn.D[0])`, the packing form used by Dynarmic.
    pub fn mov_d1_from_d0(&mut self, rd: VReg2D, rn: VReg2D) -> Result<(), String> {
        self.emit(inst::mov_v_d1_from_v_d0(rd.index(), rn.index()))
    }

    pub fn dup_from_gp<V: VRegArranged, R: GpReg>(&mut self, rd: V, rn: R) -> Result<(), String> {
        assert!(
            R::SF == (V::SIZE == 64) && (V::Q || V::SIZE != 64),
            "DUP requires X for 2D, W for B/H/S"
        );
        self.emit(inst::dup_v_from_reg(rd.index(), rn.index(), V::SIZE, V::Q))
    }

    /// The source element can be anywhere in the full 128-bit register even
    /// when the destination arrangement is only 64 bits.
    pub fn dup_element<V: VRegArranged>(
        &mut self,
        rd: V,
        rn: VReg,
        index: u8,
    ) -> Result<(), String> {
        assert!(
            V::Q || V::SIZE != 64,
            "DUP D lanes require a 2D destination"
        );
        self.emit(inst::dup_v_from_element(
            rd.index(),
            rn.index(),
            V::SIZE,
            index,
            V::Q,
        ))
    }

    /// The table starts at `rn` and occupies `list_len` consecutive registers.
    pub fn tbl_v<V: VRegBytes>(
        &mut self,
        rd: V,
        rn: VReg16B,
        rm: V,
        list_len: u8,
    ) -> Result<(), String> {
        self.emit(inst::tbl_v(
            rd.index(),
            rn.index(),
            rm.index(),
            list_len,
            V::Q,
        ))
    }

    /// The table starts at `rn` and occupies `list_len` consecutive registers.
    pub fn tbx_v<V: VRegBytes>(
        &mut self,
        rd: V,
        rn: VReg16B,
        rm: V,
        list_len: u8,
    ) -> Result<(), String> {
        self.emit(inst::tbx_v(
            rd.index(),
            rn.index(),
            rm.index(),
            list_len,
            V::Q,
        ))
    }
}
