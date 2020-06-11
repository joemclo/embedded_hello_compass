#![deny(unsafe_code)]
#![no_main]
#![no_std]

#[allow(unused_extern_crates)]
extern crate panic_halt;
#[macro_use(block)]
extern crate nb;

use byteorder::{ByteOrder, LittleEndian};
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
    lsm303dlhc::Sensitivity,
    Lsm303dlhc,
};

#[allow(unused_imports)]
use m::Float;

const M_BIAS_X: f32 = -185.;
const M_SCALE_X: f32 = 1.092723;

const M_BIAS_Y: f32 = -163.;
const M_SCALE_Y: f32 = 0.92178;

const M_BIAS_Z: f32 = -7.5;
const M_SCALE_Z: f32 = 1.06;
const SENSITIVITY: f32 = 12. / (1 << 14) as f32;

struct F32x3 {
    x: f32,
    y: f32,
    z: f32,
}

fn setup() -> (
    Delay,
    serial::Tx<stm32f30x::USART1>,
    f3::led::Leds,
    Lsm303dlhc,
) {
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
    let lsm303dlhc = Lsm303dlhc::new(i2c).unwrap();

    // setup usart interface
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let tx = gpioa.pa9.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let rx = gpioa.pa10.into_af7(&mut gpioa.moder, &mut gpioa.afrh);
    let a = Serial::usart1(dp.USART1, (tx, rx), 9600.bps(), clocks, &mut rcc.apb2);
    let (tx, _) = a.split();

    // setup delay interface
    let delay = Delay::new(cp.SYST, clocks);

    (delay, tx, leds, lsm303dlhc)
}

fn get_compass_led_direction(angle: f32) -> Direction {
    if angle < -7. * PI / 8. {
        Direction::North
    } else if angle < -5. * PI / 8. {
        Direction::Northwest
    } else if angle < -3. * PI / 8. {
        Direction::West
    } else if angle < -PI / 8. {
        Direction::Southwest
    } else if angle < PI / 8. {
        Direction::South
    } else if angle < 3. * PI / 8. {
        Direction::Southeast
    } else if angle < 5. * PI / 8. {
        Direction::East
    } else if angle < 7. * PI / 8. {
        Direction::Northeast
    } else {
        Direction::North
    }
}

#[entry]
fn main() -> ! {
    let (mut delay, mut tx, mut leds, mut lsm303dlhc) = setup();

    leds[Direction::North].on();
    leds[Direction::South].on();

    delay.delay_ms(1000_u16);
    lsm303dlhc.set_accel_sensitivity(Sensitivity::G12).unwrap();

    // infinite loop;
    let mut tx_buf = [0; 26];
    loop {
        let mag_values = lsm303dlhc.mag().unwrap();
        let mag_values = F32x3 {
            x: (mag_values.x as f32 - M_BIAS_X) * M_SCALE_X,
            y: (mag_values.y as f32 - M_BIAS_Y) * M_SCALE_Y,
            z: (mag_values.z as f32 - M_BIAS_Z) * M_SCALE_Z,
        };

        let g = lsm303dlhc.accel().unwrap();
        let accel_values = F32x3 {
            x: g.x as f32 * SENSITIVITY,
            y: g.y as f32 * SENSITIVITY,
            z: g.z as f32 * SENSITIVITY,
        };

        let theta = (mag_values.y as f32).atan2(mag_values.x as f32);

        let dir = get_compass_led_direction(theta);

        leds.iter_mut().for_each(|led| led.off());
        leds[dir].on();

        // serialize mag readings
        let mut start = 0;
        let mut buf = [0; 24];

        LittleEndian::write_f32(&mut buf[start..start + 4], mag_values.x);
        start += 4;
        LittleEndian::write_f32(&mut buf[start..start + 4], mag_values.y);
        start += 4;
        LittleEndian::write_f32(&mut buf[start..start + 4], mag_values.z);
        start += 4;
        LittleEndian::write_f32(&mut buf[start..start + 4], accel_values.x);
        start += 4;
        LittleEndian::write_f32(&mut buf[start..start + 4], accel_values.y);
        start += 4;
        LittleEndian::write_f32(&mut buf[start..start + 4], accel_values.z);

        cobs::encode(&buf, &mut tx_buf);

        for byte in tx_buf.iter_mut() {
            // write to usart, block until sent
            block!(tx.write(*byte)).ok();
        }

        // delay for 10ms
        delay.delay_ms(10_u16);
    }
}
