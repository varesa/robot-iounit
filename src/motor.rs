use embedded_hal::digital::v2::OutputPin;
use crate::direction::{Direction, Invert};
use core::fmt::Debug;

pub trait Drive {
    fn enable(&mut self);
    fn set_direction(&mut self,  dir: Direction);
}

pub struct Motor<PinA, PinB, PinEn, PinPwm> {
    pub pin_a: PinA,
    pub pin_b: PinB,
    pub pin_en: PinEn,
    pub pin_pwm: PinPwm,
    pub invert: bool,
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