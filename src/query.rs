use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

use chrono::{Duration, NaiveDateTime};
use failure;
use failure::ResultExt;
use itertools;

use input;
use Result;

const LAT_BOUNDS: [f64; 2] = [-90.0, 90.0];
const LON_BOUNDS: [f64; 2] = [-180.0, 180.0];
const WEEK_IN_SECONDS: i64 = 60 * 60 * 24 * 7;

#[derive(Debug, PartialEq)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

impl Coord {
    pub fn new(lat: f64, lon: f64) -> Result<Coord> {
        if lat < LAT_BOUNDS[0] || lat > LAT_BOUNDS[1] {
            Err(ParseError::InvalidLatitude { inv: lat })?
        };
        if lon < LON_BOUNDS[0] || lon > LON_BOUNDS[1] {
            Err(ParseError::InvalidLongitude { inv: lat })?
        };
        Ok(Coord { lat, lon })
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:.6},{:.6}", self.lat, self.lon)
    }
}

#[derive(Debug, PartialEq)]
pub struct DepartureTime(NaiveDateTime);

impl DepartureTime {
    pub fn new(timestamp: i64) -> Result<DepartureTime> {
        let timestamp = NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or_else(|| ParseError::InvalidTime { inv: timestamp })?;
        Ok(DepartureTime(timestamp))
    }

    /// If the current departure time occurs before the specified 'now'
    /// timestamp, then shift the departure time to the closest future
    /// time that falls on the same day of week and time of day.
    pub fn shift(&self, now: &NaiveDateTime) -> DepartureTime {
        let timestamp = if self.0 < *now {
            let delta = now.signed_duration_since(self.0).num_seconds();
            let remainder = delta % WEEK_IN_SECONDS;
            let offset = Duration::weeks(match remainder {
                0 => delta / WEEK_IN_SECONDS,
                _ => delta / WEEK_IN_SECONDS + 1,
            });
            self.0 + offset
        } else {
            self.0
        };

        DepartureTime(timestamp)
    }
}

impl fmt::Display for DepartureTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.timestamp())
    }
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    Bicycling,
    Driving,
    Transit,
    Walking,
}

impl FromStr for Mode {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "driving" => Ok(Mode::Driving),
            "walking" => Ok(Mode::Walking),
            "bicycling" => Ok(Mode::Bicycling),
            "transit" => Ok(Mode::Transit),
            _ => Err(ParseError::UnknownMode { unk: s.to_string() }.into()),
        }
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Mode::Bicycling => "bicycling",
            Mode::Driving => "driving",
            Mode::Transit => "transit",
            Mode::Walking => "walking",
        };
        write!(f, "{}", out)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Avoidance {
    Tolls,
    Highways,
    Ferries,
    Indoors,
}

impl FromStr for Avoidance {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "tolls" => Ok(Avoidance::Tolls),
            "highways" => Ok(Avoidance::Highways),
            "ferries" => Ok(Avoidance::Ferries),
            "indoors" => Ok(Avoidance::Indoors),
            _ => Err(ParseError::UnknownAvoidance { unk: s.to_string() }.into()),
        }
    }
}

impl fmt::Display for Avoidance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Avoidance::Tolls => "tolls",
            Avoidance::Highways => "highways",
            Avoidance::Ferries => "ferries",
            Avoidance::Indoors => "indoors",
        };
        write!(f, "{}", out)
    }
}

#[derive(Debug, PartialEq)]
pub struct Avoidances(HashSet<Avoidance>);

impl Avoidances {
    pub fn new(inp: &[Avoidance]) -> Avoidances {
        Avoidances(inp.iter().cloned().collect())
    }
}

impl FromStr for Avoidances {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Avoidances::new(&Vec::new()));
        }

        let iter = s.split('|').map(|a| a.parse::<Avoidance>());
        let avoidances = itertools::process_results(iter, |iter| iter.collect::<Vec<_>>())?;
        Ok(Avoidances::new(&avoidances))
    }
}

impl fmt::Display for Avoidances {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = itertools::join(self.0.iter(), "|");
        write!(f, "{}", out)
    }
}

#[derive(Debug, PartialEq)]
pub enum TrafficModel {
    BestGuess,
    Pessimistic,
    Optimistic,
}

impl FromStr for TrafficModel {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "best_guess" => Ok(TrafficModel::BestGuess),
            "pessimistic" => Ok(TrafficModel::Pessimistic),
            "optimistic" => Ok(TrafficModel::Optimistic),
            _ => Err(ParseError::UnknownTrafficModel { unk: s.to_string() }.into()),
        }
    }
}

impl fmt::Display for TrafficModel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            TrafficModel::BestGuess => "best_guess",
            TrafficModel::Pessimistic => "pessimistic",
            TrafficModel::Optimistic => "optimistic",
        };
        write!(f, "{}", out)
    }
}

fn to_i64(inp: &str) -> Result<i64> {
    Ok(inp.parse::<i64>().context(ParseError::InvalidInt {
        inv: inp.to_string(),
    })?)
}

fn to_f64(inp: &str) -> Result<f64> {
    Ok(inp.parse::<f64>().context(ParseError::InvalidFloat {
        inv: inp.to_string(),
    })?)
}

#[derive(Debug, PartialEq)]
pub struct Query {
    pub id: String,
    pub origin: Coord,
    pub destination: Coord,
    pub departure_time: DepartureTime,
    pub mode: Mode,
    pub avoidances: Option<Avoidances>,
    pub traffic_model: Option<TrafficModel>,
}

impl Query {
    pub fn from_csv_record(inp: input::CsvRecord, now: &NaiveDateTime) -> Result<Query> {
        let origin_lat = to_f64(&inp.origin_lat)?;
        let origin_lon = to_f64(&inp.origin_lon)?;
        let destination_lat = to_f64(&inp.destination_lat)?;
        let destination_lon = to_f64(&inp.destination_lon)?;
        let departure_time = to_i64(&inp.departure_time)?;

        let id = inp.id;
        let origin = Coord::new(origin_lat, origin_lon)?;
        let destination = Coord::new(destination_lat, destination_lon)?;
        let departure_time = DepartureTime::new(departure_time)?.shift(now);
        let mode = inp.mode.parse::<Mode>()?;
        let avoidances = match &inp.avoidances {
            Some(a) => Some(a.parse::<Avoidances>()?),
            None => None,
        };
        let traffic_model = match &inp.traffic_model {
            Some(t) => Some(t.parse::<TrafficModel>()?),
            None => None,
        };

        if mode == Mode::Driving && traffic_model.is_none() {
            Err(ParseError::MissingTrafficModel)?
        };

        Ok(Query {
            id,
            origin,
            destination,
            departure_time,
            mode,
            avoidances,
            traffic_model,
        })
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut query = String::new();

        // Required parameters
        query.push_str(&format!("origin={}", self.origin.to_string()));
        query.push_str("&");
        query.push_str(&format!("destination={}", self.destination.to_string()));
        query.push_str("&");
        query.push_str(&format!(
            "departure_time={}",
            self.departure_time.to_string()
        ));
        query.push_str("&");
        query.push_str(&format!("mode={}", self.mode.to_string()));

        // Optional parameters
        if let Some(avoidances) = &self.avoidances {
            query.push_str("&");
            query.push_str(&format!("avoid={}", avoidances.to_string()));
        };
        if let Some(traffic_model) = &self.traffic_model {
            query.push_str("&");
            query.push_str(&format!("traffic_model={}", traffic_model.to_string()));
        };

        write!(f, "{}", query)
    }
}

#[derive(Debug, Fail)]
enum ParseError {
    #[fail(display = "integer expected, found {} instead", inv)]
    InvalidInt { inv: String },
    #[fail(display = "float expected, found {} instead", inv)]
    InvalidFloat { inv: String },
    #[fail(display = "invalid latitude coordinate supplied ({})", inv)]
    InvalidLatitude { inv: f64 },
    #[fail(display = "invalid longitude coordinate supplied ({})", inv)]
    InvalidLongitude { inv: f64 },
    #[fail(display = "invalid UNIX timestamp supplied ({})", inv)]
    InvalidTime { inv: i64 },
    #[fail(display = "unrecognised mode of transport ({})", unk)]
    UnknownMode { unk: String },
    #[fail(display = "unrecognised avoidance type ({})", unk)]
    UnknownAvoidance { unk: String },
    #[fail(display = "unrecognised traffic model ({})", unk)]
    UnknownTrafficModel { unk: String },
    #[fail(display = "traffic model not supplied, this must be provided when driving is selected")]
    MissingTrafficModel,
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;
    use input::CsvRecord;

    #[test]
    fn test_parse_i64() {
        assert_eq!(1, to_i64("1").unwrap());
        assert!(to_i64("a").is_err());
    }

    #[test]
    fn test_parse_f64() {
        assert_eq!(1.0, to_f64("1.0").unwrap());
        assert!(to_f64("a").is_err());
    }

    #[test]
    fn test_construct_coord() {
        assert!(Coord::new(90.0, 180.0).is_ok());
        assert!(Coord::new(91.0, 180.0).is_err());
        assert!(Coord::new(90.0, 181.0).is_err());
        assert!(Coord::new(91.0, 181.0).is_err());
    }

    #[test]
    fn test_display_coord() {
        assert_eq!(
            "-37.820189,145.149954",
            Coord::new(-37.820189, 145.149954).unwrap().to_string(),
        )
    }

    #[test]
    fn test_construct_departure_time() {
        assert_eq!(
            NaiveDateTime::from_timestamp(1534284000, 0),
            DepartureTime::new(1534284000).unwrap().0
        );
    }

    #[test]
    fn test_display_departure_time() {
        assert_eq!(
            "1534284000",
            DepartureTime::new(1534284000).unwrap().to_string()
        )
    }

    #[test]
    fn test_update_departure_time() {
        assert_eq!(
            DepartureTime::new(1537308000).unwrap(),
            DepartureTime::new(1537308000)
                .unwrap()
                .shift(&NaiveDateTime::from_timestamp(1536991111, 0))
        );

        assert_eq!(
            DepartureTime::new(1565820000).unwrap(),
            DepartureTime::new(1565820000)
                .unwrap()
                .shift(&NaiveDateTime::from_timestamp(1536991111, 0))
        );
    }

    #[test]
    fn test_parse_mode() {
        assert_eq!(Mode::Bicycling, "bicycling".parse().unwrap());
        assert_eq!(Mode::Driving, "driving".parse().unwrap());
        assert_eq!(Mode::Transit, "transit".parse().unwrap());
        assert_eq!(Mode::Walking, "walking".parse().unwrap());
        assert!("unknown".parse::<Mode>().is_err());
    }

    #[test]
    fn test_display_mode() {
        assert_eq!("bicycling", Mode::Bicycling.to_string());
        assert_eq!("driving", Mode::Driving.to_string());
        assert_eq!("transit", Mode::Transit.to_string());
        assert_eq!("walking", Mode::Walking.to_string());
    }

    #[test]
    fn test_parse_avoidance() {
        assert_eq!(Avoidance::Tolls, "tolls".parse().unwrap());
        assert_eq!(Avoidance::Highways, "highways".parse().unwrap());
        assert_eq!(Avoidance::Ferries, "ferries".parse().unwrap());
        assert_eq!(Avoidance::Indoors, "indoors".parse().unwrap());
        assert!("unknown".parse::<Avoidance>().is_err());
    }

    #[test]
    fn test_display_avoidance() {
        assert_eq!("tolls", Avoidance::Tolls.to_string());
        assert_eq!("highways", Avoidance::Highways.to_string());
        assert_eq!("ferries", Avoidance::Ferries.to_string());
        assert_eq!("indoors", Avoidance::Indoors.to_string());
    }

    #[test]
    fn test_parse_avoidances() {
        let inp1 = "highways|ferries|indoors|tolls";
        let inp2 = "";
        let inp3 = "unknown|tolls";

        let res1 = Avoidances::new(&vec![
            Avoidance::Tolls,
            Avoidance::Highways,
            Avoidance::Ferries,
            Avoidance::Indoors,
        ]);
        let res2 = Avoidances::new(&Vec::new());

        assert_eq!(res1, inp1.parse().unwrap());
        assert_eq!(res2, inp2.parse().unwrap());
        assert!(inp3.parse::<Avoidances>().is_err());
    }

    #[test]
    fn test_display_avoidances() {
        let inp = Avoidances::new(&vec![
            Avoidance::Tolls,
            Avoidance::Highways,
            Avoidance::Ferries,
            Avoidance::Indoors,
        ]);

        assert_eq!(inp, inp.to_string().parse().unwrap());
    }

    #[test]
    fn test_parse_traffic_model() {
        assert_eq!(TrafficModel::BestGuess, "best_guess".parse().unwrap());
        assert_eq!(TrafficModel::Optimistic, "optimistic".parse().unwrap());
        assert_eq!(TrafficModel::Pessimistic, "pessimistic".parse().unwrap());
        assert!("unknown".parse::<TrafficModel>().is_err());
    }

    #[test]
    fn test_display_traffic_model() {
        assert_eq!("best_guess", TrafficModel::BestGuess.to_string());
        assert_eq!("optimistic", TrafficModel::Optimistic.to_string());
        assert_eq!("pessimistic", TrafficModel::Pessimistic.to_string());
    }

    #[test]
    fn test_csv_record_to_query() {
        let inp = CsvRecord {
            id: "1".to_string(),
            origin_lat: "-37.820189".to_string(),
            origin_lon: "145.149954".to_string(),
            destination_lat: "-37.819681".to_string(),
            destination_lon: "144.952302".to_string(),
            departure_time: "1534284000".to_string(),
            mode: "driving".to_string(),
            avoidances: Option::Some("tolls".to_string()),
            traffic_model: Option::Some("best_guess".to_string()),
        };

        let res = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1537308000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        };

        assert_eq!(
            res,
            Query::from_csv_record(inp, &NaiveDateTime::from_timestamp(1536991111, 0),).unwrap()
        );
    }

    #[test]
    fn test_missing_traffic_model() {
        let inp = CsvRecord {
            id: "1".to_string(),
            origin_lat: "-37.820189".to_string(),
            origin_lon: "145.149954".to_string(),
            destination_lat: "-37.819681".to_string(),
            destination_lon: "144.952302".to_string(),
            departure_time: "1534284000".to_string(),
            mode: "driving".to_string(),
            avoidances: Option::None,
            traffic_model: Option::None,
        };

        let res = Query::from_csv_record(inp, &NaiveDateTime::from_timestamp(1536991111, 0));
        assert!(res.is_err());
    }

    #[test]
    fn test_to_query() {
        let inp = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1534284000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        }.to_string();

        let res = "origin=-37.820189,145.149954&destination=-37.819681,144.952302&departure_time=1534284000&mode=driving&avoid=tolls&traffic_model=best_guess";

        assert_eq!(res, inp);
    }
}
