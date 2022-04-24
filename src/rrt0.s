.section .text.rrt0, "ax", %progbits
.align 2

.global _start
_start:
	b __nx_rrt0_entry
	.word __nx_mod0_header - _start