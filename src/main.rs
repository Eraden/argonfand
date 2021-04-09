use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread::sleep;

use gumdrop::Options;
use tokio::time::Duration;

use crate::config::{Config, ConfigError, Speed, Temp};

mod config;

#[derive(Options)]
pub struct AppOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "print more info")]
    verbose: bool,
    #[options(help = "delay between updates")]
    delay: Option<u64>,
    #[options(help = "enforce speed")]
    force_speed: Option<u8>,
    #[options(help = "generate config")]
    generate: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let opts = AppOptions::parse_args_default_or_exit();
    let mut bus = match rppal::i2c::I2c::with_bus(1) {
        Ok(bus) => bus,
        Err(e) => {
            eprintln!("Failed to open I2c {}", e);
            panic!()
        }
    };
    if let Err(e) = bus.set_slave_address(0x1a) {
        panic!("Failed to set I2c address {}", e);
    };
    let bus = Arc::new(Mutex::new(bus));

    let mut config = read_config().await?;
    if opts.generate {
        println!("Writing /etc/argononed.conf");
        std::fs::write(
            "/etc/argononed.conf",
            r#"45=0
54=1
55=55
65=80
80=100
"#,
        )
        .unwrap();
        return Ok(());
    }
    config.help = opts.help;
    config.verbose = opts.verbose;
    config.delay = opts.delay;
    config.force_speed = opts.force_speed;
    eprintln!("Loaded config: {:?}", config);
    let config = Arc::new(config);
    set_speed(bus, config.clone()).await;
    Ok(())
}

async fn set_speed(bus: Arc<Mutex<rppal::i2c::I2c>>, config: Arc<Config>) {
    let duration = Duration::from_secs(config.delay.unwrap_or(30));
    let mut prev_block = Speed(0);

    if let Some(speed) = config.force_speed {
        match bus.lock() {
            Ok(mut bus) => if let Err(e) = bus.write(&[speed]) {
                eprintln!("  bus out {}", e);
            }
            Err(e) => eprintln!("{}", e),
        }
        return;
    }

    loop {
        let temp = match read_temp(config.verbose).await {
            Ok(t) => t,
            _ => continue,
        };
        if config.verbose {
            eprintln!("TEMP: {:?}", temp)
        };
        let block = config.temp_speed(temp);
        if config.verbose {
            eprintln!("SPEED: {:?}", block)
        };
        if block != prev_block {
            prev_block = block;
            match bus.lock() {
                Ok(mut bus) => if let Err(e) = bus.write(&[block.into_inner()]) {
                    eprintln!("  bus out {}", e);
                }
                Err(e) => eprintln!("{}", e),
            }
        }
        sleep(duration);
    }
}

async fn read_temp(verbose: bool) -> Result<Temp, ConfigError> {
    let output = std::process::Command::new("vcgencmd")
        .arg("measure_temp")
        .stdout(Stdio::piped())
        .output()
        .map_err(|_| ConfigError::MeasureTempOutput)?
        .stdout;
    let buffer = String::from_utf8_lossy(&output);
    let buffer = buffer.replace("temp=", "");
    if verbose {
        eprintln!("  buffer stripped {:?}", buffer);
    }
    buffer.trim().parse()
}

async fn read_config() -> std::io::Result<Config> {
    let contents = tokio::fs::read_to_string("/etc/argononed.conf").await?;
    let config: Config = contents.parse().unwrap();
    Ok(config)
}
