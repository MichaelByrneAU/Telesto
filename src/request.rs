use std::mem;
use std::str;
use std::time::Duration;

use futures::{stream, Future, Stream};
use itertools::Itertools;
use ratelimit;
use reqwest::unstable::async::{Client, Decoder};
use tokio_core::reactor::Core;

use response::TaggedResponse;
use url::TaggedUrl;
use Result;

pub fn execute_requests(
    requests: &[TaggedUrl],
    rate_limit: usize,
) -> Result<Vec<TaggedResponse>> {
    // Build rate limiter
    let mut ratelimit = ratelimit::Builder::new()
        .capacity(1)
        .quantum(1)
        .interval(Duration::new(1, 0))
        .build();

    // Build event loop
    let mut core = Core::new()?;

    // Build request client
    let client = Client::new(&core.handle());

    // Retrieve iterator of IDs
    let id_iter = requests.iter().map(|r| r.id.clone());

    // Send the requests in chunks
    let mut responses: Vec<String> = Vec::with_capacity(requests.len());

    let url_iter = requests.iter().map(|r| r.url.clone());
    for chunk in &url_iter.into_iter().chunks(rate_limit) {
        let bodies = stream::iter_ok(chunk)
            .map(|url| {
                client.get(url).send().and_then(|mut res| {
                    let body = mem::replace(res.body_mut(), Decoder::empty());
                    body.concat2().from_err()
                })
            }).buffered(rate_limit);

        let work = bodies.for_each(|b| {
            responses.push(str::from_utf8(&b).unwrap().to_string());
            Ok(())
        });

        core.run(work)?;
        ratelimit.wait();
    }

    // Zip ids back to corresponding responses
    let tagged_responses = id_iter
        .zip(responses.iter())
        .map(|t| TaggedResponse::new(&t.0, t.1))
        .collect::<Vec<_>>();

    Ok(tagged_responses)
}
