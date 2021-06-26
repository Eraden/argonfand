use crate::config::*;
use gumdrop::Options;
use static_alloc::Bump;
use std::process::Stdio;
use std::thread::sleep;
use std::process::exit;

mod config;

#[global_allocator]
static A: Bump<[u8; 1 << 14]> = Bump::uninit();

static CONFIG_PATH: &str = "/etc/argonfand.toml";

#[derive(Options)]
pub struct AppOptions {
    #[options(help = "print help message")]
    help: bool,
    #[options(help = "print more info")]
    verbose: Option<bool>,
    #[options(help = "read current temperature")]
    read: bool,
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
    if opts.read {
        match read_temp(true) {
            Ok(temp) => {
                println!("{:?}", temp);
                exit(0);
            },
            Err(e) => {
                eprintln!("{:?}", e);
                exit(1);
            },
        };
    }

    config.help = opts.help;
    if let Some(verbose) = opts.verbose {
        config.verbose = verbose;
    }
    if let Some(delay) = opts.delay {
        config.delay = Some(delay);
    }
    config.force_speed = opts.force_speed;
    eprintln!("Loaded config: {:#?}", config);
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
    let mut prev_block = Speed(255);

    if let Some(speed) = config.force_speed {
        if let Err(e) = bus.write(&[speed]) {
            eprintln!("  bus out {}", e);
        }
        return;
    }
    if config.verbose {
        println!("Starting service loop");
    }

    loop {
        let temp = match read_temp(config.verbose) {
            Ok(t) => t,
            Err(e) => {
                if config.verbose {
                    eprintln!("failed to read temperature ({:?})....", e);
                }
                sleep(duration);
                continue
            },
        };
        if config.verbose {
            eprintln!("TEMP: {:?}", temp)
        };
        let block = config.temp_speed(temp);
        if config.verbose {
            eprintln!("SPEED: {:?}", block)
        };
        if *block != *prev_block {
            prev_block = block;
            match bus.write(&[*block]) {
                Err(e) => eprintln!("  bus out {}", e),
                Ok(n) if config.verbose => {
                    println!("write new speed result {} bytes written", n);
                }
                _ => (),
            };
        }
        sleep(duration);
    }
}

fn read_temp(verbose: bool) -> Result<Temp, ConfigError> {
    let output = std::process::Command::new("vcgencmd")
        .arg("measure_temp")
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| {
            eprintln!("vcgencmd failed with {:?}", e);
            ConfigError::MeasureTempOutput
        })?
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
