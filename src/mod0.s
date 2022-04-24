.section .text.mod0, "ax", %progbits
.align 2

.global __nx_module_header
__nx_module_header:
	.ascii "MOD0"
	.word __dynamic_start - __nx_module_header
	.word __bss_start - __nx_module_header
	.word __bss_end - __nx_module_header
	.word __eh_frame_hdr_start - __nx_module_header
	.word __eh_frame_hdr_end - __nx_module_header
	.word 0 // Runtime-generated module object offset, unused