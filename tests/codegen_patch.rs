//! AddressSpace::Link/LinkBlockLinks use a separate Oaknut patch pointer.
//! RelinkForDescriptor owns range invalidation, not the patch generator.

use rhazel::{BlockOfCode, CodeGenerator, Cond, Label, W0, X0, X17};
use std::panic::{catch_unwind, AssertUnwindSafe};

const NOP: u32 = 0xd503_201f;
const RET: u32 = 0xd65f_03c0;

#[test]
fn patch_label_operations_fail_before_code_cursor_or_label_mutation() {
    for deferred in [false, true] {
        for label_state in 0..3 {
            for operation in 0..8 {
                let mut block = block_with_nops(4096, 8);
                let mut label = Label::new();
                match label_state {
                    0 => {}
                    1 => {
                        CodeGenerator::new(&mut block).b(&mut label).unwrap();
                    }
                    2 => {
                        CodeGenerator::new(&mut block).l(&mut label).unwrap();
                    }
                    _ => unreachable!(),
                }
                let append_size = block.code_size();
                let before = bytes(&block, 0, append_size);
                // Debug exposes both the bound offset and pending fixups.
                let label_before = format!("{label:?}");
                {
                    let mut patch = CodeGenerator::patch_at(&mut block, 4, deferred);
                    let result = match operation {
                        0 => patch.l(&mut label),
                        1 => patch.b(&mut label),
                        2 => patch.b_cond(Cond::EQ, &mut label),
                        3 => patch.cbz(W0, &mut label),
                        4 => patch.cbz(X0, &mut label),
                        5 => patch.cbnz(W0, &mut label),
                        6 => patch.cbnz(X0, &mut label),
                        7 => patch.tbnz(X0, 0, &mut label),
                        _ => unreachable!(),
                    };
                    assert_eq!(
                        result.unwrap_err(),
                        "ARM64 patch generator does not support label operations"
                    );
                    assert_eq!(patch.code_size(), 4);
                }
                assert_eq!(block.code_size(), append_size);
                assert_eq!(bytes(&block, 0, append_size), before);
                assert_eq!(format!("{label:?}"), label_before);
            }
        }
    }
}

#[test]
fn mutable_deref_cannot_bypass_patch_label_guards() {
    for deferred in [false, true] {
        for operation in 0..3 {
            let mut block = block_with_nops(4096, 8);
            let mut label = Label::new();
            CodeGenerator::new(&mut block).b(&mut label).unwrap();
            let append_size = block.code_size();
            let before = bytes(&block, 0, append_size);
            let label_before = format!("{label:?}");
            {
                let mut patch = CodeGenerator::patch_at(&mut block, 4, deferred);
                let panic = catch_unwind(AssertUnwindSafe(|| {
                    match operation {
                        // These coerce through DerefMut rather than generator methods.
                        0 => {
                            label.b(&mut patch).unwrap();
                        }
                        1 => {
                            label.bind(&mut patch).unwrap();
                        }
                        2 => {
                            patch.write_u32(RET).unwrap();
                        }
                        _ => unreachable!(),
                    }
                }))
                .unwrap_err();
                assert_eq!(
                    panic.downcast_ref::<&str>().copied(),
                    Some("ARM64 patch generator cannot mutably dereference the append buffer")
                );
                assert_eq!(patch.code_size(), 4);
            }
            assert_eq!(block.code_size(), append_size);
            assert_eq!(bytes(&block, 0, append_size), before);
            assert_eq!(format!("{label:?}"), label_before);
        }
    }
}

#[test]
fn append_generator_label_operations_and_mutable_deref_still_work() {
    let mut block = BlockOfCode::with_size(4096).unwrap();
    let mut label = Label::new();
    {
        let mut code = CodeGenerator::new(&mut block);
        code.b(&mut label).unwrap();
        code.b_cond(Cond::EQ, &mut label).unwrap();
        code.cbz(W0, &mut label).unwrap();
        code.cbz(X0, &mut label).unwrap();
        code.cbnz(W0, &mut label).unwrap();
        code.cbnz(X0, &mut label).unwrap();
        code.tbnz(X0, 0, &mut label).unwrap();
        code.l(&mut label).unwrap();
        assert_eq!(code.code_size(), 32);
        code.write_u32(RET).unwrap();
    }
    let expected: [u32; 9] = [
        0x1400_0008,
        0x5400_00e0,
        0x3400_00c0,
        0xb400_00a0,
        0x3500_0080,
        0xb500_0060,
        0x3600_0040,
        0x1400_0001,
        RET,
    ];
    assert_eq!(block.code_size(), 36);
    for (i, expected) in expected.into_iter().enumerate() {
        assert_eq!(word(&block, i * 4), expected);
    }
}

fn block_with_nops(size: usize, count: usize) -> BlockOfCode {
    let mut block = BlockOfCode::with_size(size).unwrap();
    for _ in 0..count {
        CodeGenerator::new(&mut block).nop().unwrap();
    }
    block
}

fn word(block: &BlockOfCode, offset: usize) -> u32 {
    assert!(offset + 4 <= block.total_size());
    // Only called on initialized instruction slots in this test suite.
    unsafe {
        block
            .code_base_ptr()
            .add(offset)
            .cast::<u32>()
            .read_unaligned()
    }
}

fn bytes(block: &BlockOfCode, offset: usize, size: usize) -> Vec<u8> {
    assert!(offset + size <= block.total_size());
    unsafe { std::slice::from_raw_parts(block.code_base_ptr().add(offset), size).to_vec() }
}

#[test]
fn patch_branches_use_each_patch_pc_and_preserve_append_cursor() {
    for deferred in [false, true] {
        let mut block = block_with_nops(4096, 32);
        let base = block.code_base_ptr();
        let before = bytes(&block, 0, 128);
        {
            let mut patch = CodeGenerator::patch_at(&mut block, 4, deferred);
            assert_eq!(patch.code_size(), 4);
            patch.b_to(base.wrapping_add(32)).unwrap();
            assert_eq!(patch.code_size(), 8);
            patch.bl_to(base).unwrap();
            assert_eq!(patch.code_size(), 12);
            patch.ret().unwrap();
            assert_eq!(patch.code_size(), 16);
        }
        assert_eq!(block.code_size(), 128);
        // B: (32 - 4) / 4 = 7. BL: (0 - 8) / 4 = -2.
        assert_eq!(
            bytes(&block, 4, 12),
            [0x07, 0x00, 0x00, 0x14, 0xfe, 0xff, 0xff, 0x97, 0xc0, 0x03, 0x5f, 0xd6,]
        );
        assert_eq!(bytes(&block, 0, 4), before[..4]);
        assert_eq!(bytes(&block, 16, 112), before[16..]);
        let mut append = CodeGenerator::new(&mut block);
        assert_eq!(append.code_size(), 128);
        append.b_to(base.wrapping_add(128)).unwrap();
        append.bl_to(base.wrapping_add(128)).unwrap();
        assert_eq!(append.code_size(), 136);
        assert_eq!(word(&block, 128), 0x1400_0000);
        assert_eq!(word(&block, 132), 0x97ff_ffff);
    }
}

#[test]
fn adrl_uses_first_instruction_page_not_append_or_second_instruction_page() {
    for deferred in [false, true] {
        let mut block = block_with_nops(0x4000, 0x3000 / 4);
        let base = block.code_base_ptr();
        assert_eq!(base as usize & 0xfff, 0, "mmap page alignment");
        {
            let mut patch = CodeGenerator::patch_at(&mut block, 0xffc, deferred);
            patch.adrl(X17, base.wrapping_add(0x3123)).unwrap();
            assert_eq!(patch.code_size(), 0x1004);
            patch.adrl(X17, base.wrapping_add(0xabc)).unwrap();
            assert_eq!(patch.code_size(), 0x100c);
        }
        assert_eq!(block.code_size(), 0x3000);
        // Oaknut ADRL = ADRP + ADD, even when an ADR would fit.
        // Page deltas +3 and -1: immlo in [30:29], immhi in [23:5].
        assert_eq!(word(&block, 0xffc), 0xf000_0011);
        assert_eq!(word(&block, 0x1000), 0x9104_8e31);
        assert_eq!(word(&block, 0x1004), 0xf0ff_fff1);
        assert_eq!(word(&block, 0x1008), 0x912a_f231);
        assert_eq!(word(&block, 0xff8), NOP);
        assert_eq!(word(&block, 0x100c), NOP);
        CodeGenerator::new(&mut block)
            .adrl(X17, base.wrapping_add(0xfff))
            .unwrap();
        assert_eq!(block.code_size(), 0x3008);
        assert_eq!(word(&block, 0x3000), 0xb0ff_fff1); // -3 pages
        assert_eq!(word(&block, 0x3004), 0x913f_fe31); // low 12 bits = 0xfff
    }
}

#[test]
fn invalid_patch_slots_preserve_existing_errors_bytes_and_both_cursors() {
    for deferred in [false, true] {
        let mut block = block_with_nops(4096, 4);
        let before = bytes(&block, 0, 16);
        for offset in [1, 2, 4094, 4096, 4100] {
            let existing_error = if deferred {
                block.patch_u32_deferred_icache(offset, RET)
            } else {
                block.patch_u32(offset, RET)
            }
            .unwrap_err();
            {
                let mut patch = CodeGenerator::patch_at(&mut block, offset, deferred);
                // Construction is lazy; failed emission must not advance the patch PC.
                assert_eq!(patch.code_size(), offset);
                assert_eq!(patch.ret().unwrap_err(), existing_error);
                assert_eq!(patch.code_size(), offset);
            }
            assert_eq!(block.code_size(), 16);
            assert_eq!(bytes(&block, 0, 16), before);
        }
    }
}

#[test]
fn allocated_slots_beyond_append_cursor_are_still_patchable() {
    for deferred in [false, true] {
        let mut block = block_with_nops(4096, 1);
        let base = block.code_base_ptr();
        // Existing BlockOfCode validation uses allocation capacity, not code_size.
        block.patch_u32(64, NOP).unwrap();
        {
            let mut patch = CodeGenerator::patch_at(&mut block, 64, deferred);
            patch.b_to(base).unwrap();
            assert_eq!(patch.code_size(), 68);
        }
        assert_eq!(block.code_size(), 4);
        assert_eq!(word(&block, 0), NOP);
        assert_eq!(word(&block, 64), 0x17ff_fff0);
    }
}

#[test]
fn adrl_at_last_slot_keeps_existing_nontransactional_write_behavior() {
    for deferred in [false, true] {
        let mut block = block_with_nops(4096, 4096 / 4);
        let target = block.code_base_ptr();
        {
            let mut patch = CodeGenerator::patch_at(&mut block, 4092, deferred);
            let error = patch.adrl(X17, target).unwrap_err();
            assert_eq!(error, "ARM64 patch offset out of code cache range: 4096");
            // ADRP succeeded before ADD failed. No implied atomic two-word patch.
            assert_eq!(patch.code_size(), 4096);
        }
        assert_eq!(block.code_size(), 4096);
        assert_eq!(word(&block, 4088), NOP);
        assert_eq!(word(&block, 4092), 0x9000_0011);
    }
}

#[test]
fn branch_boundaries_and_invalid_targets_keep_original_encoder_contract() {
    let mut block = block_with_nops(4096, 8);
    let pc = block.code_base_ptr().wrapping_add(4);
    for (delta, expected_b, expected_bl) in [
        (-(1isize << 27), 0x1600_0000, 0x9600_0000),
        ((1isize << 27) - 4, 0x15ff_ffff, 0x95ff_ffff),
    ] {
        let target = pc.wrapping_offset(delta);
        CodeGenerator::patch_at(&mut block, 4, true)
            .b_to(target)
            .unwrap();
        assert_eq!(word(&block, 4), expected_b);
        CodeGenerator::patch_at(&mut block, 4, true)
            .bl_to(target)
            .unwrap();
        assert_eq!(word(&block, 4), expected_bl);
    }
    for delta in [2, 1isize << 27, -(1isize << 27) - 4] {
        for link in [false, true] {
            let before = word(&block, 4);
            let mut patch = CodeGenerator::patch_at(&mut block, 4, true);
            let result = catch_unwind(AssertUnwindSafe(|| {
                if link {
                    patch.bl_to(pc.wrapping_offset(delta))
                } else {
                    patch.b_to(pc.wrapping_offset(delta))
                }
            }));
            assert!(
                result.is_err(),
                "legacy encoder rejects invalid imm26 by panic"
            );
            assert_eq!(patch.code_size(), 4);
            assert_eq!(word(&block, 4), before);
            assert_eq!(block.code_size(), 32);
        }
    }
}

#[test]
fn deferred_patch_batch_is_published_by_the_range_owner() {
    let mut block = block_with_nops(4096, 8);
    block.seal();
    block.unprotect();
    for (offset, value) in [(0, 7), (16, 11)] {
        let mut patch = CodeGenerator::patch_at(&mut block, offset, true);
        patch.movz(X0, value, 0).unwrap();
        patch.ret().unwrap();
    }
    assert_eq!(block.code_size(), 32);
    assert_eq!(word(&block, 0), 0xd280_00e0);
    assert_eq!(word(&block, 16), 0xd280_0160);
    // AddressSpace batches modified ranges. Do not execute deferred writes before
    // the owner publishes them; byte visibility alone does not prove an I-cache flush.
    block.seal_ranges(&[(0, 8), (16, 8)]);
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let first: extern "C" fn() -> u64 = std::mem::transmute(block.code_base_ptr());
        let second: extern "C" fn() -> u64 = std::mem::transmute(block.code_base_ptr().add(16));
        assert_eq!(first(), 7);
        assert_eq!(second(), 11);
    }
}

#[test]
fn immediate_patch_is_publishable_without_an_owner_seal() {
    let mut block = block_with_nops(4096, 2);
    {
        let mut append = CodeGenerator::patch_at(&mut block, 0, true);
        append.movz(X0, 1, 0).unwrap();
        append.ret().unwrap();
    }
    block.seal();
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let call: extern "C" fn() -> u64 = std::mem::transmute(block.code_base_ptr());
        assert_eq!(call(), 1);
    }
    CodeGenerator::patch_at(&mut block, 0, false)
        .movz(X0, 42, 0)
        .unwrap();
    assert_eq!(block.code_size(), 8);
    assert_eq!(word(&block, 0), 0xd280_0540);
    assert_eq!(word(&block, 4), RET);
    // No seal here: the immediate patch writer owns publication and restores the
    // prior write-protection state. Execution is meaningful only on an ARM64 host.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let call: extern "C" fn() -> u64 = std::mem::transmute(block.code_base_ptr());
        assert_eq!(call(), 42);
    }
}
