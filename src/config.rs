use clap::{App, Arg};

#[derive(Debug)]
pub enum Credentials {
    Normal {
        api_key: String,
    },
    Premium {
        client_id: String,
        private_key: String,
        channel: Option<String>,
    },
}

#[derive(Debug)]
pub struct Args {
    pub input_path: Option<String>,
    pub output_path: Option<String>,
    pub credentials: Credentials,
}

fn build_cli<'a, 'b>() -> App<'a, 'b> {
    App::new("Telesto")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Batch request Google Directions API responses.")
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("Input file path")
                .value_name("PATH")
                .takes_value(true)
                .display_order(0),
        ).arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Output file path")
                .value_name("PATH")
                .takes_value(true)
                .display_order(1),
        ).arg(
            Arg::with_name("api-key")
                .short("a")
                .long("api-key")
                .help("Directions API key")
                .value_name("KEY")
                .takes_value(true)
                .conflicts_with_all(&["client-id", "private-key", "channel"])
                .required_unless_all(&["client-id", "private-key"])
                .display_order(2),
        ).arg(
            Arg::with_name("client-id")
                .short("c")
                .long("client-id")
                .help("Premium plan client ID")
                .value_name("ID")
                .requires("private-key")
                .required_unless("api-key")
                .takes_value(true)
                .display_order(3),
        ).arg(
            Arg::with_name("private-key")
                .short("p")
                .long("private-key")
                .help("Premium plan private key")
                .value_name("KEY")
                .requires("client-id")
                .required_unless("api-key")
                .takes_value(true)
                .display_order(4),
        ).arg(
            Arg::with_name("channel")
                .short("C")
                .long("channel")
                .help("Premium plan channel")
                .value_name("NAME")
                .takes_value(true)
                .requires_all(&["client-id", "private-key"])
                .display_order(5),
        )
}

pub fn run() -> Args {
    let matches = build_cli().get_matches();

    let credentials = {
        if matches.is_present("api-key") {
            Credentials::Normal {
                api_key: matches.value_of("api-key").map(|x| x.to_string()).unwrap(),
            }
        } else {
            Credentials::Premium {
                client_id: matches
                    .value_of("client-id")
                    .map(|x| x.to_string())
                    .unwrap(),
                private_key: matches
                    .value_of("private-key")
                    .map(|x| x.to_string())
                    .unwrap(),
                channel: matches.value_of("channel").map(|x| x.to_string()),
            }
        }
    };

    Args {
        input_path: matches.value_of("input").map(|x| x.to_string()),
        output_path: matches.value_of("output").map(|x| x.to_string()),
        credentials,
    }
}

#[cfg(test)]
#[cfg_attr(tarpaulin, skip)]
mod tests {
    use super::*;
    use clap::ErrorKind;

    /// You should not be able to run the program without providing credentials,
    /// either in the form of an API key or a client ID/private key pair.
    #[test]
    fn test_missing_credentials() {
        let app = build_cli();
        let res = app.get_matches_from_safe(vec!["telesto"]);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
    }

    /// You should not be able to simultaneously provide an API key and a premium
    /// plan client ID/private key pair.
    #[test]
    fn test_conflicting_credentials() {
        let app = build_cli();
        let res = app.get_matches_from_safe(vec![
            "telesto",
            "--api-key",
            "key",
            "--client-id",
            "id",
            "--private-key",
            "key",
        ]);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    }

    /// When using a premium plan client ID/private key pair, both components
    /// must be provided.
    #[test]
    fn test_premium_credentials() {
        let app = build_cli();
        let res1 = app.get_matches_from_safe(vec!["telesto", "--client-id", "id"]);
        let app = build_cli();
        let res2 = app.get_matches_from_safe(vec!["telesto", "--private-key", "key"]);
        let app = build_cli();
        let res3 =
            app.get_matches_from_safe(vec!["telesto", "--client-id", "id", "--private-key", "key"]);

        assert!(res1.is_err());
        assert_eq!(res1.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
        assert!(res2.is_err());
        assert_eq!(res2.unwrap_err().kind, ErrorKind::MissingRequiredArgument);
        assert!(res3.is_ok());
    }

    /// A channel name should only be supplied when using a premium plan client
    /// ID/private key pair.
    #[test]
    fn test_channel_conflict() {
        let app = build_cli();
        let res =
            app.get_matches_from_safe(vec!["telesto", "--api-key", "key", "--channel", "name"]);

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, ErrorKind::ArgumentConflict);
    }
}
