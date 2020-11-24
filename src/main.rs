
#![deny(unsafe_code)]   //  Don't allow unsafe code in this file.
//#![deny(warnings)]      //  If the Rust compiler generates a warning, stop the compilation with an error.
#![no_main]             //  Don't use the Rust standard bootstrap. We will provide our own.
#![no_std]              //  Don't use the Rust standard library. We are building a binary that can run on its own.


use cortex_m_rt::{entry, exception, ExceptionFrame};    //  Stack frame for exception handling.
use cortex_m_semihosting::hprintln;                     //  For displaying messages on the debug console.
use panic_semihosting as _;

use stm32l0xx_hal::{delay::Delay, pac, prelude::*, rcc::Config};

enum Direction {
    Forward,
    Backward,
    Stopped,
}

trait Invert {
    fn invert(self) -> Self;
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

struct Motor<PinA, PinB, PinEn, PinPwm> {
    pin_a: PinA,
    pin_b: PinB,
    pin_en: PinEn,
    pin_pwm: PinPwm,
    invert: bool,
}

impl<PinA: OutputPin, PinB: OutputPin, PinEn: OutputPin, PinPwm: OutputPin> Motor<PinA, PinB, PinEn, PinPwm> 
where PinA: OutputPin, PinEn: OutputPin {
    fn enable(&mut self) {
       self.pin_en.set_high();
       self.pin_pwm.set_high();
    }

    fn set_direction(&mut self, dir: Direction) {
        let dir = if self.invert {
            dir.invert()
        } else {
            dir
        };
        match dir {
            Direction::Forward => {
                self.pin_a.set_high();
                self.pin_b.set_low();
            },
            Direction::Backward =>  {
                self.pin_a.set_low();
                self.pin_b.set_high();
            },
            Direction::Stopped => {
                self.pin_a.set_low();
                self.pin_b.set_low();
            }
        }
    }
}

#[entry]
fn main() -> ! {
    hprintln!("Hello, world!").unwrap();

    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(Config::hsi16());

    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpiob = dp.GPIOB.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);

    let mut m1_in_a = gpioa.pa10.into_push_pull_output();
    let mut m1_in_b = gpiob.pb5.into_push_pull_output();
    let mut m1_en = gpiob.pb10.into_push_pull_output();
    let mut m1_pwm = gpioc.pc7.into_push_pull_output();

    let mut motor1  = Motor {
        pin_a: m1_in_a,
        pin_b: m1_in_b,
        pin_en: m1_en,
        pin_pwm: m1_pwm,
        invert: true,
    };

    let mut m2_in_a = gpioa.pa8.into_push_pull_output();
    let mut m2_in_b = gpioa.pa9.into_push_pull_output();
    let mut m2_en = gpioa.pa6.into_push_pull_output();
    let mut m2_pwm = gpiob.pb6.into_push_pull_output();

    let mut motor2  = Motor {
        pin_a: m2_in_a,
        pin_b: m2_in_b,
        pin_en: m2_en,
        pin_pwm: m2_pwm,
        invert: false,
    };
    
    // M1_CS = pa0
    // M2_CS = pa1

    motor1.enable();
    motor2.enable();

    motor1.set_direction(Direction::Forward);
    motor2.set_direction(Direction::Forward);

    let mut delay = cp.SYST.delay(rcc.clocks);
    delay.delay_ms(2_000_u16);

    motor1.set_direction(Direction::Stopped);
    motor2.set_direction(Direction::Stopped);

    loop {}

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

