use std::fs;
use std::io;
use std::io::Read;

use chrono::NaiveDateTime;
use csv;

use failure::ResultExt;

use query::Query;
use Result;

enum Input {
    File(io::BufReader<fs::File>),
    Stdin(io::Stdin),
}

impl io::Read for Input {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Input::File(ref mut file) => file.read(buf),
            Input::Stdin(ref mut stdin) => stdin.read(buf),
        }
    }
}

pub fn load(path: &Option<String>) -> Result<String> {
    let mut rdr = match path {
        Some(p) => Input::File(io::BufReader::new(
            fs::File::open(&p).context(InputError::Path { path: p.clone() })?,
        )),
        None => Input::Stdin(io::stdin()),
    };

    let mut buffer = String::new();
    rdr.read_to_string(&mut buffer)
        .context(InputError::Data)?;
    Ok(buffer.trim().to_string())
}

#[derive(Debug, Deserialize)]
pub struct CsvRecord {
    pub id: String,
    pub origin_lat: String,
    pub origin_lon: String,
    pub destination_lat: String,
    pub destination_lon: String,
    pub departure_time: String,
    pub mode: String,
    pub avoidances: Option<String>,
    pub traffic_model: Option<String>,
}

pub fn read_csv(inp: &str, now: &NaiveDateTime) -> Result<Vec<Query>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(inp.as_bytes());

    let mut queries = Vec::new();
    let mut current_line = 1;
    for result in rdr.deserialize() {
        let record: CsvRecord = result.context(InputError::Line { ln: current_line })?;
        let query = Query::from_csv_record(record, now)
            .context(InputError::Line { ln: current_line })?;
        queries.push(query);

        current_line += 1;
    }

    Ok(queries)
}

#[derive(Debug, Fail)]
enum InputError {
    #[fail(display = "invalid input file path supplied ({})", path)]
    Path { path: String },
    #[fail(display = "input file is not valid")]
    Data,
    #[fail(display = "invalid contents on line {}", ln)]
    Line { ln: i64 },
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;
    use query::*;

    #[test]
    fn test_read_valid_csv() {
        let inp = indoc!("
            id,origin_lat,origin_lon,destination_lat,destination_lon,departure_time,mode,avoidances,traffic_model
            1,-37.820189,145.149954,-37.819681,144.952302,1534284000,driving,tolls,best_guess
        ");

        let exp = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1537308000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        };

        assert_eq!(
            exp,
            read_csv(inp, &NaiveDateTime::from_timestamp(1536991111, 0)).unwrap()[0]
        );
    }

    #[test]
    fn test_csv_missing_field() {
        let inp = indoc!("
            origin_lat,origin_lon,destination_lat,destination_lon,departure_time,mode,avoidances,traffic_model
            -37.820189,145.149954,-37.819681,144.952302,1534284000,driving,tolls,best_guess
        ");

        assert!(read_csv(inp, &NaiveDateTime::from_timestamp(1536991111, 0)).is_err());
    }

    #[test]
    fn test_csv_mispelled_field_name() {
        let inp = indoc!("
            ids,origin_lat,origin_lon,destination_lat,destination_lon,departure_time,mode,avoidances,traffic_model,unknown_field
            1,-37.820189,145.149954,-37.819681,144.952302,1534284000,driving,tolls,best_guess,unknown_value
        ");

        assert!(read_csv(inp, &NaiveDateTime::from_timestamp(1536991111, 0)).is_err());
    }
}
