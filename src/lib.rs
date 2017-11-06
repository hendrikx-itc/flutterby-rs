#![feature(lang_items, unwind_attributes)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(abi_avr_interrupt)]
#![no_std]
#![feature(const_fn)]
#![feature(unique)]
#![feature(coerce_unsized)]
#![feature(unsize)]

#[macro_use]
pub mod macros;

extern crate arrayvec;
extern crate bare_metal;
extern crate futures;
extern crate volatile_register;

#[cfg(feature = "simavr")]
#[macro_use]
pub mod simavr;

#[macro_use]
pub mod mcu;
pub mod mutex;
pub mod fcpu;
pub mod eventloop;
pub mod timer1;
#[cfg(AVR_WDT)]
pub mod wdt;
pub mod sleep;
pub mod heap;

// The bootloader may leave some devices in a state that will cause
// a fault as soon as we re-enable interrupts.  Turn those things off
// here before we call into main().
pub fn reset_peripherals() {
    logln!("reset_peripherals");

    mutex::interrupt_free(|_cs| {
        #[cfg(AVR_USB_DEVICE)]
        unsafe {
            (*mcu::USB_DEVICE.get())
                .usbcon
                .write(mcu::UsbDeviceUsbconFlags::empty());
        }

        #[cfg(AVR_WDT)]
        wdt::initialize_disabled();
    });
}

// This lang item is present to satisfy the rust linking machinery
// that we've got an entry point.  It also provides us a way to insert
// code that runs before main.  This costs us a few bytes of instructions
// that get emitted in the bin crate, but improves ergonomics.
#[lang = "start"]
extern "C" fn __bin_crate_start(main: fn(), _argc: isize, _argv: *const *const u8) -> isize {
    reset_peripherals();
    main();
    0
}

#[lang = "eh_personality"]
#[no_mangle]
pub unsafe extern "C" fn rust_eh_personality(
    _state: (),
    _exception_object: *mut (),
    _context: *mut (),
) -> () {
}

#[lang = "panic_fmt"]
#[unwind]
pub extern "C" fn rust_begin_panic(_msg: (), _file: &'static str, _line: u32) -> ! {
    unsafe {
        asm!("BREAK" ::::"volatile");
    }
    loop {}
}
