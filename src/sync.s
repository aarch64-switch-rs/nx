FN_START __nx_sync_load_exclusive
    ldaxr w0, [x1]
    ret
FN_END

FN_START __nx_sync_store_exclusive
    stlxr w0, w1, [x2]
    ret
FN_END

FN_START __nx_sync_clear_exclusive
    clrex
    ret
FN_END