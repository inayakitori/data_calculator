use std::sync::Arc;

use gnuplot::{AutoOption, AxesCommon, Caption, Color, Figure, MultiplotFillDirection, MultiplotFillOrder, PlotOption};
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
            },
                //hard coded from word doc
            [
                //top   bottom
                0.76   , 1.52   ,
                3.81   , 7.62   ,
                11.43  , 15.24  ,
                19.05  , 22.86  ,
                38.00  , 41.15  ,
                62.0   , 59.44  ,
                80.77  , 77.73  ,
                101.35 , 96.02  ,
                121.92 , 114.30 ,
                137.16 , 129.54 ,
            ]
        )
    );


    //If there is a row present and it has numbers in it, take it and the next rows that have numbers in it and average their results
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

    //dbg!(&avg_datums[0]);

    //render
    render_plots(&avg_datums);

    let (aoas, lift_coeff): (Vec<f64>, Vec<f64>) =
        avg_datums.iter()
            .map(|datum| (datum.aoa , datum.lift_coefficient()))
            .filter(|(a,c)| &-12. < c && c < &12.)
            .unzip();


    //for the linear regression (which I just did in excel isnstead cause I couldn't be bothered to
    //like install GSL and do it here
    aoas.iter().for_each(|a|
        println!("{}",a)
    );

    lift_coeff.iter().for_each(|a|
        println!("{}",a)
    );

}

fn render_plots(avg_datums : &Vec<TimeDatum>){
    let mut fg = Figure::new();

    let aoa: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.aoa).collect();

    let lift: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.lift_coefficient()).collect();

    let lift_via_c_p: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.lift_coefficient_via_pressures()).collect();

    //percentage error
    let c_l_p_error: Vec<f64> = lift.iter()
        .zip(&lift_via_c_p)
        .map(|(C_l, C_l_p)| (C_l_p / C_l) - 1.)
        .map(|e| e * 100.)
        .collect();

    //absolute error
    let c_l_a_error : Vec<f64> = lift.iter()
        .zip(&lift_via_c_p)
        .map(|(C_l, C_l_p)| C_l - C_l_p)
        .collect();

    let drag: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.drag_coefficient()).collect();

    let moment: Vec<f64> = avg_datums.iter()
        .map(|datum| datum.moment_coefficient()).collect();

    //plot

    //if this breaks then you need to install gnuplot

    //C_l vs AoA
    fg = Figure::new();
    fg.axes2d()
        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("C_l", &[])

        .lines(&aoa, &lift, &[Caption("C_l via force balance"), Color("blue")])
        .lines(&aoa, &lift_via_c_p, &[Caption("C_l via C_p"), Color("red")])
        .lines(&[-20, 20], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[-1.0, 1.5], &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //C_l % error
    fg = Figure::new();
    fg.axes2d()
        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("% error", &[])

        .lines(&aoa, &c_l_p_error, &[Caption("C_l vs C_l via C_p % error"), Color("blue")])
        .lines(&[-20, 20], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[0., 120.0], &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //C_l absolute error
    fg = Figure::new();
    fg.axes2d()
        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("Absolute error", &[])

        .lines(&aoa, &c_l_a_error, &[Caption("C_l vs C_l via C_p absolute error"), Color("blue")])
        .lines(&[-20, 20], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[-0.2, 0.2], &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //C_d vs AoA
    fg = Figure::new();
    fg.axes2d()

        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("C_d", &[])

        .lines(&aoa, &drag, &[Caption("C_d"), Color("blue")])
        .lines(&[-20, 20], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[-0.1, 0.35], &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //C_m vs AoA
    fg = Figure::new();
    fg.axes2d()

        .set_x_label("Angle of Attack (deg)", &[])
        .set_y_label("C_m", &[])

        .lines(&aoa, &moment, &[Caption("C_m"), Color("blue")])
        .lines(&[-20, 20], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[-0.1, 0.2], &[Caption(""), Color("black")]);

    fg.show().unwrap();

    //pressure coeff
    fg = Figure::new();

    let mut axes = fg.axes3d();

    for datum in avg_datums {
        //split across the +ve and -ve side
        for side in [false, true] {
            let x = [datum.aoa; 10];
            let (y, z): (Vec<f64>, Vec<f64>) =
                datum
                    .pressure_coefficients(side)
                    .into_iter()
                    .unzip();
            axes
                .set_z_range(AutoOption::Fix(2.), AutoOption::Fix(-9.))
                .lines(&x, &y, &z, &[Caption(""), Color("black")]);
        }
    }

    fg.show().unwrap();

    //C_l vs C_d
    fg = Figure::new();
    fg.axes2d()

        .set_x_label("C_d", &[])
        .set_y_label("C_l", &[])

        .lines(&drag, &lift, &[Caption("C_d vs C_l"), Color("blue")])
        .lines(&[-0.1, 0.5], &[0, 0], &[Caption(""), Color("black")])
        .lines(&[0, 0], &[-1.4, 1.4], &[Caption(""), Color("black")]);

    fg.show().unwrap();

}

