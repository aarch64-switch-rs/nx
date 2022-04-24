// Note: using "{{" and "}}" since global_asm! requires it to properly parse them...

FN_START __nx_svc_set_heap_size
	str r0, [sp, #-0x4]!
    svc 0x1
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_memory_permission
	svc 0x2
	bx lr
FN_END

FN_START __nx_svc_set_memory_attribute
	svc 0x3
	bx lr
FN_END

FN_START __nx_svc_map_memory
	svc 0x4
	bx lr
FN_END

FN_START __nx_svc_unmap_memory
	svc 0x5
	bx lr
FN_END

FN_START __nx_svc_query_memory
    str r1, [sp, #-0x4]!
    svc 0x6
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_exit_process
	svc 0x7
	bx lr
FN_END

FN_START __nx_svc_create_thread
	stmfd sp!, {{r0, r4}}
    ldr r0, [sp, #0x8]
    ldr r4, [sp, #0xC]
    svc 0x8
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_start_thread
	svc 0x9
	bx lr
FN_END

FN_START __nx_svc_exit_thread
	svc 0xA
	bx lr
FN_END

FN_START __nx_svc_sleep_thread
	svc 0xB
	bx lr
FN_END

FN_START __nx_svc_get_thread_priority
    str r0, [sp, #-0x4]!
    svc 0xC
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_thread_priority
	svc 0xD
	bx lr
FN_END

FN_START __nx_svc_get_thread_core_mask
	stmfd sp!, {{r0, r1, r4}}
    svc 0xE
    ldr r4, [sp]
    str r1, [r4]
    ldr r4, [sp, #0x4]
    str r2, [r4]
    str r3, [r4, #0x4]
    add sp, sp, #0x4
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_set_thread_core_mask
	svc 0xF
	bx lr
FN_END

FN_START __nx_svc_get_current_processor_number
	svc 0x10
	bx lr
FN_END

FN_START __nx_svc_signal_event
	svc 0x11
	bx lr
FN_END

FN_START __nx_svc_clear_event
	svc 0x12
	bx lr
FN_END

FN_START __nx_svc_map_shared_memory
	svc 0x13
	bx lr
FN_END

FN_START __nx_svc_unmap_shared_memory
	svc 0x14
	bx lr
FN_END

FN_START __nx_svc_create_transfer_memory
    str r0, [sp, #-0x4]!
    svc 0x15
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_close_handle
	svc 0x16
	bx lr
FN_END

FN_START __nx_svc_reset_signal
	svc 0x17
	bx lr
FN_END

FN_START __nx_svc_wait_synchronization
	str r0, [sp, #-0x4]!
    ldr r0, [sp, #0x4]
    ldr r3, [sp, #0x8]
    svc 0x18
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_cancel_synchronization
	svc 0x19
	bx lr
FN_END

FN_START __nx_svc_arbitrate_lock
	svc 0x1A
	bx lr
FN_END

FN_START __nx_svc_arbitrate_unlock
	svc 0x1B
	bx lr
FN_END

FN_START __nx_svc_wait_process_wide_key_atomic
	str r4, [sp, #-0x4]!
    ldr r3, [sp, #0x4]
    ldr r4, [sp, #0x8]
    svc 0x1C
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_signal_process_wide_key
	svc 0x1D
	bx lr
FN_END

FN_START __nx_svc_get_system_tick
	svc 0x1E
	bx lr
FN_END

FN_START __nx_svc_connect_to_named_port
	str r0, [sp, #-0x4]!
    svc 0x1F
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_send_sync_request_light
	svc 0x20
	bx lr
FN_END

FN_START __nx_svc_send_sync_request
	svc 0x21
	bx lr
FN_END

FN_START __nx_svc_send_sync_request_with_user_buffer
	svc 0x22
	bx lr
FN_END

FN_START __nx_svc_send_async_request_with_user_buffer
	str r0, [sp, #-0x4]!
    svc 0x23
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_get_process_id
	str r0, [sp, #-0x4]!
    svc 0x24
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_get_thread_id
	str r0, [sp, #-0x4]!
    svc 0x25
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_break
	svc 0x26
	bx lr
FN_END

FN_START __nx_svc_output_debug_string
	svc 0x27
	bx lr
FN_END

FN_START __nx_svc_return_from_exception
	svc 0x28
	bx lr
FN_END

FN_START __nx_svc_get_info
	str r0, [sp, #-0x4]!
    ldr r0, [sp, #0x4]
    ldr r3, [sp, #0x8]
    svc 0x29
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_flush_entire_data_cache
	svc 0x2A
	bx lr
FN_END

FN_START __nx_svc_flush_data_cache
	svc 0x2B
	bx lr
FN_END

FN_START __nx_svc_map_physical_memory
	svc 0x2C
	bx lr
FN_END

FN_START __nx_svc_unmap_physical_memory
	svc 0x2D
	bx lr
FN_END

// TODO: __nx_svc_get_future_thread_info (0x2E)

// TODO: __nx_svc_get_debug_future_thread_info (0x2E)

// TODO: __nx_svc_get_last_thread_info (0x2f)

FN_START __nx_svc_get_resource_limit_limit_value
	str r0, [sp, #-0x4]!
    svc 0x30
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_get_resource_limit_current_value
	str r0, [sp, #-0x4]!
    svc 0x31
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_thread_activity
	svc 0x32
	bx lr
FN_END

// TODO: __nx_svc_get_thread_context3 (0x33)

FN_START __nx_svc_wait_for_address
	svc 0x34
	bx lr
FN_END

FN_START __nx_svc_signal_to_address
	svc 0x35
	bx lr
FN_END

FN_START __nx_svc_synchronize_preemption_state
	svc 0x36
	bx lr
FN_END

// TODO: __nx_svc_get_resource_limit_peak_value (0x37)

// TODO: __nx_svc_create_io_pool (0x39)

// TODO: __nx_svc_create_io_region (0x3A)

FN_START __nx_svc_dump_info
	svc 0x3C
	bx lr
FN_END

FN_START __nx_svc_kernel_debug
	svc 0x3C
	bx lr
FN_END

FN_START __nx_svc_change_kernel_trace_state
	svc 0x3D
	bx lr
FN_END

FN_START __nx_svc_create_session
	stmfd sp!, {{r0, r1}}
    svc 0x40
    ldr r3, [sp]
    str r1, [r3]
    ldr r3, [sp, #0x4]
    str r2, [r3]
    add sp, sp, #0x8
    bx lr
FN_END

FN_START __nx_svc_accept_session
	str r0, [sp, #-0x4]!
    svc 0x41
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_reply_and_receive_light
	svc 0x42
	bx lr
FN_END

FN_START __nx_svc_reply_and_receive
	stmfd sp!, {{r0, r4}}
    ldr r0, [sp, #0x8]
    ldr r4, [sp, #0xC]
    svc 0x43
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_reply_and_receive_with_user_buffer
	stmfd sp!, {{r0, r4-r6}}
    ldr r0, [sp, #0x10]
    ldr r4, [sp, #0x14]
    ldr r5, [sp, #0x18]
    ldr r6, [sp, #0x1C]
    svc 0x44
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #4
    ldmfd sp!, {{r4-r6}}
    bx lr
FN_END

FN_START __nx_svc_create_event
	stmfd sp!, {{r0, r1}}
    svc 0x45
    ldr r3, [sp]
    str r1, [r3]
    ldr r3, [sp, #0x4]
    str r2, [r3]
    add sp, sp, #0x8
    bx lr
FN_END

FN_START __nx_svc_map_io_region
	svc 0x48
	bx lr
FN_END

FN_START __nx_svc_unmap_io_region
	svc 0x49
	bx lr
FN_END

FN_START __nx_svc_map_physical_memory_unsafe
	svc 0x48
	bx lr
FN_END

FN_START __nx_svc_unmap_physical_memory_unsafe
	svc 0x49
	bx lr
FN_END

FN_START __nx_svc_set_unsafe_limit
	svc 0x4A
	bx lr
FN_END

// TODO: __nx_svc_create_code_memory (0x4B)

FN_START __nx_svc_control_code_memory
	svc 0x4C
	bx lr
FN_END

FN_START __nx_svc_sleep_system
	svc 0x4D
	bx lr
FN_END

FN_START __nx_svc_read_write_register
	str r0, [sp, #-0x4]!
    ldr r0, [sp, #0x4]
    ldr r1, [sp, #0x8]
    svc 0x4E
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_process_activity
	svc 0x4F
	bx lr
FN_END

FN_START __nx_svc_create_shared_memory
    str r0, [sp, #-0x4]!
    svc 0x50
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_map_transfer_memory
	svc 0x51
	bx lr
FN_END

FN_START __nx_svc_unmap_transfer_memory
	svc 0x52
	bx lr
FN_END

FN_START __nx_svc_create_interrupt_event
	str r0, [sp, #-0x4]!
    svc 0x53
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

// TODO: __nx_svc_query_physical_address (0x54)

// TODO: __nx_svc_query_io_mapping (0x55)

FN_START __nx_svc_legacy_query_io_mapping
	str r0, [sp, #-0x4]!
    ldr r0, [sp, #0x4]
    svc 0x55
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_create_device_address_space
	str r0, [sp, #-0x4]!
    ldr r0, [sp, #0x4]
    svc 0x55
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_attach_device_address_space
	svc 0x57
	bx lr
FN_END

FN_START __nx_svc_detach_device_address_space
	svc 0x58
	bx lr
FN_END

FN_START __nx_svc_map_device_address_space_by_force
	stmfd sp!, {{r4-r7}}
    ldr r4, [sp, #0x10]
    ldr r5, [sp, #0x18]
    ldr r6, [sp, #0x1C]
    ldr r7, [sp, #0x20]
    svc 0x59
    ldmfd sp!, {{r4-r7}}
    bx lr
FN_END

FN_START __nx_svc_map_device_address_space_aligned
	stmfd sp!, {{r4-r7}}
    ldr r4, [sp, #0x10]
    ldr r5, [sp, #0x18]
    ldr r6, [sp, #0x1C]
    ldr r7, [sp, #0x20]
    svc 0x5A
    ldmfd sp!, {{r4-r7}}
    bx lr
FN_END

FN_START __nx_svc_map_device_address_space
	stmfd sp!, {{r0, r4-r7}}
    ldr r0, [sp, #0x14]
    ldr r3, [sp, #0x18]
    ldr r4, [sp, #0x1C]
    ldr r5, [sp, #0x24]
    ldr r6, [sp, #0x28]
    ldr r7, [sp, #0x2C]
    svc 0x5B
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    ldmfd sp!, {{r4-r7}}
    bx lr
FN_END

FN_START __nx_svc_unmap_device_address_space
	stmfd sp!, {{r4-r6}}
    ldr r4, [sp, #0xC]
    ldr r5, [sp, #0x14]
    ldr r6, [sp, #0x18]
    svc 0x5C
    ldmfd sp!, {{r4-r6}}
    bx lr
FN_END

FN_START __nx_svc_invalidate_process_data_cache
	str r4, [sp, #-0x4]!
    ldr r1, [sp, #0x4]
    ldr r4, [sp, #0x8]
    svc 0x5D
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_store_process_data_cache
	str r4, [sp, #-0x4]!
    ldr r1, [sp, #0x4]
    ldr r4, [sp, #0x8]
    svc 0x5E
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_flush_process_data_cache
	str r4, [sp, #-0x4]!
    ldr r1, [sp, #0x4]
    ldr r4, [sp, #0x8]
    svc 0x5F
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_debug_active_process
	str r0, [sp, #-0x4]!
    svc 0x60
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_break_debug_process
	svc 0x61
	bx lr
FN_END

FN_START __nx_svc_terminate_debug_process
	svc 0x62
	bx lr
FN_END

// TODO: __nx_svc_get_debug_event (0x63)

// TODO: __nx_svc_legacy_continue_debug_event (0x64)

// TODO: __nx_svc_continue_debug_event (0x64)

FN_START __nx_svc_get_process_list
	str r0, [sp, #-0x4]!
    svc 0x65
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_get_thread_list
	str r0, [sp, #-0x4]!
    svc 0x66
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

// TODO: __nx_svc_get_debug_thread_context (0x67)

// TODO: __nx_svc_set_debug_thread_context (0x68)

// TODO: __nx_svc_query_debug_process_memory (0x69)

// TODO: __nx_svc_read_debug_process_memory (0x6A)

// TODO: __nx_svc_write_debug_process_memory (0x6B)

// TODO: __nx_svc_set_hardware_break_point (0x6C)

// TODO: __nx_svc_get_debug_thread_param (0x6D)

// TODO: __nx_svc_get_system_info (0x6F)

FN_START __nx_svc_create_port
	stmfd sp!, {{r0, r1}}
    ldr r0, [sp, #0x8]
    svc 0x70
    ldr r3, [sp]
    str r1, [r3]
    ldr r3, [sp, #0x4]
    str r2, [r3]
    add sp, sp, #0x8
    bx lr
FN_END

FN_START __nx_svc_manage_named_port
	str r0, [sp, #-0x4]!
    svc 0x71
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_connect_to_port
	str r0, [sp, #-0x4]!
    svc 0x72
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_process_memory_permission
	stmfd sp!, {{r4, r5}}
    ldr r1, [sp, #0x8]
    ldr r4, [sp, #0xC]
    ldr r5, [sp, #0x10]
    svc 0x73
    ldmfd sp!, {{r4, r5}}
    bx lr
FN_END

FN_START __nx_svc_map_process_memory
	str r4, [sp, #-0x4]!
    ldr r4, [sp, #0x4]
    svc 0x74
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_unmap_process_memory
	str r4, [sp, #-0x4]!
    ldr r4, [sp, #0x4]
    svc 0x75
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_query_process_memory
	str r1, [sp, #-0x4]!
    ldr r1, [sp, #0x4]
    ldr r3, [sp, #0x8]
    svc 0x76
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_map_process_code_memory
	stmfd sp!, {{r4-r6}}
    ldr r1, [sp, #0xC]
    ldr r4, [sp, #0x10]
    ldr r5, [sp, #0x14]
    ldr r6, [sp, #0x18]
    svc 0x77
    ldmfd sp!, {{r4-r6}}
    bx lr
FN_END

FN_START __nx_svc_unmap_process_code_memory
	stmfd sp!, {{r4-r6}}
    ldr r1, [sp, #0xC]
    ldr r4, [sp, #0x10]
    ldr r5, [sp, #0x14]
    ldr r6, [sp, #0x18]
    svc 0x78
    ldmfd sp!, {{r4-r6}}
    bx lr
FN_END

FN_START __nx_svc_create_process
	str r0, [sp, #-0x4]!
    svc 0x79
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_start_process
	str r4, [sp, #-0x4]!
    ldr r3, [sp, #0x4]
    ldr r3, [sp, #0x8]
    svc 0x7A
    pop {{r4}}
    bx lr
FN_END

FN_START __nx_svc_terminate_process
	svc 0x7B
	bx lr
FN_END

FN_START __nx_svc_get_process_info
	str r0, [sp, #-0x4]!
    svc 0x7C
    ldr r3, [sp]
    str r1, [r3]
    str r2, [r3, #0x4]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_create_resource_limit
	str r0, [sp, #-0x4]!
    svc 0x7D
    ldr r2, [sp]
    str r1, [r2]
    add sp, sp, #0x4
    bx lr
FN_END

FN_START __nx_svc_set_resource_limit_limit_value
	svc 0x7E
	bx lr
FN_END

FN_START __nx_svc_call_secure_monitor
	svc 0x7F
	bx lr
FN_END