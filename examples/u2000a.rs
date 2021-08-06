use rusb;
use std::error::Error;
use tmc::list_instruments;

// Find and connect to an attached Keysight U2000A power sensor
// and stream power readings out.  This will work on several other
// devices as well, but figuring out the right subset of IDs to accept
// is left as an exercise to the reader.
const FREQ_HZ: f32 = 2.45e9;

fn main() -> Result<(), Box<dyn Error>> {
  let context = rusb::Context::new()?;
  let instruments = list_instruments(context)?;

  if instruments.len() == 0 {
    println!("no instruments found");
    return Ok(());
  }

  let mut power_sensor = None;

  for mut instrument in instruments {
    println!("Found instrument: {}", instrument.read_resource_string()?);

    let handle = instrument.open()?;
    if let Some(id) = &handle.scpi_id {
      if id.starts_with("Keysight Technologies,U2000A") {
        println!("Found power sensor: {}", id);
        power_sensor = Some(handle);
        break;
      } else {
        println!("This is not the instrument I'm looking for: {}", id);
      }
    } else {
      println!("Sensor does not seem to support SCPI");
    }
  }

  if let Some(mut handle) = power_sensor {
    handle.set_max_transfer_size(1024);
    handle.set_term_char(Some(b'\n'))?;

    handle.write(&format!("SENS1:FREQ {}\n", FREQ_HZ as u64))?;
    loop {
      let power_str = handle.ask("FETCH1?")?;
      let power = power_str.trim().parse::<f64>()?;
      println!("{:2.3} dBm", power);
    }
  } else {
    println!("Sorry, didn't find a U2000A sensor");
    Ok(())
  }
}
