#![no_main]
#![no_std]
#![warn(rust_2018_idioms, single_use_lifetimes, unsafe_op_in_unsafe_fn)]
#![feature(panic_info_message)]
#![allow(clippy::empty_loop)] // this test crate is #![no_std]

#[macro_use]
#[path = "../../api-test/src/helper.rs"]
mod helper;

use core::{arch::asm, sync::atomic::Ordering};

use portable_atomic::*;

macro_rules! print {
    ($($tt:tt)*) => {
        if let Some(mut uart) = $crate::semihosting::Uart::new() {
            use core::fmt::Write as _;
            let _ = write!(uart, $($tt)*);
        }
    };
}
macro_rules! println {
    ($($tt:tt)*) => {
        if let Some(mut uart) = $crate::semihosting::Uart::new() {
            use core::fmt::Write as _;
            let _ = writeln!(uart, $($tt)*);
        }
    };
}

#[no_mangle]
unsafe fn _start(_hartid: usize, fdt_address: usize) -> ! {
    unsafe {
        asm!("la sp, _stack");
        semihosting::init(fdt_address);
    }

    macro_rules! test_atomic_int {
        ($int_type:ident) => {
            paste::paste! {
                fn [<test_atomic_ $int_type>]() {
                    __test_atomic_int!([<Atomic $int_type:camel>], $int_type);
                }
                print!("test test_atomic_{} ... ", stringify!($int_type));
                [<test_atomic_ $int_type>]();
                println!("ok");
            }
        };
    }
    #[cfg(any(target_feature = "f", target_feature = "d"))]
    macro_rules! test_atomic_float {
        ($float_type:ident) => {
            paste::paste! {
                fn [<test_atomic_ $float_type>]() {
                    __test_atomic_float!([<Atomic $float_type:camel>], $float_type);
                }
                print!("test test_atomic_{} ... ", stringify!($float_type));
                [<test_atomic_ $float_type>]();
                println!("ok");
            }
        };
    }
    macro_rules! test_atomic_bool {
        () => {
            fn test_atomic_bool() {
                __test_atomic_bool!(AtomicBool);
            }
            print!("test test_atomic_bool ... ");
            test_atomic_bool();
            println!("ok");
        };
    }
    macro_rules! test_atomic_ptr {
        () => {
            fn test_atomic_ptr() {
                __test_atomic_ptr!(AtomicPtr<u8>);
            }
            print!("test test_atomic_ptr ... ");
            test_atomic_ptr();
            println!("ok");
        };
    }

    for &order in &test_helper::FENCE_ORDERINGS {
        fence(order);
        compiler_fence(order);
    }
    hint::spin_loop();
    test_atomic_bool!();
    test_atomic_ptr!();
    test_atomic_int!(isize);
    test_atomic_int!(usize);
    test_atomic_int!(i8);
    test_atomic_int!(u8);
    test_atomic_int!(i16);
    test_atomic_int!(u16);
    test_atomic_int!(i32);
    test_atomic_int!(u32);
    test_atomic_int!(i64);
    test_atomic_int!(u64);
    test_atomic_int!(i128);
    test_atomic_int!(u128);
    // TODO
    #[cfg(target_feature = "f")]
    test_atomic_float!(f32);
    #[cfg(target_feature = "d")]
    test_atomic_float!(f64);

    semihosting::exit(semihosting::EXIT_SUCCESS)
}

#[inline(never)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    if let Some(m) = info.message() {
        print!("panicked at '{m:?}'");
    } else {
        print!("panic occurred (no message)");
    }
    if let Some(l) = info.location() {
        println!(", {l}");
    } else {
        println!(" (no location info)");
    }

    semihosting::exit(semihosting::EXIT_FAILURE)
}

mod semihosting {
    // Inspired by https://github.com/SimonSapin/riscv-qemu-demo

    use core::{
        fmt,
        ptr::{self, NonNull},
        sync::atomic::Ordering,
    };

    use portable_atomic::AtomicPtr;

    #[inline(always)]
    pub unsafe fn init(fdt_address: usize) {
        unsafe {
            let fdt = &fdt::Fdt::from_ptr(fdt_address as _).unwrap();
            EXIT_HANDLE.store(find_compatible(fdt, "sifive,test0").cast(), Ordering::Release);
            UART_HANDLE.store(find_compatible(fdt, "ns16550a").cast(), Ordering::Release);
        }
    }

    #[inline(always)]
    fn find_compatible(fdt: &fdt::Fdt<'_>, with: &str) -> *mut () {
        let device = fdt.find_compatible(&[with]).unwrap();
        let register = device.reg().unwrap().next().unwrap();
        register.starting_address as _
    }

    static EXIT_HANDLE: AtomicPtr<u32> = AtomicPtr::new(ptr::null_mut());
    pub const EXIT_SUCCESS: u32 = 0;
    pub const EXIT_FAILURE: u32 = 1;
    pub fn exit(code: u32) -> ! {
        if let Some(ptr) = NonNull::new(EXIT_HANDLE.load(Ordering::Acquire)) {
            unsafe { ptr.as_ptr().write_volatile(code << 16 | 0x3333) }
        }
        loop {}
    }

    static UART_HANDLE: AtomicPtr<u8> = AtomicPtr::new(ptr::null_mut());
    pub struct Uart(NonNull<u8>);
    impl Uart {
        pub fn new() -> Option<Self> {
            let ptr = UART_HANDLE.load(Ordering::Acquire);
            NonNull::new(ptr).map(Self)
        }
    }
    impl fmt::Write for Uart {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for b in s.bytes() {
                unsafe { self.0.as_ptr().write_volatile(b) }
            }
            Ok(())
        }
    }
}
