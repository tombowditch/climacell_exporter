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
extern crate tokio;

#[derive(Serialize, Deserialize, Clone)]
pub struct TomorrowWeatherApiResponse {
    data: Data,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    timelines: Vec<Timeline>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Timeline {
    timestep: String,
    #[serde(rename = "startTime")]
    start_time: String,
    #[serde(rename = "endTime")]
    end_time: String,
    intervals: Vec<Interval>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Interval {
    #[serde(rename = "startTime")]
    start_time: String,
    values: Values,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Values {
    temperature: f64,
    #[serde(rename = "temperatureApparent")]
    temperature_apparent: f64,
    humidity: f64,
    #[serde(rename = "windSpeed")]
    wind_speed: f64,
    #[serde(rename = "precipitationIntensity")]
    precipitation_intensity: f64,
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

    let climacell_url = format!("https://api.tomorrow.io/v4/timelines?location={},{}&fields=temperature&fields=temperatureApparent&fields=humidity&fields=windSpeed&fields=precipitationIntensity&apikey={}&timesteps=current", opt.lat, opt.lon, opt.token);

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

    let json: TomorrowWeatherApiResponse = match body.json().await {
        Err(e) => {
            SCRAPE_STATE.set(0.0);
            println!("Error converting to JSON: {}", e);
            return;
        }
        Ok(json) => json,
    };

    if json.data.timelines.len() < 1 {
        SCRAPE_STATE.set(0.0);
    } else {
        if json.data.timelines[0].intervals.len() < 1 {
            SCRAPE_STATE.set(0.0);
        } else {
            let weather_data = json.data.timelines[0].intervals[0].values.clone();
            SCRAPE_STATE.set(1.0);

            TEMPERATURE.set(weather_data.temperature);
            TEMPERATURE_FEELS_LIKE.set(weather_data.temperature_apparent);
            WIND_SPEED.set(weather_data.wind_speed);
            HUMIDITY.set(weather_data.humidity);
            PRECIPITATION.set(weather_data.precipitation_intensity);
        }
    }
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
