extern crate serialport;

use serialport::prelude::*;
use std::io::{self};
use std::time::Duration;

static PREAMBULA: &'static [u8] = &[0x01, 0x02, 0x02, 0x02];

fn main() {
  let s = SerialPortSettings {
    baud_rate: 115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(1000),
  };
  let _port = serialport::open_with_settings("COM5", &s);
  match _port {
    Ok(mut _port) => {
      println!("Ready:");
      let mut serial_buf: Vec<u8> = vec![0; 1000];
      println!("Receiving data:");
      loop {
        match _port.read(serial_buf.as_mut_slice()) {
          Ok(t) => process_data(&serial_buf[..t]), //io::stdout().write_all(&serial_buf[..t]).unwrap(),
          Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
          Err(e) => eprintln!("{:?}", e),
        }
      }
    }
    Err(_e) => {
      eprintln!("Failed to open. Error:{} ", _e);

      ::std::process::exit(1);
    }
  }
}

fn process_data(n: &[u8]) -> () {
  if n[0..7].eq(PREAMBULA) {
    let preamb = &n[..7];
    println!("Receiving data:{:?}", preamb);
  } else {
    println!("None found");
  }
}
