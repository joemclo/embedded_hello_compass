
use bluetooth_serial_port::{BtProtocol, BtSocket, BtAddr};
use std::{
    io::{Read},
    time,
};

static target_address: BtAddr = BtAddr([0x00,0x18,0xE4,0x34,0xE0,0xC8]);

fn main() {
    println!("Connecting");
    // create and connect the RFCOMM socket
    let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
    socket.connect(target_address).unwrap();
    println!("Connected");

    loop {
        let mut buffer = [0; 3];
        socket.read_exact(&mut buffer[..]).unwrap();
        println!("Reading");

        for byte in buffer.iter() {
            println!(
                "{}",
                byte
            );
        }
    }

}
