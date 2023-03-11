#![allow(
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::too_many_lines,
    clippy::undocumented_unsafe_blocks,
    clippy::wildcard_imports
)]

#[macro_use]
pub(crate) mod helper;

#[cfg(feature = "serde")]
mod serde;

#[allow(dead_code)]
#[path = "../../version.rs"]
mod version;

use super::*;

test_atomic_bool_pub!();
test_atomic_ptr_pub!();

test_atomic_int_pub!(isize);
test_atomic_int_pub!(usize);
test_atomic_int_pub!(i8);
test_atomic_int_pub!(u8);
test_atomic_int_pub!(i16);
test_atomic_int_pub!(u16);
test_atomic_int_pub!(i32);
test_atomic_int_pub!(u32);
test_atomic_int_pub!(i64);
test_atomic_int_pub!(u64);

// As of QEMU 7.2, using lqarx/stqcx. with qemu-user hangs.
// To test this, use real powerpc64 hardware or use POWER Functional
// Simulator. See DEVELOPMENT.md for more.
#[cfg_attr(
    all(
        target_arch = "powerpc64",
        portable_atomic_unstable_asm_experimental_arch,
        any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        ),
    ),
    cfg(not(qemu))
)]
test_atomic_int_pub!(i128);
#[cfg_attr(
    all(
        target_arch = "powerpc64",
        portable_atomic_unstable_asm_experimental_arch,
        any(
            target_feature = "quadword-atomics",
            portable_atomic_target_feature = "quadword-atomics",
        ),
    ),
    cfg(not(qemu))
)]
test_atomic_int_pub!(u128);
#[cfg(qemu)]
#[cfg(all(
    target_arch = "powerpc64",
    portable_atomic_unstable_asm_experimental_arch,
    any(target_feature = "quadword-atomics", portable_atomic_target_feature = "quadword-atomics"),
))]
test_atomic_int_load_store_pub!(i128);
#[cfg(qemu)]
#[cfg(all(
    target_arch = "powerpc64",
    portable_atomic_unstable_asm_experimental_arch,
    any(target_feature = "quadword-atomics", portable_atomic_target_feature = "quadword-atomics"),
))]
test_atomic_int_load_store_pub!(u128);

#[cfg(feature = "float")]
test_atomic_float_pub!(f32);
#[cfg(feature = "float")]
test_atomic_float_pub!(f64);

#[deny(improper_ctypes)]
extern "C" {
    fn _atomic_bool_ffi_safety(_: AtomicBool);
    fn _atomic_ptr_ffi_safety(_: AtomicPtr<u8>);
    fn _atomic_isize_ffi_safety(_: AtomicIsize);
    fn _atomic_usize_ffi_safety(_: AtomicUsize);
    fn _atomic_i8_ffi_safety(_: AtomicI8);
    fn _atomic_u8_ffi_safety(_: AtomicU8);
    fn _atomic_i16_ffi_safety(_: AtomicI16);
    fn _atomic_u16_ffi_safety(_: AtomicU16);
    fn _atomic_i32_ffi_safety(_: AtomicI32);
    fn _atomic_u32_ffi_safety(_: AtomicU32);
    fn _atomic_i64_ffi_safety(_: AtomicI64);
    fn _atomic_u64_ffi_safety(_: AtomicU64);
    // TODO: 128-bit integers are not FFI safe
    // https://github.com/rust-lang/unsafe-code-guidelines/issues/119
    // https://github.com/rust-lang/rust/issues/54341
    // fn _atomic_i128_ffi_safety(_: AtomicI128);
    // fn _atomic_u128_ffi_safety(_: AtomicU128);
    #[cfg(feature = "float")]
    fn _atomic_f32_ffi_safety(_: AtomicF32);
    #[cfg(feature = "float")]
    fn _atomic_f64_ffi_safety(_: AtomicF64);
}

#[rustversion::since(1.60)] // cfg!(target_has_atomic) requires Rust 1.60
#[test]
fn test_is_lock_free() {
    assert!(AtomicI8::is_always_lock_free());
    assert!(AtomicI8::is_lock_free());
    assert!(AtomicU8::is_always_lock_free());
    assert!(AtomicU8::is_lock_free());
    assert!(AtomicI16::is_always_lock_free());
    assert!(AtomicI16::is_lock_free());
    assert!(AtomicU16::is_always_lock_free());
    assert!(AtomicU16::is_lock_free());
    assert!(AtomicI32::is_always_lock_free());
    assert!(AtomicI32::is_lock_free());
    assert!(AtomicU32::is_always_lock_free());
    assert!(AtomicU32::is_lock_free());
    if cfg!(target_has_atomic = "64") {
        assert!(AtomicI64::is_always_lock_free());
        assert!(AtomicI64::is_lock_free());
        assert!(AtomicU64::is_always_lock_free());
        assert!(AtomicU64::is_lock_free());
    } else {
        assert!(!AtomicI64::is_always_lock_free());
        assert!(!AtomicI64::is_lock_free());
        assert!(!AtomicU64::is_always_lock_free());
        assert!(!AtomicU64::is_lock_free());
    }
    if cfg!(any(
        target_arch = "aarch64",
        all(
            target_arch = "powerpc64",
            portable_atomic_unstable_asm_experimental_arch,
            any(
                target_feature = "quadword-atomics",
                portable_atomic_target_feature = "quadword-atomics",
            ),
        ),
        all(target_arch = "s390x", portable_atomic_unstable_asm_experimental_arch),
        all(
            target_arch = "x86_64",
            any(target_feature = "cmpxchg16b", portable_atomic_target_feature = "cmpxchg16b"),
        ),
    )) {
        assert!(AtomicI128::is_always_lock_free());
        assert!(AtomicI128::is_lock_free());
        assert!(AtomicU128::is_always_lock_free());
        assert!(AtomicU128::is_lock_free());
    } else {
        assert!(!AtomicI128::is_always_lock_free());
        assert!(!AtomicU128::is_always_lock_free());
        #[cfg(not(target_arch = "x86_64"))]
        {
            assert!(!AtomicI128::is_lock_free());
            assert!(!AtomicU128::is_lock_free());
        }
        #[cfg(target_arch = "x86_64")]
        // Miri doesn't support inline assembly used in is_x86_feature_detected
        #[cfg(not(miri))]
        {
            let has_cmpxchg16b = cfg!(all(
                feature = "fallback",
                portable_atomic_cmpxchg16b_target_feature,
                not(portable_atomic_no_outline_atomics),
                not(target_env = "sgx"),
            )) && std::is_x86_feature_detected!("cmpxchg16b");
            assert_eq!(AtomicI128::is_lock_free(), has_cmpxchg16b);
            assert_eq!(AtomicU128::is_lock_free(), has_cmpxchg16b);
        }
    }
}

// test version parsing code used in the build script.
#[test]
fn test_rustc_version() {
    use version::Version;

    // rustc 1.34 (rustup)
    let v = Version::parse(
        "rustc 1.34.2 (6c2484dc3 2019-05-13)
binary: rustc
commit-hash: 6c2484dc3c532c052f159264e970278d8b77cdc9
commit-date: 2019-05-13
host: x86_64-apple-darwin
release: 1.34.2
LLVM version: 8.0",
    )
    .unwrap();
    assert_eq!(v, Version::stable(34, 8));

    // rustc 1.67 (rustup)
    let v = Version::parse(
        "rustc 1.67.0 (fc594f156 2023-01-24)
binary: rustc
commit-hash: fc594f15669680fa70d255faec3ca3fb507c3405
commit-date: 2023-01-24
host: aarch64-apple-darwin
release: 1.67.0
LLVM version: 15.0.6",
    )
    .unwrap();
    assert_eq!(v, Version::stable(67, 15));

    // rustc 1.68-beta (rustup)
    let v = Version::parse(
        "rustc 1.68.0-beta.2 (10b73bf73 2023-02-01)
binary: rustc
commit-hash: 10b73bf73a6b770cd92ad8ff538173bc3298411c
commit-date: 2023-02-01
host: aarch64-apple-darwin
release: 1.68.0-beta.2
LLVM version: 15.0.6",
    )
    .unwrap();
    // We do not distinguish between stable and beta because we are only
    // interested in whether unstable features are potentially available.
    assert_eq!(v, Version::stable(68, 15));

    // rustc nightly-2019-01-27 (rustup)
    let v = Version::parse(
        "rustc 1.33.0-nightly (20c2cba61 2019-01-26)
binary: rustc
commit-hash: 20c2cba61dc83e612d25ed496025171caa3db30f
commit-date: 2019-01-26
host: x86_64-apple-darwin
release: 1.33.0-nightly
LLVM version: 8.0",
    )
    .unwrap();
    assert_eq!(v.minor, 33);
    assert!(v.nightly);
    assert_eq!(v.llvm, 8);
    assert_eq!(v.commit_date().year, 2019);
    assert_eq!(v.commit_date().month, 1);
    assert_eq!(v.commit_date().day, 26);

    // rustc 1.69-nightly (rustup)
    let v = Version::parse(
        "rustc 1.69.0-nightly (bd39bbb4b 2023-02-07)
binary: rustc
commit-hash: bd39bbb4bb92df439bf6d85470e296cc6a47ffbd
commit-date: 2023-02-07
host: aarch64-apple-darwin
release: 1.69.0-nightly
LLVM version: 15.0.7",
    )
    .unwrap();
    assert_eq!(v.minor, 69);
    assert!(v.nightly);
    assert_eq!(v.llvm, 15);
    assert_eq!(v.commit_date().year, 2023);
    assert_eq!(v.commit_date().month, 2);
    assert_eq!(v.commit_date().day, 7);

    // clippy-driver 1.69-nightly (rustup)
    let v = Version::parse(
        "rustc 1.69.0-nightly (bd39bbb4b 2023-02-07)
binary: rustc
commit-hash: bd39bbb4bb92df439bf6d85470e296cc6a47ffbd
commit-date: 2023-02-07
host: aarch64-apple-darwin
release: 1.69.0-nightly
LLVM version: 15.0.7",
    )
    .unwrap();
    assert_eq!(v.minor, 69);
    assert!(v.nightly);
    assert_eq!(v.llvm, 15);
    assert_eq!(v.commit_date().year, 2023);
    assert_eq!(v.commit_date().month, 2);
    assert_eq!(v.commit_date().day, 7);

    // rustc 1.69-dev (from source: ./x.py build)
    let v = Version::parse(
        "rustc 1.69.0-dev
binary: rustc
commit-hash: unknown
commit-date: unknown
host: aarch64-unknown-linux-gnu
release: 1.69.0-dev
LLVM version: 16.0.0",
    )
    .unwrap();
    assert_eq!(v.minor, 69);
    assert!(v.nightly);
    assert_eq!(v.llvm, 16);
    assert_eq!(v.commit_date().year, 0);
    assert_eq!(v.commit_date().month, 0);
    assert_eq!(v.commit_date().day, 0);

    // rustc 1.64 (debian: apt-get install cargo)
    let v = Version::parse(
        "rustc 1.48.0
binary: rustc
commit-hash: unknown
commit-date: unknown
host: aarch64-unknown-linux-gnu
release: 1.48.0
LLVM version: 11.0",
    )
    .unwrap();
    assert_eq!(v, Version::stable(48, 11));

    // rustc 1.67 (fedora: dnf install cargo)
    let v = Version::parse(
        "rustc 1.67.0 (fc594f156 2023-01-24) (Fedora 1.67.0-2.fc37)
binary: rustc
commit-hash: fc594f15669680fa70d255faec3ca3fb507c3405
commit-date: 2023-01-24
host: aarch64-unknown-linux-gnu
release: 1.67.0
LLVM version: 15.0.7",
    )
    .unwrap();
    assert_eq!(v, Version::stable(67, 15));

    // rustc 1.64 (alpine: apk add cargo)
    let v = Version::parse(
        "rustc 1.64.0
binary: rustc
commit-hash: unknown
commit-date: unknown
host: aarch64-alpine-linux-musl
release: 1.64.0
LLVM version: 15.0.3",
    )
    .unwrap();
    assert_eq!(v, Version::stable(64, 15));
}
