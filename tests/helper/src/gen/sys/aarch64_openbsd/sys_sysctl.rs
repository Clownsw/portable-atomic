// This file is @generated by portable-atomic-internal-codegen
// (gen function at tools/codegen/src/ffi.rs).
// It is not intended for manual editing.

pub const CTL_MACHDEP: u32 = 7;
pub type u_int = ::std::os::raw::c_uint;
extern "C" {
    pub fn sysctl(
        arg1: *const ::std::os::raw::c_int,
        arg2: u_int,
        arg3: *mut ::core::ffi::c_void,
        arg4: *mut usize,
        arg5: *mut ::core::ffi::c_void,
        arg6: usize,
    ) -> ::std::os::raw::c_int;
}
