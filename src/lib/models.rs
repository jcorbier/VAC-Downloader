/*
 * Copyright (c) 2025 Jeremie Corbier
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the “Software”), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

/// Custom deserializer for elevation that handles both String and f64
fn deserialize_elevation<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ElevationVisitor;

    impl<'de> Visitor<'de> for ElevationVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number representing elevation")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(ElevationValueVisitor)
        }
    }

    struct ElevationValueVisitor;

    impl<'de> Visitor<'de> for ElevationValueVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or number")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as f64))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value as f64))
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            value.parse::<f64>().map(Some).map_err(|_| {
                de::Error::custom(format!("failed to parse elevation string: {}", value))
            })
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(&value)
        }
    }

    deserializer.deserialize_option(ElevationVisitor)
}

/// Response from the OACIS API (Hydra pagination format)
#[derive(Debug, Deserialize)]
pub struct OacisResponse {
    #[serde(rename = "hydra:member")]
    pub members: Vec<OacisEntry>,
    #[serde(rename = "hydra:totalItems")]
    pub total_items: i32,
}

/// Individual OACIS entry (VAC/Heliport)
#[derive(Debug, Deserialize, Clone)]
pub struct OacisEntry {
    pub code: String,
    pub city: String,
    pub grounds: Vec<Ground>,
    pub maps: Vec<Map>,
    pub runways: Vec<Runway>,
    pub frequencies: Vec<Frequency>,
    pub information: Vec<Information>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ground {
    #[serde(rename = "type")]
    pub ground_type: String,
    #[serde(deserialize_with = "deserialize_elevation")]
    pub elevation: Option<f64>,
    pub coordinates: Option<Coordinates>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Map {
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "type")]
    pub map_type: String,
    pub version: String,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Runway {
    pub length: String,
    pub width: String,
    #[serde(rename = "type")]
    pub runway_type: String,
    pub degrees: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Frequency {
    #[serde(rename = "freqAPP")]
    pub freq_app: Option<String>,
    #[serde(rename = "freqTWR")]
    pub freq_twr: Option<String>,
    #[serde(rename = "freqVDF")]
    pub freq_vdf: Option<String>,
    #[serde(rename = "freqATIS")]
    pub freq_atis: Option<String>,
    #[serde(rename = "freqFIS")]
    pub freq_fis: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Information {
    pub address: Option<String>,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
    #[serde(rename = "faxNumber")]
    pub fax_number: Option<String>,
    pub hotel: Option<String>,
    pub restaurant: Option<String>,
    pub fuel: Option<String>,
    pub repair: Option<String>,
    pub night: Option<bool>,
    #[serde(rename = "codeActivity")]
    pub code_activity: Option<String>,
    #[serde(rename = "descriptionActivity")]
    pub description_activity: Option<String>,
    pub language: Option<String>,
    pub manager: Option<String>,
    pub bank: Option<String>,
}

/// Processed VAC entry for database storage
#[derive(Debug, Clone)]
pub struct VacEntry {
    pub oaci: String,
    pub city: String,
    pub vac_type: String,
    pub version: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_hash: Option<String>,
    pub available_locally: bool,
}

impl VacEntry {
    /// Extract AD (airport) entries from OACIS data
    pub fn from_oacis_entry(entry: &OacisEntry) -> Vec<Self> {
        let mut results = Vec::new();

        for map in &entry.maps {
            // Filter only "AD" type (airports)
            if map.map_type == "AD" {
                results.push(VacEntry {
                    oaci: entry.code.clone(),
                    city: entry.city.clone(),
                    vac_type: map.map_type.clone(),
                    version: map.version.clone(),
                    file_name: map.file_name.clone(),
                    file_size: map.file_size,
                    file_hash: None,          // Hash computed after download
                    available_locally: false, // Not yet known to be local
                });
            }
        }

        results
    }
}
