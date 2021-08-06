use rusb;
use std::error::Error;
use std::time::Instant;
use tmc::list_instruments;

fn main() -> Result<(), Box<dyn Error>> {
  let context = rusb::Context::new()?;

  let timer = Instant::now();
  let instruments = list_instruments(context)?;

  if instruments.len() == 0 {
    println!("no instruments found");
  } else {
    for mut instrument in instruments {
      println!("Found instrument: {}", instrument.read_resource_string()?);

      let handle = instrument.open()?;
      println!("    USBTMC: {:?}", handle.usbtmc_capabilities);
      println!("    USB488: {:?}", handle.usb488_capabilities);
      println!("    SCPI ID: {:?}", handle.scpi_id);
      println!("    PULSE result: {:?}", handle.pulse());
    }
  }

  println!("Time elapsed: {:?}", timer.elapsed());
  Ok(())
}
