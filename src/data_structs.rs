use std::mem::align_of_val;
use std::ops::{AddAssign, Deref, DivAssign};
use office::DataType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDatum{
    pub lift: f64,
    pub drag: f64,
    pub moment: f64,
    pub aoa: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub pressures: PressureReadings,
    pub conditions: Conditions,
}

impl TimeDatum {
    pub(crate) fn read(row: &[DataType]) -> TimeDatum {
        TimeDatum{
            lift:       TimeDatum::get_val(&row[1]),
            drag:       TimeDatum::get_val(&row[2]),
            moment:     TimeDatum::get_val(&row[3]),
            aoa:        TimeDatum::get_val(&row[4]),
            pressure:   TimeDatum::get_val(&row[5]),
            wind_speed: TimeDatum::get_val(&row[6]),
            pressures:  PressureReadings::read(row),
            conditions: Conditions {
                temperature: TimeDatum::get_val(&row[39]),
                pressure:    TimeDatum::get_val(&row[40]),
                density:     TimeDatum::get_val(&row[41]),
                span:        TimeDatum::get_val(&row[42]),
                chord:       TimeDatum::get_val(&row[43]),
            },
        }
    }

    fn get_val(data: &DataType) -> f64{
        if let DataType::Float(val) = data {
            *val
        } else {
            panic!("float unwrap error on {:?}", data);
        }
    }
}

impl TimeDatum{
    pub fn get_average(readings: Vec<TimeDatum>) -> TimeDatum{
        let mut final_datum = readings[0].clone();

        for datum in &readings[1..] {
            final_datum.lift       +=  datum.lift;
            final_datum.drag       +=  datum.drag;
            final_datum.moment     +=  datum.moment;
            final_datum.aoa        +=  datum.aoa;
            final_datum.pressure   +=  datum.pressure;
            final_datum.wind_speed +=  datum.wind_speed;
            final_datum.pressures  += &datum.pressures;
        }

        let n : f64 = readings.len() as f64;

        final_datum.lift       /= n;
        final_datum.drag       /= n;
        final_datum.moment     /= n;
        final_datum.aoa        /= n;
        final_datum.pressure   /= n;
        final_datum.wind_speed /= n;
        final_datum.pressures  /= n;

        final_datum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureReadings([f64; 32]);

impl PressureReadings {
    fn read(row: &[DataType]) -> PressureReadings {
        let mut readings = PressureReadings([0f64;32]);
        for i in 0..32 {
            readings.0[i] = TimeDatum::get_val(&row[7 + i]);
        }

        readings
    }
}

impl Deref for PressureReadings{
    type Target = [f64; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign<&Self> for PressureReadings {
    fn add_assign(&mut self, rhs: &Self) {
        for i in 0..32 {
            self.0[i] += rhs.0[i];
        }
    }
}

impl DivAssign<f64> for PressureReadings{
    fn div_assign(&mut self, rhs: f64) {
        for i in 0..32 {
            self.0[i] /= rhs;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditions{
    pub temperature: f64,
    pub pressure: f64,
    pub density: f64,
    pub span: f64,
    pub chord: f64
}