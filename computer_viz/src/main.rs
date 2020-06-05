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

    println!("x\ty\tz");

    for frame in BufReader::new(socket).split(0) {
        let mut frame = frame.unwrap();
        if let Ok(length) = cobs::decode_in_place(&mut frame) {
            if length == 6 {
                let mut start = 0;
                let x = LittleEndian::read_i16(&mut frame[start..start + 2]);
                start += 2;
                let y = LittleEndian::read_i16(&mut frame[start..start + 2]);
                start += 2;
                let z = LittleEndian::read_i16(&mut frame[start..start + 2]);

                println!("{}\t{}\t{}", x, y, z);
            }
        }
    }
}
