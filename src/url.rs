use std::str;

use base64;
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha1::Sha1;
use failure::ResultExt;
use reqwest::Url;

use config::Credentials;
use query::Query;
use Result;

const DOMAIN: &str = "https://maps.googleapis.com";
const PATH: &str = "/maps/api/directions/json";

#[derive(Debug, PartialEq)]
pub struct TaggedUrl {
    pub id: String,
    pub url: Url,
}

impl TaggedUrl {
    pub fn new(query: &Query, credentials: &Credentials) -> Result<TaggedUrl> {
        let tagged_url = match credentials {
            Credentials::Normal { api_key } => build_normal_url(query, api_key)?,
            Credentials::Premium {
                client_id,
                private_key,
                channel,
            } => build_premium_url(
                query,
                client_id,
                private_key,
                channel.as_ref().map(String::as_str),
            )?,
        };

        Ok(tagged_url)
    }
}

fn parse_url(inp: &str) -> Result<Url> {
    Ok(inp.parse::<Url>().context(UrlError::InvalidUrl {
        inv: inp.to_string(),
    })?)
}

fn build_normal_url(query: &Query, api_key: &str) -> Result<TaggedUrl> {
    let query_string = query.to_string();
    let key_parameter = format!("key={}", api_key);
    let uri_str = format!("{}{}?{}&{}", DOMAIN, PATH, query_string, key_parameter);

    let tagged_url = TaggedUrl {
        id: query.id.clone(),
        url: parse_url(&uri_str)?,
    };
    Ok(tagged_url)
}

fn build_premium_url(
    query: &Query,
    client_id: &str,
    private_key: &str,
    channel: Option<&str>,
) -> Result<TaggedUrl> {
    let query_string = query.to_string();
    let client_id_parameter = format!("client={}", client_id);
    let path_and_query = match channel {
        Some(channel) => {
            let channel_parameter = format!("channel={}", channel);
            format!(
                "{}?{}&{}&{}",
                PATH, query_string, client_id_parameter, channel_parameter
            )
        }
        None => format!("{}?{}&{}", PATH, query_string, client_id_parameter),
    };

    let signature = sign_url(&path_and_query, private_key)?;
    let signature_parameter = format!("signature={}", signature);

    let url_str = format!("{}{}&{}", DOMAIN, path_and_query, signature_parameter);

    let tagged_url = TaggedUrl {
        id: query.id.clone(),
        url: parse_url(&url_str)?,
    };
    Ok(tagged_url)
}

fn sign_url(path_and_query: &str, private_key: &str) -> Result<String> {
    let decoded_key = base64::decode_config(private_key, base64::URL_SAFE)?;
    let mut hmac = Hmac::new(Sha1::new(), &decoded_key);
    hmac.input(path_and_query.as_bytes());
    let signature = base64::encode_config(hmac.result().code(), base64::URL_SAFE);
    Ok(signature)
}

#[derive(Debug, Fail)]
enum UrlError {
    #[fail(display = "could not parse given URL string ({})", inv)]
    InvalidUrl { inv: String },
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;
    use query::*;

    #[test]
    fn test_sign_url() {
        let path_and_query = "/maps/api/geocode/json?address=New+York&client=clientID";
        let private_key = "vNIXE0xscrmjlyV-12Nj_BvUPaw=";
        assert_eq!(
            "chaRF2hTJKOScPr-RQCEhZbSzIE=",
            sign_url(path_and_query, private_key).unwrap()
        );
    }

    #[test]
    fn test_build_normal_url() {
        let query = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1537308000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        };
        let api_key = "test_key";

        let res_url = parse_url("https://maps.googleapis.com/maps/api/directions/json?origin=-37.820189,145.149954&destination=-37.819681,144.952302&departure_time=1537308000&mode=driving&avoid=tolls&traffic_model=best_guess&key=test_key").unwrap();
        let res = TaggedUrl {
            id: query.id.clone(),
            url: res_url,
        };

        assert_eq!(res, build_normal_url(&query, api_key).unwrap());
    }

    #[test]
    fn test_build_premium_url() {
        let query = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1537308000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        };
        let client_id = "clientID";
        let private_key = "vNIXE0xscrmjlyV-12Nj_BvUPaw=";

        let res_url = parse_url("https://maps.googleapis.com/maps/api/directions/json?origin=-37.820189,145.149954&destination=-37.819681,144.952302&departure_time=1537308000&mode=driving&avoid=tolls&traffic_model=best_guess&client=clientID&signature=PGyz3IR_yXL9_4MSks6uMClHDQ8=").unwrap();
        let res = TaggedUrl {
            id: query.id.clone(),
            url: res_url,
        };

        assert_eq!(
            res,
            build_premium_url(&query, client_id, private_key, Option::None).unwrap()
        );
    }

    #[test]
    fn test_build_premium_url_with_channel() {
        let query = Query {
            id: "1".to_string(),
            origin: Coord::new(-37.820189, 145.149954).unwrap(),
            destination: Coord::new(-37.819681, 144.952302).unwrap(),
            departure_time: DepartureTime::new(1537308000).unwrap(),
            mode: Mode::Driving,
            avoidances: Option::Some(Avoidances::new(&vec![Avoidance::Tolls])),
            traffic_model: Option::Some(TrafficModel::BestGuess),
        };
        let client_id = "clientID";
        let private_key = "vNIXE0xscrmjlyV-12Nj_BvUPaw=";
        let channel = Option::Some("CHANNEL");

        let res_url = parse_url("https://maps.googleapis.com/maps/api/directions/json?origin=-37.820189,145.149954&destination=-37.819681,144.952302&departure_time=1537308000&mode=driving&avoid=tolls&traffic_model=best_guess&client=clientID&channel=CHANNEL&signature=ewwxwGeX8iEt5K0NclwinyqEeLc=").unwrap();
        let res = TaggedUrl {
            id: query.id.clone(),
            url: res_url,
        };

        assert_eq!(
            res,
            build_premium_url(&query, client_id, private_key, channel).unwrap()
        );
    }
}
