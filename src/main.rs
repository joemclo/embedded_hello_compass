// #![deny(unsafe_code)]
#![no_main]
#![no_std]


#[allow(unused_extern_crates)]
extern crate panic_halt;
#[macro_use(block)]
extern crate nb;

use cortex_m;
use cortex_m_rt::entry;
use f3::{
    hal::{i2c::I2c, delay::Delay, prelude::*, stm32f30x, serial::Serial},
    led::{Leds, Direction},
    lsm303dlhc::I16x3,
    Lsm303dlhc
};
use core::f32::consts::PI;

#[allow(unused_imports)]
use m::Float;


#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // setup leds
    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let mut leds = Leds::new(gpioe);


    // setup i2c for magnometer
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);
    let mut lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();


    // setup usart interface
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let a = Serial::usart1(dp.USART1, (tx, rx), 9600.bps(), clocks, &mut rcc.apb2);
    let (mut tx, _) = a.split();

    // setup delay interface
    let mut delay = Delay::new(cp.SYST, clocks);


    leds[Direction::North].on();
    leds[Direction::South].on();

    delay.delay_ms(1000_u16);

    // infinite loop;
    loop {


        let I16x3 {x, y, z:_} = lsm303dlhc.mag().unwrap();


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

        // write to usart, block until sent
        block!(tx.write(x as u8)).ok();

        // write a new line
        for byte in b"\n\r" {
            block!(tx.write(*byte)).ok();
        }


        // delay for 100ms
        delay.delay_ms(100_u16);
    }
}
