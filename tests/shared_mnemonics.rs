use rhazel::*;

#[test]
fn shared_a32_and_prelude_overloads_match_clang() {
    let mut block = BlockOfCode::with_size(4096).unwrap();
    let mut code = CodeGenerator::new(&mut block);
    code.orr_lsl(W1, W2, W3, 17).unwrap();
    code.orr_lsr(W4, W5, W6, 31).unwrap();
    code.bfxil(W7, W8, 3, 29).unwrap();
    code.sbfm(X9, X10, 7, 42).unwrap();
    code.ldaxr(W11, SP).unwrap();
    code.ldaxr(X12, X13).unwrap();
    code.stlxr(W14, W15, SP).unwrap();
    code.stlxr(W16, X17, X18).unwrap();
    // Independent Clang ARM64 assembly, not the raw encoders under test.
    let expected = [
        0x2a03_4441,
        0x2a46_7ca4,
        0x3303_7d07,
        0x9347_a949,
        0x885f_ffeb,
        0xc85f_fdac,
        0x880e_ffef,
        0xc810_fe51,
    ];
    assert_eq!(code.code_size(), expected.len() * 4);
    for (index, expected) in expected.into_iter().enumerate() {
        let actual = unsafe {
            code.code_base_ptr()
                .add(index * 4)
                .cast::<u32>()
                .read_unaligned()
        };
        assert_eq!(actual, expected, "instruction {index}");
    }
}
