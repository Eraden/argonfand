use std::str::FromStr;

#[derive(Debug)]
pub enum ConfigError {
    NoTemp,
    TempNotNumber,
    NoSpeed,
    SpeedNotNumber,
    MeasureTempOutput,
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Temp(pub u8);

impl FromStr for Temp {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace("'C", "");
        let n = if s.contains('.') {
            s.parse::<f64>()
                .map_err(|_| ConfigError::TempNotNumber)?
                .round()
                .min(101f64) as u8
        } else {
            s.parse::<u8>().map_err(|_| ConfigError::TempNotNumber)?
        };
        Ok(Self(n))
    }
}

#[derive(Debug, Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Speed(pub u8);

impl Speed {
    pub fn into_inner(&self) -> u8 {
        self.0
    }
}

impl FromStr for Speed {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.parse::<u8>().map_err(|_| ConfigError::SpeedNotNumber)?,
        ))
    }
}

#[derive(Default, Debug)]
pub struct SpeedConfig {
    temp: Temp,
    speed: Speed,
}

impl FromStr for SpeedConfig {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split("=").collect();
        Ok(SpeedConfig {
            temp: v.get(0).ok_or_else(|| ConfigError::NoTemp)?.parse()?,
            speed: v.get(1).ok_or_else(|| ConfigError::NoSpeed)?.parse()?,
        })
    }
}

#[derive(Default, Debug)]
pub struct Config {
    values: Vec<SpeedConfig>,
    pub help: bool,
    pub verbose: bool,
    pub delay: Option<u64>,
    pub force_speed: Option<u8>,
}

impl Config {
    pub fn push(&mut self, speed: SpeedConfig) {
        self.values.push(speed);
        self.values.sort_by(|a, b| a.temp.cmp(&b.temp))
    }

    // for config
    //   20'C = 30
    //   40'C = 50
    //   60'C = 80
    // for temp
    //   10'C => 0
    //   30'C => 30
    //   60'C => 80
    //   99'C => 100
    pub fn temp_speed(&self, temp: Temp) -> Speed {
        let mut speed = None;
        for s in self.values.iter().rev() {
            if s.temp >= temp {
                speed = Some(s);
            } else {
                break;
            }
        }
        eprintln!("  found {:?} for {:?}", speed, temp);
        speed.map(|c| c.speed).unwrap_or(Speed(100))
    }
}

impl FromStr for Config {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut config = Self::default();
        for line in s
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.starts_with('#'))
            .filter(|s| s.contains("="))
        {
            config.push(line.parse()?);
        }
        Ok(config)
    }
}