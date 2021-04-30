use crate::config::*;
use gumdrop::Options;
use std::process::Stdio;
use std::thread::sleep;

mod config;

static CONFIG_PATH: &str = "/etc/argonfand.toml";

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

fn main() -> std::io::Result<()> {
    let opts = AppOptions::parse_args_default_or_exit();

    let mut config = if opts.generate {
        println!("Writing {}", CONFIG_PATH);
        let config = Config {
            values: vec![
                SpeedConfig { temp: Temp(45), speed: Speed(0) },
                SpeedConfig { temp: Temp(54), speed: Speed(10) },
                SpeedConfig { temp: Temp(55), speed: Speed(50) },
                SpeedConfig { temp: Temp(65), speed: Speed(80) },
                SpeedConfig { temp: Temp(80), speed: Speed(100) },
            ],
            delay: Some(1000),
            force_speed: None,
            help: false,
            verbose: false,
        };
        std::fs::write(
            CONFIG_PATH,
            toml::to_string_pretty(&config).unwrap()
        )
        .unwrap();
        return Ok(());
    } else {
        read_config()?
    };
    config.help = opts.help;
    config.verbose = opts.verbose;
    if opts.delay.is_some() {
        config.delay = opts.delay;
    }
    config.force_speed = opts.force_speed;
    eprintln!("Loaded config: {:?}", config);
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
    set_speed(&mut bus, &config);
    Ok(())
}

fn set_speed(bus: &mut rppal::i2c::I2c, config: &Config) {
    let duration = std::time::Duration::from_secs(config.delay.unwrap_or(30));
    let mut prev_block = Speed(0);

    if let Some(speed) = config.force_speed {
        if let Err(e) = bus.write(&[speed]) {
            eprintln!("  bus out {}", e);
        }
        return;
    }

    loop {
        let temp = match read_temp(config.verbose) {
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
            if let Err(e) = bus.write(&[block.into_inner()]) {
                eprintln!("  bus out {}", e);
            }
        }
        sleep(duration);
    }
}

fn read_temp(verbose: bool) -> Result<Temp, ConfigError> {
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

fn read_config() -> std::io::Result<Config> {
    let contents = std::fs::read_to_string(CONFIG_PATH)?;
    let config: Config = toml::from_str(&contents).unwrap();
    Ok(config)
}
