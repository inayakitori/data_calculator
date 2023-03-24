use std::ops::{AddAssign, Deref, DivAssign};
use std::sync::Arc;

use office::DataType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDatum{
    pub lift: f64,
    pub drag: f64,
    pub moment: f64,
    pub aoa: f64,
    pub dynamic_pressure: f64,
    pub wind_speed: f64,
    pub pressures: PressureReadings,
    pub wall_pressure: f64,
    pub conditions: Arc<Conditions>//only need to have one conditions so there's just a reference to them
}


impl TimeDatum {
    pub(crate) fn read(row: &[DataType], conditions: Arc<Conditions>) -> TimeDatum {
        TimeDatum{
            lift:       TimeDatum::get_val(&row[1]), //N
            drag:       TimeDatum::get_val(&row[2]), //N
            moment:     TimeDatum::get_val(&row[3]), //Nm
            aoa:        TimeDatum::get_val(&row[4]), //deg
            dynamic_pressure:   TimeDatum::get_val(&row[5]), //Pa
            wall_pressure: TimeDatum::get_val(&row[38]) * 1000.,//kPa -> Pa
            wind_speed: TimeDatum::get_val(&row[6]), //m/s
            pressures:  PressureReadings::read(row), //Pa
            /// see [Conditions]
            conditions
        }
    }

    fn get_val(data: &DataType) -> f64{
        if let DataType::Float(val) = data {
            *val
        } else {
            panic!("float unwrap error on {:?}", data);
        }
    }

    pub fn lift_coefficient(&self) -> f64{
        //C_l = l / qS
        self.lift / (self.dynamic_pressure *  self.conditions.area())
    }

    pub fn drag_coefficient(&self) -> f64{
        //C_d = d / qS
        self.drag / (self.dynamic_pressure *  self.conditions.area())
    }

    pub fn moment_coefficient(&self) -> f64{
        //C_m = m / qSc
        self.moment / (self.dynamic_pressure *  self.conditions.area() * self.conditions.chord)
    }

    pub fn pressure_coefficients(&self, side: bool) -> [(f64, f64) ; 10] {
        let mut readings = [(0f64, 0f64) ; 10];
        let side_index: usize = if side {1} else {0};
        for position_on_side in 0..10 {
            let pressure_index = 2 * position_on_side + side_index;
            //C_p = delta_p / q
            let pressure_coeff = (self.pressures[pressure_index] / self.dynamic_pressure) + 1.;
            readings[position_on_side] = (self.conditions.pressure_positions[pressure_index], pressure_coeff)
        }

        readings
    }

    pub(crate) fn lift_coefficient_via_pressures(&self) -> f64 {



        let pressure_per_side: [f64; 2] = [false, true].map(|side|{

            let (x, c_p): (Vec<f64>, Vec<f64>) = self.pressure_coefficients(side)
                .into_iter()
                .unzip();

            let pressure_coefficient = trap_int(&x, &c_p).unwrap();

            // println!("--- side: {}, calculate lift coeff from pressure coeff for this side: {}",
            //          side,
            //          pressure_coefficient
            // );

            self.pressure_coefficients(side).into_iter().for_each(|(x, c_p)| {

            });



            pressure_coefficient
        });


        let final_pressure = (pressure_per_side[0] - pressure_per_side[1]) / self.conditions.chord;

        final_pressure
    }

    //there's probably a #derive trait for this but oh well
    pub fn get_average(readings: Vec<TimeDatum>) -> TimeDatum{
        let mut final_datum = readings[0].clone();

        for datum in &readings[1..] {
            final_datum.lift       +=  datum.lift;
            final_datum.drag       +=  datum.drag;
            final_datum.moment     +=  datum.moment;
            final_datum.aoa        +=  datum.aoa;
            final_datum.dynamic_pressure +=  datum.dynamic_pressure;
            final_datum.wind_speed +=  datum.wind_speed;
            final_datum.pressures  += &datum.pressures;
        }

        let n : f64 = readings.len() as f64;

        final_datum.lift       /= n;
        final_datum.drag       /= n;
        final_datum.moment     /= n;
        final_datum.aoa        /= n;
        final_datum.dynamic_pressure /= n;
        final_datum.wind_speed /= n;
        final_datum.pressures  /= n;

        final_datum
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureReadings([f64; 20]);

impl PressureReadings {
    fn read(row: &[DataType]) -> PressureReadings {
        let mut readings = PressureReadings([0f64;20]);
        for i in 0..20 {
            readings.0[i] = TimeDatum::get_val(&row[7 + i])
                //kPa -> Pa
                * 1000.;
        }

        readings
    }
}

impl Deref for PressureReadings{
    type Target = [f64; 20];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign<&Self> for PressureReadings {
    fn add_assign(&mut self, rhs: &Self) {
        for i in 0..20 {
            self.0[i] += rhs.0[i];
        }
    }
}

impl DivAssign<f64> for PressureReadings{
    fn div_assign(&mut self, rhs: f64) {
        for i in 0..20 {
            self.0[i] /= rhs;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditions{
    pub temperature: f64,//K
    pub pressure: f64,//Pa
    pub density: f64,//Kg/m^3
    pub span: f64,//m
    pub chord: f64,//m
    pub pressure_positions: [f64; 20]
}

impl Conditions {

    pub(crate) fn area(&self) -> f64{
        return self.chord * self.span;
    }

    pub(crate) fn read(row: &[DataType], pressure_positions: [f64; 20]) -> Conditions {
        Conditions {
            temperature: TimeDatum::get_val(&row[39]) + 273.15, //C -> K
            pressure:    TimeDatum::get_val(&row[40]) * 100., //mbar -> Pa
            density:     TimeDatum::get_val(&row[41]), //kg/m^3
            span:        TimeDatum::get_val(&row[42]) / 1000., //mm -> m
            chord:       TimeDatum::get_val(&row[43]) / 1000., //mm -> m
            pressure_positions: pressure_positions.map(|pos| pos / 1000.) //mm -> m
        }
    }

}

fn trap_int(x: &[f64], y: &[f64]) -> Result<f64, String>{
    if x.len() != y.len() {
        return Err(format!(
            "lengths of x ({}) and y ({}) should be the same",
            x.len(), y.len()
        ));
    }

    Ok(
    (0..x.len() - 1)
        .into_iter()
        .map(|i| (x[i + 1] - x[i]) * (y[i + 1] + y[i]) / 2.0)
        .sum()
    )

}