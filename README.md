# Telesto

A simple command line application for batching requests to the [Google Directions API](https://developers.google.com/maps/documentation/directions/start "Directions API: Get Started Guide"). Requests are handled asynchronously, enabling many to be resolved concurrently in a short amount of time. 

[![Build Status](https://travis-ci.com/MichaelByrneAU/Telesto.svg?branch=master)](https://travis-ci.com/MichaelByrneAU/Telesto) [![Build Status](https://ci.appveyor.com/api/projects/status/github/MichaelByrneAU/Telesto?svg=true)](https://ci.appveyor.com/project/MichaelByrneAU/telesto)

Licensed under MIT. 

## Contents
  * [Installation](#installation)
  * [About](#about)
  * [Command line reference](#command-line-reference)
  * [Input data scheme](#input-data-schema)
  * [Output data scheme](#output-data-schema)
  * [Directions API terms of service](#directions-api-terms-of-service)
  * [What does the name mean?](#what-does-the-name-mean)

## Installation
Pre-compiled binaries of Telesto are available for Windows, macOS and Linux in the [releases section](https://github.com/MichaelByrneAU/Telesto/releases "Telesto Releases"). Users can alternatively compile Telesto from the source:

```
$ git clone https://github.com/MichaelByrneAU/Telesto
$ cd Telesto
$ cargo build --release
$ ./target/release/telesto --help
```

## About

For transport planners and modellers, it can be useful to understand what route someone will likely take when travelling between A to B at specific times of the day, as well as how long this journey will take. The Google Directions API service can serve as a useful tool in this respect, providing a constantly updating source of data to sense-check against when simulating travel behaviour. 

Given a list of origins, destinations and other parameters, this application will return the Google Directions API JSON response for each entry. Requests are handled asynchronously using [Futures](https://docs.rs/futures/0.2.1/futures/ "Futures") and the [Tokio runtime](https://tokio.rs/ "Tokio"), meaning that many can be handled concurrently at once - increasing the speed of data retrieval. This can make it feasible to facilitate thousands of requests in a short amount of time. 

You will require a Directions API key or client ID/private key pair (for premium plan users) to use this application. More information on this is available [here](https://developers.google.com/maps/documentation/directions/get-api-key "Get API Key").

## Command line reference

<a name="telesto" href="#telesto">#</a> **telesto** [options...]

Given a <a href="#input-data-schema">CSV</a> of route parameters, returns corresponding Directions API responses in a <a href="#output-data-schema">JSON list</a>. Credentials, either through an <a href="#telesto_api_key">API key</a> or <a href="#telesto_client_id">client ID</a>/<a href="#telesto_private_key">private key</a> pair must be provided. 

<a name="telesto_help" href="#telesto_help">#</a> telesto **-h**
<br><a href="#telesto_help">#</a> telesto **--help**

Output usage information.

<a name="telesto_version" href="#telesto_version">#</a> telesto **-V**
<br><a href="#telesto_version">#</a> telesto **--version**

Output version information.

<a name="telesto_input" href="#telesto_input">#</a> telesto **-i** *file*
<br><a href="#telesto_input">#</a> telesto **--input** *file*

Specify the input CSV data file path. If this option is not included, defaults to stdin. 

<a name="telesto_output" href="#telesto_output">#</a> telesto **-o** *file*
<br><a href="#telesto_output">#</a> telesto **--output** *file*

Specify the output JSON file path. If this option is not included, defaults to stout.

<a name="telesto_api_key" href="#telesto_api_key">#</a> telesto **-a** *value*
<br><a href="#telesto_api_key">#</a> telesto **--api-key** *value*

Specify the API key the application will use when making requests to the Google Directions API. Unless using a <a href="#telesto_client_id">client ID</a>/<a href="#telesto_private_key">private key</a> pair, this option is **required**. You cannot simultaneously supply an <a href="#telesto_api_key">API key</a> and a <a href="#telesto_client_id">client ID</a>/<a href="#telesto_private_key">private key</a> pair.

<a name="telesto_client_id" href="#telesto_client_id">#</a> telesto **-c** *value*
<br><a href="#telesto_client_id">#</a> telesto **--client-id** *value*

Specify the client ID the application will use when making requests to the Google Directions API. Unless using an <a href="#telesto_api_key">API key</a>, this option is **required**, and must be accompanied by the accompanying <a href="#telesto_private_key">private key</a>. You cannot simultaneously supply a <a href="#telesto_client_id">client ID</a>/<a href="#telesto_private_key">private key</a> pair and an <a href="#telesto_api_key">API key</a>. 

<a name="telesto_channel" href="#telesto_channel">#</a> telesto **-C** *value*
<br><a href="#telesto_channel">#</a> telesto **--channel** *value*

Specify the channel name the application will use when making requests to the Google Directions API. This is optional but can only be used when accompanied by a <a href="#telesto_client_id">client ID</a>/<a href="#telesto_private_key">private key</a> pair. 

## Input data schema

Telesto takes a CSV file as an input with nine fields. Here is an example:

```
  id   origin_lat   origin_lon   destination_lat   destination_lon   departure_time    mode       avoidances     traffic_model  
 ---- ------------ ------------ ----------------- ----------------- ---------------- --------- ---------------- --------------- 
   1   -37.820189   145.149954        -37.819681        144.952302       1534284000   driving   tolls|highways   best_guess     
   2   -37.820189   145.149954        -37.819681        144.952302       1534284000   transit                                   
```

<a name="telesto_field_id" href="#telesto_field_id">#</a> field **id** *string*

A string that acts as an identifier for that request. This string must be unique across all requests in the CSV input.

<a name="telesto_field_origin_lat" href="#telesto_field_origin_lat">#</a> field **origin_lat** *float*

The latitude coordinate of the request's origin. 

<a name="telesto_field_origin_lon" href="#telesto_field_origin_lon">#</a> field **origin_lon** *float*

The longitude coordinate of the request's origin.

<a name="telesto_field_destination_lat" href="#telesto_field_destination_lat">#</a> field **destination_lat** *float*

The latitude coordinate of the request's destination. 

<a name="telesto_field_destination_lon" href="#telesto_field_destination_lon">#</a> field **destination_lon** *float*

The longitude coordinate of the request's destination.

<a name="telesto_field_departure_time" href="#telesto_field_departure_time">#</a> field **departure_time** *integer*

The departure time of the request in [unix epoch time](https://www.epochconverter.com/ "Unix Timestamp Converter"). Note that the Directions API only accepts *future* times for requests. If a departure time in the past is specified, then this time will be replaced with the closest future time that falls on the same day of the week and time of day. If you wish to have finer control over the desired departure time for a request, specify a future time relative to when you run the application. 

<a name="telesto_field_mode" href="#telesto_field_mode">#</a> field **mode** *string*

The mode of transport for the request. This can be one of the following:

* bicycling
* driving
* transit
* walking

<a name="telesto_field_avoidances" href="#telesto_field_avoidances">#</a> field **avoidances** *string*

Any avoidances the Directions API should take into account when providing a route for a request. This can include the following conditions:

* ferries
* highways
* indoors
* tolls

Multiple conditions can be specified simultaneously by separating them with a pipe, e.g. 'tolls|highways'. It is **optional** to include values in this field.

<a name="telesto_field_traffic_model" href="#telesto_field_traffic_model">#</a> field **traffic_model** *string*

The traffic model the Directions API should use when providing travel time estimates for driving trips. This can be one of the following:

* best_guess
* pessimistic
* optimistic

This field is **required** when the specified mode is 'driving', otherwise it should be left blank.

## Output data schema

Telesto will provide as output a JSON list with the following structure:

```
[
    {
        id: your_request_id,
        response: {
            ... JSON response from the Directions API ...
        }
    },
    
    ...

]
```

Essentially, a list of <a href="#telesto_field_id">request ID</a> and response pairs. The format of the Directions API JSON response is described in more detail [here](https://developers.google.com/maps/documentation/directions/intro#DirectionsResponses "Directions Responses"). 

## Directions API terms of service

The data you collect through Telesto using your API key or client ID/private key pair is subject to the particular [terms of service](https://developers.google.com/maps/documentation/directions/policies "Directions API Policies") that you agreed to with the Google Maps Platform when supplied with those credentials. This includes, but is not limited to, the storage, reporting, visualisation and attribution of this data in accordance with your particular agreement with Google.

## What does the name mean?

Nothing, I like naming my projects after moons in the solar system. [Telesto](https://en.wikipedia.org/wiki/Telesto_(moon) "Telesto") is a potato-shaped moon of Saturn that was discovered in 1980. 
