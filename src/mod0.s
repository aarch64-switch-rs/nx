.section .text.mod0, "ax", %progbits
.align 2

.global __nx_mod0_header // Same as nx::elf::mod0::Header type
__nx_mod0_header:
	.ascii "MOD0"
	.word __dynamic_start - __nx_mod0_header
	.word __bss_start - __nx_mod0_header
	.word __bss_end - __nx_mod0_header
	.word __eh_frame_hdr_start - __nx_mod0_header
	.word __eh_frame_hdr_end - __nx_mod0_header
	.word 0 // Runtime-generated module object offset, unused (we don't work with dynamically linked modules like N)