
#![deny(unsafe_code)]   //  Don't allow unsafe code in this file.
//#![deny(warnings)]      //  If the Rust compiler generates a warning, stop the compilation with an error.
#![no_main]             //  Don't use the Rust standard bootstrap. We will provide our own.
#![no_std]              //  Don't use the Rust standard library. We are building a binary that can run on its own.


use cortex_m_rt::{entry, exception, ExceptionFrame};    //  Stack frame for exception handling.
use cortex_m_semihosting::hprintln;                     //  For displaying messages on the debug console.
use panic_semihosting as _;

use embedded_hal::digital::v2::OutputPin;
use stm32l0xx_hal::{delay::Delay, pac, prelude::*, rcc::Config};

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    //let mut flash = dp.FLASH.constrain();
    //let mut rcc = dp.RCC.constrain();

    // Freeze the configuration of all the clocks in the system and store the frozen frequencies in
    // `clocks`
    //let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut rcc = dp.RCC.freeze(Config::hsi16());

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);

    let mut M1_IN_A = gpioa.pa10.into_push_pull_output();
    let mut M1_IN_B = gpiob.pb5.into_push_pull_output();
    let mut M1_EN = gpiob.pb10.into_push_pull_output();
    let mut M1_PWM = gpioc.pc7.into_push_pull_output();

    let mut M2_IN_A = gpioa.pa8.into_push_pull_output();
    let mut M2_IN_B = gpioa.pa9.into_push_pull_output();
    let mut M2_EN = gpioa.pa6.into_push_pull_output();
    let mut M2_PWM = gpiob.pb6.into_push_pull_output();
    
    // M1_CS = pa0
    // M2_CS = pa1

    M1_EN.set_high().unwrap();
    M1_PWM.set_high().unwrap();

    M2_EN.set_high().unwrap();
    M2_PWM.set_high().unwrap();

    M1_IN_A.set_high().unwrap();
    M2_IN_B.set_high().unwrap();

    let mut led = gpioc.pc13.into_push_pull_output();
    let mut delay = cp.SYST.delay(rcc.clocks);

    //loop {}

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        delay.delay_ms(1_000_u16);
        led.set_high().unwrap();
        delay.delay_ms(1_000_u16);
        led.set_low().unwrap();
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("Hard fault: {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

