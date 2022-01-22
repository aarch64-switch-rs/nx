FN_START __nx_svc_set_heap_size
	str x0, [sp, #-16]!
	svc 0x1
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_set_memory_permission
	svc 0x2
	ret
FN_END

FN_START __nx_svc_set_memory_attribute
	svc 0x3
	ret
FN_END

FN_START __nx_svc_map_memory
	svc 0x4
	ret
FN_END

FN_START __nx_svc_unmap_memory
	svc 0x5
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

FN_START __nx_svc_set_thread_priority
	svc 0xD
	ret
FN_END

FN_START __nx_svc_get_thread_core_mask
	stp x0, x1, [sp, #-16]!
	svc 0xE
	ldp x3, x4, [sp], #16
	str w1, [x3]
	str x2, [x4]
	ret
FN_END

FN_START __nx_svc_set_thread_core_mask
	svc 0xF
	ret
FN_END

FN_START __nx_svc_get_current_processor_number
	svc 0x10
	ret
FN_END

FN_START __nx_svc_signal_event
	svc 0x11
	ret
FN_END

FN_START __nx_svc_clear_event
	svc 0x12
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

FN_START __nx_svc_cancel_synchronization
	svc 0x19
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

FN_START __nx_svc_wait_process_wide_key_atomic
	svc 0x1C
	ret
FN_END

FN_START __nx_svc_signal_process_wide_key
	svc 0x1D
	ret
FN_END

FN_START __nx_svc_get_system_tick
	svc 0x1E
	ret
FN_END

FN_START __nx_svc_connect_to_named_port
	str x0, [sp, #-16]!
	svc 0x1F
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_send_sync_request_light
	svc 0x20
	ret
FN_END

FN_START __nx_svc_send_sync_request
	svc 0x21
	ret
FN_END

FN_START __nx_svc_send_sync_request_with_user_buffer
	svc 0x22
	ret
FN_END

FN_START __nx_svc_send_async_request_with_user_buffer
	str x0, [sp, #-16]!
	svc 0x23
	ldr x2, [sp], #16
	str w1, [x2]
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

FN_START __nx_svc_flush_entire_data_cache
	svc 0x2A
	ret
FN_END

FN_START __nx_svc_flush_data_cache
	svc 0x2B
	ret
FN_END

FN_START __nx_svc_map_physical_memory
	svc 0x2C
	ret
FN_END

FN_START __nx_svc_unmap_physical_memory
	svc 0x2D
	ret
FN_END

FN_START __nx_svc_get_debug_future_thread_info
	stp x0, x1, [sp, #-16]!
	svc 0x2E
	ldp x6, x7, [sp], #16
	stp x1, x2, [x6]
	stp x3, x4, [x6, #16]
	str x5, [x7]
	ret
FN_END

FN_START __nx_svc_get_last_thread_info
	stp x1, x2, [sp, #-16]!
	str x0, [sp, #-16]!
	svc 0x2F
	ldr x7, [sp], #16
	stp x1, x2, [x7]
	stp x3, x4, [x7, #16]
	ldp x1, x2, [sp], #16
	str x5, [x1]
	str w6, [x2]
	ret
FN_END

FN_START __nx_svc_get_resource_limit_limit_value
	str x0, [sp, #-16]!
	svc 0x30
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_get_resource_limit_current_value
	str x0, [sp, #-16]!
	svc 0x31
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_set_thread_activity
	svc 0x32
	ret
FN_END

FN_START __nx_svc_get_thread_context3
	svc 0x33
	ret
FN_END

FN_START __nx_svc_wait_for_address
	svc 0x34
	ret
FN_END

FN_START __nx_svc_signal_to_address
	svc 0x35
	ret
FN_END

FN_START __nx_svc_synchronize_preemption_state
	svc 0x36
	ret
FN_END

FN_START __nx_svc_get_resource_limit_peak_value
	str x0, [sp, #-16]!
	svc 0x37
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_create_io_pool
	str x0, [sp, #-16]!
	svc 0x39
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_create_io_region
	str x0, [sp, #-16]!
	svc 0x3A
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_dump_info
	svc 0x3C
	ret
FN_END

FN_START __nx_svc_kernel_debug
	svc 0x3C
	ret
FN_END

FN_START __nx_svc_change_kernel_trace_state
	svc 0x3D
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

FN_START __nx_svc_reply_and_receive_light
	svc 0x42
	ret
FN_END

FN_START __nx_svc_reply_and_receive
	str x0, [sp, #-16]!
	svc 0x43
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_reply_and_receive_with_user_buffer
	str x0, [sp, #-16]!
	svc 0x44
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

FN_START __nx_svc_map_io_region
	svc 0x48
	ret
FN_END

FN_START __nx_svc_unmap_io_region
	svc 0x49
	ret
FN_END

FN_START __nx_svc_map_physical_memory_unsafe
	svc 0x48
	ret
FN_END

FN_START __nx_svc_unmap_physical_memory_unsafe
	svc 0x49
	ret
FN_END

FN_START __nx_svc_set_unsafe_limit
	svc 0x4A
	ret
FN_END

FN_START __nx_svc_create_code_memory
	str x0, [sp, #-16]!
	svc 0x4B
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_control_code_memory
	svc 0x4C
	ret
FN_END

FN_START __nx_svc_sleep_system
	svc 0x4D
	ret
FN_END

FN_START __nx_svc_read_write_register
	str x0, [sp, #-16]!
	svc 0x4E
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_set_process_activity
	svc 0x4F
	ret
FN_END

FN_START __nx_svc_create_shared_memory
	str x0, [sp, #-16]!
	svc 0x50
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_map_transfer_memory
	svc 0x51
	ret
FN_END

FN_START __nx_svc_unmap_transfer_memory
	svc 0x52
	ret
FN_END

FN_START __nx_svc_create_interrupt_event
	str x0, [sp, #-16]!
	svc 0x53
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_query_physical_address
	str x0, [sp, #-16]!
	svc 0x54
	ldr x4, [sp], #16
	stp x1, x2, [x4]
	str x3, [x4, #16]
	ret
FN_END

FN_START __nx_svc_query_io_mapping
	stp x0, x1, [sp, #-16]!
	svc 0x55
	ldp x3, x4, [sp], #16
	str x1, [x3]
	str x2, [x4]
	ret
FN_END

FN_START __nx_svc_legacy_query_io_mapping
	str x0, [sp, #-16]!
	svc 0x55
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_create_device_address_space
	str x0, [sp, #-16]!
	svc 0x56
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_attach_device_address_space
	svc 0x57
	ret
FN_END

FN_START __nx_svc_detach_device_address_space
	svc 0x58
	ret
FN_END

FN_START __nx_svc_map_device_address_space_by_force
	svc 0x59
	ret
FN_END

FN_START __nx_svc_map_device_address_space_aligned
	svc 0x5A
	ret
FN_END

FN_START __nx_svc_map_device_address_space
	str x0, [sp, #-16]!
	svc 0x5B
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_unmap_device_address_space
	svc 0x5C
	ret
FN_END

FN_START __nx_svc_invalidate_process_data_cache
	svc 0x5D
	ret
FN_END

FN_START __nx_svc_store_process_data_cache
	svc 0x5E
	ret
FN_END

FN_START __nx_svc_flush_process_data_cache
	svc 0x5F
	ret
FN_END

FN_START __nx_svc_debug_active_process
	str x0, [sp, #-16]!
	svc 0x60
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_break_debug_process
	svc 0x61
	ret
FN_END

FN_START __nx_svc_terminate_debug_process
	svc 0x62
	ret
FN_END

FN_START __nx_svc_get_debug_event
	svc 0x63
	ret
FN_END

FN_START __nx_svc_legacy_continue_debug_event
	svc 0x64
	ret
FN_END

FN_START __nx_svc_continue_debug_event
	svc 0x64
	ret
FN_END

FN_START __nx_svc_get_process_list
	str x0, [sp, #-16]!
	svc 0x65
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_get_thread_list
	str x0, [sp, #-16]!
	svc 0x66
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_get_debug_thread_context
	svc 0x67
	ret
FN_END

FN_START __nx_svc_set_debug_thread_context
	svc 0x68
	ret
FN_END

FN_START __nx_svc_query_debug_process_memory
	str x1, [sp, #-16]!
	svc 0x69
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_read_debug_process_memory
	svc 0x6A
	ret
FN_END

FN_START __nx_svc_write_debug_process_memory
	svc 0x6B
	ret
FN_END

FN_START __nx_svc_set_hardware_break_point
	svc 0x6C
	ret
FN_END

FN_START __nx_svc_get_debug_thread_param
	stp x0, x1, [sp, #-16]!
	svc 0x6D
	ldp x3, x4, [sp], #16
	str x1, [x3]
	str w2, [x4]
	ret
FN_END

FN_START __nx_svc_get_system_info
	str x0, [sp, #-16]!
	svc 0x6F
	ldr x2, [sp], #16
	str x1, [x2]
	ret
FN_END

FN_START __nx_svc_create_port
	stp x0, x1, [sp, #-16]!
	svc 0x70
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

FN_START __nx_svc_connect_to_port
	str x0, [sp, #-16]!
	svc 0x72
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_set_process_memory_permission
	svc 0x73
	ret
FN_END

FN_START __nx_svc_map_process_memory
	svc 0x74
	ret
FN_END

FN_START __nx_svc_unmap_process_memory
	svc 0x75
	ret
FN_END

FN_START __nx_svc_query_process_memory
	str x1, [sp, #-16]!
	svc 0x76
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_map_process_code_memory
	svc 0x77
	ret
FN_END

FN_START __nx_svc_unmap_process_code_memory
	svc 0x78
	ret
FN_END

FN_START __nx_svc_create_process
	str x0, [sp, #-16]!
	svc 0x79
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_start_process
	svc 0x7A
	ret
FN_END

FN_START __nx_svc_terminate_process
	svc 0x7B
	ret
FN_END

FN_START __nx_svc_get_process_info
	str x0, [sp, #-16]!
	svc 0x7C
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_create_resource_limit
	str x0, [sp, #-16]!
	svc 0x7D
	ldr x2, [sp], #16
	str w1, [x2]
	ret
FN_END

FN_START __nx_svc_set_resource_limit_limit_value
	svc 0x7E
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
