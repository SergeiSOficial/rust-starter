extern crate byteorder;
extern crate chrono;
extern crate csv;
extern crate serialport;

use byteorder::{ByteOrder, LittleEndian};
use chrono::prelude::*;
use csv::Writer;
use serialport::prelude::*;
use std::fs::File;
use std::io::{self};
use std::time::Duration;

static PREAMBULA: &'static [u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
static PREAMB_SIZE: usize = 8;
static TIME_SIZE: usize = 8;
static START_SIZE: usize = TIME_SIZE + PREAMB_SIZE;

fn main() {
  let s = SerialPortSettings {
    baud_rate: 115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(500),
  };
  let mut wtr = Writer::from_path("fools.csv").unwrap();
  wtr.write_record(&["a", "b", "c"]).unwrap();
  wtr.flush().unwrap();
  let port = serialport::open_with_settings("COM5", &s);
  match port {
    Ok(mut _port) => {
      let mut writer = Writer::from_path("foobar.csv").unwrap();
      let str = "Hello, World!".to_string();
      do_write(&mut writer, str.as_bytes());
      writer.flush().unwrap();
      println!("Ready:");

      //wtr.write_record(&["Southborough", "MA", "United States", "9686"]);
      let mut serial_buf: Vec<u8> = vec![0; 10000];
      println!("Receiving data:");
      loop {
        match _port.read(serial_buf.as_mut_slice()) {
          Ok(_t) => process_data(&serial_buf), //io::stdout().write_all(&serial_buf[..t]).unwrap(),
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

fn process_data(n: &Vec<u8>) -> () {
  for i in 0..n.len() - PREAMB_SIZE {
    if n[i..i + 7] == (PREAMBULA[..PREAMBULA.len() - 1]) {
      process_time(n, i);
    } else {
      //println!("None found");
    }
  }
}

fn process_time(n: &Vec<u8>, start_i: usize) -> () {
  if n.len() > (START_SIZE) {
    let time_array = &n[start_i + PREAMB_SIZE..start_i + START_SIZE];
    let time = LittleEndian::read_i64(time_array);
    println!("Timestamp:{}", time);

    // Create a NaiveDateTime from the timestamp
    let naive = NaiveDateTime::from_timestamp(time / 1000, 0);

    // Create a normal DateTime from the NaiveDateTime
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    // Format the datetime how you want
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");

    // Print the newly formatted date and time
    println!("{}", newdate);

    let preamb = &n[start_i..start_i + 7];
    println!("Receiving data:{:?}", preamb);
  }
}

fn do_write(writer: &mut csv::Writer<File>, buf: &[u8]) {
  // The error is coming from this line
  writer.write_record(&[buf]).unwrap();
}
