
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
use core::fmt::{Debug, Write};

enum Direction {
    Forward,
    Backward,
    Stopped,
}

trait Invert {
    fn invert(self) -> Self;
}

trait Drive  {
    fn enable(&mut self);
    fn set_direction(&mut self,  dir: Direction);
}

impl Invert for Direction {
    fn invert(self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
            Direction::Stopped => Direction::Stopped,
        }
    }
}

impl From<u8> for Direction {
    fn from(value: u8) -> Self {
        match value {
            1 => Direction::Forward,
            2 => Direction::Backward,
            _ => Direction::Stopped,
        }
    }
}

struct Motor<PinA, PinB, PinEn, PinPwm> {
    pin_a: PinA,
    pin_b: PinB,
    pin_en: PinEn,
    pin_pwm: PinPwm,
    invert: bool,
}

impl<PinA, PinB, PinEn, PinPwm> Drive for Motor<PinA, PinB, PinEn, PinPwm>
where
    PinA: OutputPin, PinA::Error: Debug, PinB: OutputPin, PinB::Error: Debug,
    PinEn: OutputPin, PinEn::Error: Debug, PinPwm: OutputPin, PinPwm::Error: Debug,
{
    fn enable(&mut self) {
       self.pin_en.set_high().unwrap();
       self.pin_pwm.set_high().unwrap();
    }

    fn set_direction(&mut self, dir: Direction) {
        let dir = if self.invert {
            dir.invert()
        } else {
            dir
        };
        match dir {
            Direction::Forward => {
                self.pin_a.set_high().unwrap();
                self.pin_b.set_low().unwrap();
            },
            Direction::Backward =>  {
                self.pin_a.set_low().unwrap();
                self.pin_b.set_high().unwrap();
            },
            Direction::Stopped => {
                self.pin_a.set_low().unwrap();
                self.pin_b.set_low().unwrap();
            }
        }
    }
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

    let m1_in_a = gpioa.pa10.into_push_pull_output();
    let m1_in_b = gpiob.pb5.into_push_pull_output();
    let m1_en = gpiob.pb10.into_push_pull_output();
    let m1_pwm = gpioc.pc7.into_push_pull_output();

    let motor1  = Motor {
        pin_a: m1_in_a,
        pin_b: m1_in_b,
        pin_en: m1_en,
        pin_pwm: m1_pwm,
        invert: false,
    };

    let m2_in_a = gpioa.pa8.into_push_pull_output();
    let m2_in_b = gpioa.pa9.into_push_pull_output();
    let m2_en = gpioa.pa6.into_push_pull_output();
    let m2_pwm = gpiob.pb6.into_push_pull_output();

    let motor2  = Motor {
        pin_a: m2_in_a,
        pin_b: m2_in_b,
        pin_en: m2_en,
        pin_pwm: m2_pwm,
        invert: false,
    };

    let serial = dp.LPUART1.usart(gpioc.pc4,gpiob.pb11,serial::Config::default(), &mut rcc).unwrap();

    (motor1, motor2, serial, delay)
}

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();


    let (mut motor1, mut motor2, serial, mut delay) = get_io();
    let (mut tx, mut rx) = serial.split();

    tx.write_str("Hello world");

    // M1_CS = pa0
    // M2_CS = pa1

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

            //block!(tx.write(received)).ok();
            tx.write_str("OK\n");
        }
    }

    /*let mut led = gpioc.pc13.into_push_pull_output();

    // Wait for the timer to trigger an update and change the state of the LED
    loop {
        delay.delay_ms(1_000_u16);
        led.set_high().unwrap();
        delay.delay_ms(1_000_u16);
        led.set_low().unwrap();
    }*/
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("Hard fault: {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}

