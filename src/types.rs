pub type Fd = u32;
pub type Pid = u32;

pub type Count = u64;
pub type Bytes = Count;
pub type Rpm = Count;

pub type Percent = f32;

pub type FloatCount = f64;
pub type Degrees = FloatCount;
pub type Mhz = FloatCount;

#[derive(Debug)]
pub struct Temperature {
    celcius: Degrees,
}

impl Temperature {
    pub fn new(celcius: Degrees) -> Temperature {
        Temperature { celcius }
    }

    pub fn celcius(&self) -> Degrees {
        self.celcius
    }

    #[allow(clippy::unnecessary_cast)]
    pub fn fahrenheit(&self) -> Degrees {
        (self.celcius * (9 as Degrees / 5 as Degrees)) + 32 as Degrees
    }
}
