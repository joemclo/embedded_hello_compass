// #![deny(unsafe_code)]
#![no_main]
#![no_std]


#[allow(unused_extern_crates)]
extern crate panic_halt;

use cortex_m;
use cortex_m_rt::entry;
use f3::{
    hal::{i2c::I2c, delay::Delay, prelude::*, stm32f30x::{self, USART1}, serial::Serial},
    led::{Leds, Direction},
    lsm303dlhc::I16x3,
    Lsm303dlhc
};
use core::f32::consts::PI;

#[allow(unused_imports)]
use m::Float;

// const XY_GAIN: f32 = 1100.;
// const Z_GAIN: f32 = 980.;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let gpioe = dp.GPIOE.split(&mut rcc.ahb);

    let mut leds = Leds::new(gpioe);

    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);
    let mut lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();


    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    Serial::usart1(dp.USART1, (tx, rx), 9600.bps(), clocks, &mut rcc.apb2);
    let usart1: &'static mut stm32f30x::usart1::RegisterBlock = unsafe {&mut *(USART1::ptr() as *mut _)};

    let mut delay = Delay::new(cp.SYST, clocks);


    leds[Direction::North].on();
    leds[Direction::South].on();

    delay.delay_ms(1000_u16);

    // infinite loop;
    loop {


        let I16x3 {x, y, z:_} = lsm303dlhc.mag().unwrap();

        // let x_mag = f32::from(x) / XY_GAIN;
        // let y_mag = f32::from(y) / XY_GAIN;
        // let z_mag = f32::from(z) / Z_GAIN;

        let theta = (y as f32).atan2(x as f32);

        let  dir = if theta < -7. * PI / 8. {
            Direction::North
        } else if theta < -5. * PI / 8. {
            Direction::Northwest
        } else if theta < -3. * PI / 8. {
            Direction::West
        } else if theta < -PI / 8. {
            Direction::Southwest
        } else if theta < PI / 8. {
            Direction::South
        } else if theta < 3. * PI / 8. {
            Direction::Southeast
        } else if theta < 5. * PI / 8. {
            Direction::East
        } else if theta < 7. * PI / 8. {
            Direction::Northeast
        } else {
            Direction::North
        };

        leds.iter_mut().for_each(|led| led.off());
        leds[dir].on();

        // write to usart

        while usart1.isr.read().txe().bit_is_clear() {}
        usart1.tdr.write(|w| w.tdr().bits(x as u16));

        for byte in b"\n\r" {
            while usart1.isr.read().txe().bit_is_clear() {}
            usart1.tdr.write(|w| w.tdr().bits(u16::from(*byte)));
        }

        delay.delay_ms(100_u16);

    }
}
