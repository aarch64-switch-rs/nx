FN_START __nx_arm_cache_flush
	add x1, x1, x0
	mrs x8, CTR_EL0
	lsr x8, x8, #16
	and x8, x8, #0xf
	mov x9, #4
	lsl x9, x9, x8
	sub x10, x9, #1
	bic x8, x0, x10
	mov x10, x1

	// Set flag at TLR[0x104] for kernel
	mov w1, #1
	mrs x0, tpidrro_el0
	strb w1, [x0, #0x104] 

armDCacheFlush_L0:
	dc  civac, x8
	add x8, x8, x9
	cmp x8, x10
	bcc armDCacheFlush_L0

	dsb sy

	// Unset flag at TLR[0x104] for kernel
	strb wzr, [x0, #0x104]

	ret
FN_END