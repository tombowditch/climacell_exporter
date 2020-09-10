use prometheus::{Encoder, Gauge, TextEncoder};
use reqwest::header::CONTENT_TYPE;
use structopt::StructOpt;
use warp::Filter;
use warp::{http::StatusCode, Rejection, Reply};

use serde::{Deserialize, Serialize};
#[macro_use]
extern crate prometheus;
#[macro_use]
extern crate lazy_static;
extern crate serde_derive;
extern crate tokio;

#[derive(Serialize, Deserialize)]
pub struct ClimacellResponse {
    temp: ClimacellVal,
    feels_like: ClimacellVal,
    wind_speed: ClimacellVal,
    humidity: ClimacellVal,
    precipitation: ClimacellVal,
}

#[derive(Serialize, Deserialize)]
pub struct ClimacellVal {
    value: f64,
}

lazy_static! {
    static ref TEMPERATURE: Gauge =
        register_gauge!("climacell_temperature", "Temperature").unwrap();
    static ref TEMPERATURE_FEELS_LIKE: Gauge =
        register_gauge!("climacell_temperature_feels_like", "Apparent Temperature").unwrap();
    static ref WIND_SPEED: Gauge = register_gauge!("climacell_wind_speed", "Wind speed").unwrap();
    static ref HUMIDITY: Gauge = register_gauge!("climacell_humidity", "Humidity").unwrap();
    static ref PRECIPITATION: Gauge =
        register_gauge!("climacell_precipitation", "Precipitation").unwrap();
    static ref SCRAPE_STATE: Gauge = register_gauge!("climacell_state", "Scrape state").unwrap();
}

#[derive(StructOpt, Debug)]
#[structopt(name = "climacell_exporter")]
struct Opt {
    #[structopt(short, long, env = "TOKEN")]
    token: String,

    #[structopt(long, env = "LAT")]
    lat: String,

    #[structopt(long, env = "LON")]
    lon: String,
}

async fn metrics_handler() -> std::result::Result<impl Reply, Rejection> {
    gather_weather_metrics().await;

    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(warp::reply::with_status(
        String::from_utf8(buffer.clone()).unwrap(),
        StatusCode::OK,
    ))
}

async fn gather_weather_metrics() {
    let opt = Opt::from_args();

    let climacell_url = format!("https://api.climacell.co/v3/weather/realtime?lat={}&lon={}&unit_system=si&fields[]=temp&fields[]=feels_like&fields[]=humidity&fields[]=wind_speed&fields[]=precipitation&apikey={}", opt.lat, opt.lon, opt.token);

    let client = reqwest::Client::new();

    let body = match client
        .get(&climacell_url)
        .header(CONTENT_TYPE, "application/json")
        .send()
        .await
    {
        Err(e) => {
            SCRAPE_STATE.set(0.0);
            println!("Error getting URL {}: {}", climacell_url, e);
            return;
        }
        Ok(body) => body,
    };

    let json: ClimacellResponse = match body.json().await {
        Err(e) => {
            SCRAPE_STATE.set(0.0);
            println!("Error converting to JSON: {}", e);
            return;
        }
        Ok(json) => json,
    };

    // worked
    SCRAPE_STATE.set(1.0);

    TEMPERATURE.set(json.temp.value);
    TEMPERATURE_FEELS_LIKE.set(json.feels_like.value);
    WIND_SPEED.set(json.wind_speed.value);
    HUMIDITY.set(json.humidity.value);
    PRECIPITATION.set(json.precipitation.value);
}

#[tokio::main]
async fn main() {
    Opt::from_args();

    println!("Listening on :9095");

    let metrics = warp::get()
        .and(warp::path("metrics"))
        .and_then(metrics_handler);

    warp::serve(metrics).run(([0, 0, 0, 0], 9095)).await
}
