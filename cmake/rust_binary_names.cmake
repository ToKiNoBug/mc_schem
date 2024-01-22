function(rust_binary_names out_var_shared_lib out_var_export_lib)
    unset(${out_var_shared_lib} PARENT_SCOPE)
    unset(${out_var_export_lib} PARENT_SCOPE)
    if (MSVC)
        set(${out_var_shared_lib} "mc_schem.dll" PARENT_SCOPE)
        set(${out_var_export_lib} "mc_schem.dll.lib" PARENT_SCOPE)
        return()
    endif ()

    if (LINUX)
        set(${out_var_shared_lib} "libmc_schem.so" PARENT_SCOPE)
        return()
    endif ()


    message(FATAL_ERROR "Can not guess the names of generated binaries.")
endfunction(rust_binary_names)