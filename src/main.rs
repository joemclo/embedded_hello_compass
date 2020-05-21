#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)]
extern crate panic_halt;
#[macro_use(block)]
extern crate nb;

use core::f32::consts::PI;
use cortex_m;
use cortex_m_rt::entry;
use f3::{
    hal::{
        delay::Delay,
        i2c::I2c,
        prelude::*,
        serial::{self, Serial},
        stm32f30x,
    },
    led::{Direction, Leds},
    lsm303dlhc::I16x3,
    Lsm303dlhc,
};
use numtoa::NumToA;

#[allow(unused_imports)]
use m::Float;

type I2C = f3::hal::i2c::I2c<
    stm32f30x::I2C1,
    (
        f3::hal::gpio::gpiob::PB6<f3::hal::gpio::AF4>,
        f3::hal::gpio::gpiob::PB7<f3::hal::gpio::AF4>,
    ),
>;

fn setup() -> (Delay, serial::Tx<stm32f30x::USART1>, f3::led::Leds, I2C) {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = stm32f30x::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // setup leds
    let gpioe = dp.GPIOE.split(&mut rcc.ahb);
    let leds = Leds::new(gpioe);

    // setup i2c for magnometer
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let scl = gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let sda = gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl);
    let i2c = I2c::i2c1(dp.I2C1, (scl, sda), 400.khz(), clocks, &mut rcc.apb1);

    // setup usart interface
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let a = Serial::usart1(dp.USART1, (tx, rx), 9600.bps(), clocks, &mut rcc.apb2);
    let (tx, _) = a.split();

    // setup delay interface
    let delay = Delay::new(cp.SYST, clocks);

    (delay, tx, leds, i2c)
}

#[entry]
fn main() -> ! {
    let (mut delay, mut tx, mut leds, i2c) = setup();

    let mut lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();

    leds[Direction::North].on();
    leds[Direction::South].on();

    delay.delay_ms(1000_u16);

    // infinite loop;
    loop {
        let I16x3 { x, y, z } = lsm303dlhc.mag().unwrap();

        let theta = (y as f32).atan2(x as f32);

        let dir = if theta < -7. * PI / 8. {
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

        let mut buffer_x = [0u8; 10];
        let mut buffer_y = [0u8; 10];
        let mut buffer_z = [0u8; 10];

        x.numtoa(10, &mut buffer_x);
        y.numtoa(10, &mut buffer_y);
        z.numtoa(10, &mut buffer_z);

        let mut buffer = [0u8, 30];

        let mut index = 0;
        for byte in buffer_x.iter_mut() {
            buffer[index] = *byte;

            index += 1;
        }
        for byte in buffer_y.iter_mut() {
            buffer[index] = *byte;

            index += 1;
        }
        for byte in buffer_z.iter_mut() {
            buffer[index] = *byte;

            index += 1;
        }

        // buffer[0..19].copy_from_slice(&buffer_x);
        // buffer[20..39].copy_from_slice(&buffer_y);
        // buffer[40..59].copy_from_slice(&buffer_z);

        // write to usart, block until sent
        for byte in buffer.iter_mut() {
            block!(tx.write(*byte)).ok();
        }

        block!(tx.write(b'\t')).ok();

        // for byte in y_buffer.iter_mut() {
        //     block!(tx.write(*byte)).ok();
        // }

        // block!(tx.write(b'\t')).ok();

        // for byte in z_buffer.iter_mut() {
        //     block!(tx.write(*byte)).ok();
        // }

        block!(tx.write(b'\n')).ok();

        // delay for 100ms
        // delay.delay_ms(100_u16);
    }
}
