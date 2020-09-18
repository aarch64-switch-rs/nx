FN_START __nx_svc_set_heap_size
	str x0, [sp, #-16]!
	svc 0x1
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_set_memory_attribute
	svc 0x3
	ret
FN_END

FN_START __nx_svc_query_memory
	str x1, [sp, #-16]!
	svc 0x6
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_exit_process
	svc 0x7
	ret
FN_END

FN_START __nx_svc_create_thread
	str x0, [sp, #-16]!
	svc 0x8
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_start_thread
	svc 0x9
	ret
FN_END

FN_START __nx_svc_exit_thread
	svc 0xA
	ret
FN_END

FN_START __nx_svc_sleep_thread
	svc 0xB
	ret
FN_END

FN_START __nx_svc_get_thread_priority
	str x0, [sp, #-16]!
	svc 0xC
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_signal_event
	svc 0x11
	ret
FN_END

FN_START __nx_svc_map_shared_memory
	svc 0x13
	ret
FN_END

FN_START __nx_svc_unmap_shared_memory
	svc 0x14
	ret
FN_END

FN_START __nx_svc_create_transfer_memory
	str x0, [sp, #-16]!
	svc 0x15
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_close_handle
	svc 0x16
	ret
FN_END

FN_START __nx_svc_reset_signal
	svc 0x17
	ret
FN_END

FN_START __nx_svc_wait_synchronization
	str x0, [sp, #-16]!
	svc 0x18
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_arbitrate_lock
	svc 0x1A
	ret
FN_END

FN_START __nx_svc_arbitrate_unlock
	svc 0x1B
	ret
FN_END

FN_START __nx_svc_connect_to_named_port
	str x0, [sp, #-16]!
	svc 0x1F
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_send_sync_request
	svc 0x21
	ret
FN_END

FN_START __nx_svc_get_process_id
	str x0, [sp, #-16]!
	svc 0x24
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_get_thread_id
	str x0, [sp, #-16]!
	svc 0x25
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_break
	svc 0x26
	ret
FN_END

FN_START __nx_svc_output_debug_string
	svc 0x27
	ret
FN_END

FN_START __nx_svc_return_from_exception
	svc 0x28
	ret
FN_END

FN_START __nx_svc_get_info
	str x0, [sp, #-16]!
	svc 0x29
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_create_session
	stp x0, x1, [sp, #-16]!
	svc 0x40
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str w2, [x4]
	ret
FN_END

FN_START __nx_svc_accept_session
	str x0, [sp, #-16]!
	svc 0x41
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_reply_and_receive
	str x0, [sp, #-16]!
	svc 0x43
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_create_event
	stp x0, x1, [sp, #-16]!
	svc 0x45
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str w2, [x4]
	ret
FN_END

FN_START __nx_svc_manage_named_port
	str x0, [sp, #-16]!
	svc 0x71
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_call_secure_monitor
	str x0, [sp, #-16]!
	mov x8, x0
	ldp x0, x1, [x8]
	ldp x2, x3, [x8, #0x10]
	ldp x4, x5, [x8, #0x20]
	ldp x6, x7, [x8, #0x30]
	svc 0x7F
	ldr x8, [sp], #16
	stp x0, x1, [x8]
	stp x2, x3, [x8, #0x10]
	stp x4, x5, [x8, #0x20]
	stp x6, x7, [x8, #0x30]
	ret
FN_END