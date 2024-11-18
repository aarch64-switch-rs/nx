.section .text.jmp, "ax", %progbits
.align 2

.global _start
_start:
	b trampoline
	.word __module_header - _start
	.ascii "HOMEBREW"

.section .text.trampoline, "ax", %progbits
.align 2
.type trampoline, %function
.org _start+0x40; trampoline:
    // Arguments on NSO entry:
    //   x0=zero                  | x1=main thread handle
    // Arguments on NRO entry (homebrew ABI):
    //   x0=ptr to env context    | x1=UINT64_MAX (-1 aka 0xFFFFFFFFFFFFFFFF)
    // Arguments on user-mode exception entry:
    //   x0=excpt type (non-zero) | x1=ptr to excpt context

    // Detect and handle user-mode exceptions first:
    // if (x0 != 0 && x1 != UINT64_MAX) __nx_exception_entry(<inargs>);
    cmp  x0, #0
    ccmn x1, #1, #4, ne // 4 = Z
    beq  normal        
    b    __nx_exception_dispatch
normal:
    b __nx_rrt0_entry