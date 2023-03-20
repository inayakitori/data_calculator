use std::sync::Arc;

use gnuplot::{AxesCommon, Caption, Color, Figure, MultiplotFillDirection, MultiplotFillOrder, PlotOption};
use office::{DataType, Excel, Range};

use crate::data_structs::{Conditions, TimeDatum};

mod data_structs;

fn main(){
    let mut workbook = Excel::open("res/LabDataNA.xlsx")
        .expect("could not find workbook");
    let range =  workbook.worksheet_range("VDAS Data")
        .expect("could not find sheet");

    let mut rows = range.rows().peekable();

    let mut avg_datums : Vec<TimeDatum> = Vec::new();

    //only need one reference to conditions
    let conditions: Arc<Conditions> = Arc::new(
        Conditions::read(
            {
                let mut rows = range.rows();
                for _ in 0..5 {
                    rows.next();
                }
                rows.next().unwrap()
            }
        )
    );

    while rows.peek().is_some() {
        let mut row = match rows.next() {
            None => { break; }
            Some(row) => { row}
        };
        let mut datums : Vec<TimeDatum> = Vec::new();
        while let DataType::Float(_) = row[0]{ //first float row
            let datum: TimeDatum = TimeDatum::read(row, conditions.clone());
            datums.push(datum);
            row = match rows.next() {
                None => { break; }
                Some(row) => { row}
            };
        }
        if !datums.is_empty() {
            avg_datums.push(TimeDatum::get_average(datums));
        }
    }

    avg_datums.iter().for_each(|datum| {
       println!("AoA: {}, lift: {}", datum.aoa, datum.lift);
    });


    let aoa: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.aoa).collect();

    let lift: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.lift_coefficient()).collect();

    let drag: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.drag_coefficient()).collect();

    let moment: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.moment_coefficient()).collect();

    //plot
    let mut fg = Figure::new();


    //C_l vs AoA
    fg.axes2d()

        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("Coefficient of Lift", &[])

        .lines(&aoa, &lift  , &[Caption("C_l"), Color("blue")])
        .lines(&[-20, 20], &[ 0  , 0  ], &[Caption(""), Color("black")])
        .lines(&[  0,  0], &[-1.0, 1.5], &[Caption(""), Color("black")]);

    fg.show().unwrap();


    //C_d vs AoA
    fg.axes2d()

        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("Coefficient of Drag", &[])

        .lines(&aoa, &drag , &[Caption("C_d"), Color("blue")])
        .lines(&[-20, 20], &[ 0  , 0   ],  &[Caption(""), Color("black")])
        .lines(&[  0,  0], &[-0.1, 0.35],  &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //C_m vs AoA
    fg.axes2d()

        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("Coefficient of Drag", &[])

        .lines(&aoa, &moment  , &[Caption("C_m"), Color("blue")])
        .lines(&[-20, 20], &[ 0  , 0  ],  &[Caption(""), Color("black")])
        .lines(&[  0,  0], &[-0.1, 0.2],  &[Caption(""), Color("black")]);

    fg.show().unwrap();

    dbg!(&avg_datums[0]);

    println!("dynamic pressure: {}", avg_datums[0].dynamic_pressure)

}


