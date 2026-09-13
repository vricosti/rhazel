use rhazel::{BlockOfCode, CodeGenerator};

#[test]
fn overflowing_patch_end_is_rejected_before_writing() {
    for deferred in [false, true] {
        let mut block = BlockOfCode::with_size(4096).unwrap();
        let mut patch = CodeGenerator::patch_at(&mut block, usize::MAX - 3, deferred);
        assert!(patch.nop().is_err());
        assert_eq!(patch.code_size(), usize::MAX - 3);
        assert_eq!(block.code_size(), 0);
    }
}
