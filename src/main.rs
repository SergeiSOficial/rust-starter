extern crate byteorder;
extern crate chrono;
extern crate csv;
extern crate serialport;

use byteorder::{ByteOrder, LittleEndian};
use csv::Writer;
use serialport::prelude::*;
use std::fs::File;
use std::io::{self};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

static PREAMBULA: &'static [u8] = &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
static PREAMB_SIZE: usize = 8;
static TIME_SIZE: usize = 8;
static START_SIZE: usize = TIME_SIZE + PREAMB_SIZE;
static ARRAYS_SIZE_ELEMENT: usize = 1024;
static ARRAYS_SIZE: usize = ARRAYS_SIZE_ELEMENT * 2;
static CRC_SIZE: usize = 8;

fn main() {
  let s = SerialPortSettings {
    baud_rate: 115200,
    data_bits: DataBits::Eight,
    flow_control: FlowControl::None,
    parity: Parity::None,
    stop_bits: StopBits::One,
    timeout: Duration::from_millis(500),
  };
  let port = serialport::open_with_settings("COM11", &s);
  match port {
    Ok(mut _port) => {
      let start = SystemTime::now();
      let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

      println!("{:?}", since_the_epoch.as_secs());

      let mut writer = Writer::from_path(since_the_epoch.as_secs().to_string() + ".csv").unwrap();

      println!("Ready:");
      //wtr.write_record(&["Southborough", "MA", "United States", "9686"]);
      let mut serial_buf: Vec<u8> = vec![0; 10000];
      println!("Receiving data:");
      loop {
        match _port.read(serial_buf.as_mut_slice()) {
          Ok(t) => process_data(&serial_buf, &mut writer, t), //io::stdout().write_all(&serial_buf[..t]).unwrap(),
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

fn process_data(n: &Vec<u8>, writer: &mut csv::Writer<File>, t: usize) -> () {
  for i in 0..n.len() - PREAMB_SIZE {
    if n[i..i + 7] == (PREAMBULA[..PREAMBULA.len() - 1]) {
      process_time(n, i, writer, t);
    } else {
      //println!("None found");
    }
  }
}

fn process_time(n: &Vec<u8>, start_i: usize, writer: &mut csv::Writer<File>, t: usize) -> () {
  if t >= (start_i + START_SIZE + ARRAYS_SIZE + CRC_SIZE) {
    let time_array = &n[start_i + PREAMB_SIZE..start_i + START_SIZE];
    let time = LittleEndian::read_i64(time_array);
    println!("Timestamp:{}", time);

    // // Create a NaiveDateTime from the timestamp
    // let naive = NaiveDateTime::from_timestamp(time / 1000, 0);

    // // Create a normal DateTime from the NaiveDateTime
    // let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    // // Format the datetime how you want
    // let newdate = datetime.format("%Y-%m-%d %H:%M:%S");

    // // Print the newly formatted date and time
    // println!("{}", newdate);

    let gen_state = LittleEndian::read_i32(&n[start_i + START_SIZE..start_i + START_SIZE + 4]);
    let mut buffer_array: [i32; 1025] = [0; 1025];
    buffer_array[0] = gen_state;

    for i in 0..ARRAYS_SIZE_ELEMENT {
      let first_number_array =
        &n[start_i + START_SIZE + 4 + i * 2..start_i + START_SIZE + i * 2 + 4 + 4];
      let first_number = LittleEndian::read_i16(first_number_array);
      buffer_array[i + 1] = first_number as i32;
    }
    let crc_from_array = LittleEndian::read_i64(
      &n[start_i + START_SIZE + ARRAYS_SIZE + 4..start_i + START_SIZE + ARRAYS_SIZE + 4 + CRC_SIZE],
    );

    // use provided or custom polynomial
    let mut crc: i64 = 0;
    for i in 0..ARRAYS_SIZE_ELEMENT + 1 {
      crc = crc + buffer_array[i] as i64;
    }
    if crc == crc_from_array as i64 {
      println!("crc8: {}", crc);
      println!("Receiving data:{}", buffer_array[0]);
      do_write(writer, &buffer_array);
    }
  }
}

fn do_write(writer: &mut csv::Writer<File>, buf: &[i32]) {
  // The error is coming from this .
  let mut new_string = format!("{:?}", buf);
  new_string = new_string.trim_start_matches('[').to_string();
  new_string = new_string.trim_end_matches(']').to_string();
  new_string = new_string.replace(",", ";");
  let _r = writer.serialize(&new_string);
  writer.flush().unwrap();
}
