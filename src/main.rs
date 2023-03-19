mod data_structs;

use office::{DataType, Excel};
use crate::data_structs::TimeDatum;

fn main(){
    let mut workbook = Excel::open("res/LabDataNA.xlsx")
        .expect("could not find workbook");
    let range =  workbook.worksheet_range("VDAS Data")
        .expect("could not find sheet");

    let mut rows = range.rows().peekable();

    let mut avg_datums : Vec<TimeDatum> = Vec::new();

    while rows.peek().is_some() {
        let mut row = match rows.next() {
            None => { break; }
            Some(row) => { row}
        };
        let mut datums : Vec<TimeDatum> = Vec::new();
        while let DataType::Float(_) = row[0]{ //first float row
            let datum: TimeDatum = TimeDatum::read(row);
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
       println!("AoA: {}", datum.aoa);
    });

    dbg!(&avg_datums[0]);

}


