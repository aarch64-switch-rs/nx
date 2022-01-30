// Note: this is a slightly modified version of libbio's CRT0 (https://github.com/biosphere-switch/libbio)

.section .text.jmp, "ax"

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
	// Get pointer to MOD0 struct (contains offsets to important places)
    adr x28, __module_header

    // Calculate BSS address/size
    ldp  w8, w9, [x28, #8] // load BSS start/end offset from MOD0
    sub  w9, w9, w8        // calculate BSS size
    add  w9, w9, #7        // round up to 8
    bic  w9, w9, #7        // ^
    add  x8, x28, x8       // fixup the start pointer

    // Clear the BSS in 8-byte units
__clear_bss_loop:
    subs w9, w9, #8
    str  xzr, [x8], #8
    bne  __clear_bss_loop
	
	// Set aslr base address as 3rd argument
	adrp x2, _start

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