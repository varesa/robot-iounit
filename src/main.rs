
#![deny(unsafe_code)]   //  Don't allow unsafe code in this file.
//#![deny(warnings)]      //  If the Rust compiler generates a warning, stop the compilation with an error.
#![no_main]             //  Don't use the Rust standard bootstrap. We will provide our own.
#![no_std]              //  Don't use the Rust standard library. We are building a binary that can run on its own.

use cortex_m_rt::{entry, exception, ExceptionFrame};    //  Stack frame for exception handling.
use cortex_m_semihosting::hprintln;                     //  For displaying messages on the debug console.
use panic_semihosting as _;

use stm32l0xx_hal::{delay::Delay, pac, prelude::*, rcc::Config, serial};
use nb::block;
use stm32l0xx_hal::serial::Serial1LpExt;
use core::fmt::Write;

mod direction;
mod motor;

use direction::Direction;
use motor::{Motor,Drive};

macro_rules! make_motor {
    ($pin_a:expr, $pin_b:expr, $pin_enable:expr, $pin_pwm:expr, $invert:expr) => {
        Motor  {
            pin_a: $pin_a.into_push_pull_output(),
            pin_b: $pin_b.into_push_pull_output(),
            pin_en: $pin_enable.into_push_pull_output(),
            pin_pwm: $pin_pwm.into_push_pull_output(),
            invert: $invert,
        }
    };
}

fn get_io() -> (
    impl Drive,
    impl Drive,
    serial::Serial<pac::LPUART1>,
    Delay,
) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(Config::hsi16());
    let delay = cp.SYST.delay(rcc.clocks);

    let gpioa = dp.GPIOA.split(&mut rcc);
    let gpiob = dp.GPIOB.split(&mut rcc);
    let gpioc = dp.GPIOC.split(&mut rcc);

    let motor1 = make_motor!(gpioa.pa10, gpiob.pb5, gpiob.pb10, gpioc.pc7, false);
    let motor2 = make_motor!(gpioa.pa8, gpioa.pa9, gpioa.pa6, gpiob.pb6, false);

    // Unused current sense (analog) pins:
    // M1_CS = pa0
    // M2_CS = pa1

    let serial = dp.LPUART1.usart(gpioc.pc4,gpiob.pb11,serial::Config::default(), &mut rcc).unwrap();

    (motor1, motor2, serial, delay)
}

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();


    let (mut motor1, mut motor2, serial, mut delay) = get_io();
    let (mut tx, mut rx) = serial.split();

    tx.write_str("Hello world");


    motor1.enable();
    motor2.enable();

    motor1.set_direction(Direction::Forward);
    motor2.set_direction(Direction::Forward);
    delay.delay_ms(50_u16);

    motor1.set_direction(Direction::Backward);
    motor2.set_direction(Direction::Backward);
    delay.delay_ms(50_u16);

    motor1.set_direction(Direction::Stopped);
    motor2.set_direction(Direction::Stopped);


    loop {
        if let Ok(received) = block!(rx.read()) {
            // Split the received byte into two:
            let (m1, m2) = ((received & 0xF0) >> 4, (received & 0x0F));

            motor1.set_direction(m1.into());
            motor2.set_direction(m2.into());

            tx.write_str("OK\n");
        }
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

