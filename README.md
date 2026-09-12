# rhazel

AArch64 assembler library for [ruzu](https://github.com/vricosti/ruzu-emu)'s
rdynarmic arm64 backend — the counterpart of [oaknut](https://github.com/merryhime/oaknut),
which upstream dynarmic links for the same purpose.

It owns instruction encoding (`inst`), executable code buffers
(`BlockOfCode`) and branch labels (`Label`, including the `TBZ + B` fallback
for forward `TBNZ` targets beyond imm14). IR emission stays in rdynarmic.
