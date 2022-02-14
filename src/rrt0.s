// Note: this is a slightly modified version of libbio's CRT0 (https://github.com/biosphere-switch/libbio)

.section .text.rrt0, "ax"

.global _start
_start:
	b _entry
	.word __module_header - _start

.section .data.mod0

.global __module_header
__module_header:
	.ascii "MOD0"
	.word __dynamic_start - __module_header
	.word __bss_start - __module_header
	.word __bss_end - __module_header
	.word __eh_frame_hdr_start - __module_header
	.word __eh_frame_hdr_end - __module_header
	.word 0 // Runtime-generated module object offset, unused

.section .text, "ax"

__default_entry:
	// Clean BSS first
	adrp x5, __bss_start
	add x5, x5, #:lo12:__bss_start
	adrp x6, __bss_end
	add x6, x6, #:lo12:__bss_end
__clean_bss_loop:
	cmp x5, x6
	b.eq __default_entry_start
	str xzr, [x5]
	add x5, x5, 8
	b __clean_bss_loop

__default_entry_start:
	// Set aslr base address as 3rd argument
	adr x2, _start

	// Set loader return address as 4th argument
	mov x3, x30

	// Call the normal entrypoint (implemented in Rust)
	b __nx_rrt0_entry

__exception_entry:
	// Call the exception entrypoint (implemented in Rust)
	b __nx_rrt0_exception_entry

// Actual entrypoint called

_entry:
	// Determine which entry we need to call (if x0 != 0 and x1 != -1, we're being called to handle an exception)
	cmp x0, #0
    ccmn x1, #1, #4, ne
    beq __default_entry
    bne __exception_entry