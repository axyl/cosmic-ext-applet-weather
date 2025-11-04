use reqwest::header;
use serde::Deserialize;

use crate::config::APP_ID;

#[derive(Clone, Debug, Default)]
pub struct ObservationData {
    pub wind_dir: String,
    pub wind_spd_kt: Option<i32>,
    pub gust_kt: Option<i32>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct WeatherApiResponse {
    observations: Observations,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Observations {
    data: Vec<ObservationEntry>,
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct ObservationEntry {
    wind_dir: Option<String>,
    wind_spd_kt: Option<i32>,
    gust_kt: Option<i32>,
}

pub async fn get_location_forecast(
    _latitude: String,
    _longitude: String,
) -> Result<ObservationData, reqwest::Error> {
    let request_builder = reqwest::Client::new()
        .get("https://reg.bom.gov.au/fwo/IDN60701/IDN60701.94592.json")
        .header(header::USER_AGENT, APP_ID);

    let response = request_builder.send().await?;
    let data = response.json::<WeatherApiResponse>().await?;

    let observation = data
        .observations
        .data
        .into_iter()
        .next()
        .unwrap_or_default();

    Ok(ObservationData {
        wind_dir: observation.wind_dir.unwrap_or_default(),
        wind_spd_kt: observation.wind_spd_kt,
        gust_kt: observation.gust_kt,
    })
}
