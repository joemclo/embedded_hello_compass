
use bluetooth_serial_port::{BtProtocol, BtSocket, BtAddr};
use std::{
    io::{Read},
};

static TARGET_ADDRESS: BtAddr = BtAddr([0x00,0x18,0xE4,0x34,0xE0,0xC8]);

fn main() {
    println!("Connecting");
    // create and connect the RFCOMM socket
    let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
    socket.connect(TARGET_ADDRESS).unwrap();
    println!("Connected");

    println!("Reading");

    println!("x\ty\tz");
    loop {
        let mut buffer = [0; 3];
        socket.read_exact(&mut buffer[..]).unwrap();

        let [x, y, z]= buffer;
        // Format the buffer in a naive way in order to generate a csv format to process
        // TODO write buffer to stdout correctly
        println!("{}\t{}\t{}", x,y,z)
    }

}
