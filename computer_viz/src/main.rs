use bluetooth_serial_port::{BtAddr, BtProtocol, BtSocket};
use byteorder::{ByteOrder, LittleEndian};
use std::io::{BufRead, BufReader};

static TARGET_ADDRESS: BtAddr = BtAddr([0x00, 0x18, 0xE4, 0x34, 0xE0, 0xC8]);

fn main() {
    println!("Connecting");
    // create and connect the RFCOMM socket
    let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
    socket.connect(TARGET_ADDRESS).unwrap();
    println!("Connected");

    println!("Reading");

    println!("x\ty\tz\t\tx\ty\tz");

    for frame in BufReader::new(socket).split(0) {
        let mut frame = frame.unwrap();
        if let Ok(length) = cobs::decode_in_place(&mut frame) {
            if length == 24 {
                let mut start = 0;
                
                let mag_x = LittleEndian::read_f32(&mut frame[start..start + 4]);
                start += 4;
                let mag_y = LittleEndian::read_f32(&mut frame[start..start + 4]);
                start += 4;
                let mag_z = LittleEndian::read_f32(&mut frame[start..start + 4]);
                start += 4;
                let acc_x = LittleEndian::read_f32(&mut frame[start..start + 4]);
                start += 4;
                let acc_y = LittleEndian::read_f32(&mut frame[start..start + 4]);
                start += 4;
                let acc_z = LittleEndian::read_f32(&mut frame[start..start + 4]);

                println!("{}\t{}\t{}\t\t{}\t{}\t{}", mag_x, mag_y, mag_z, acc_x, acc_y, acc_z);
            }
        }
    }
}
