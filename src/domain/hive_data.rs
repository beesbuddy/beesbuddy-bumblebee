use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Debug)]
pub struct HiveData {
    pub device_name: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub weight: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub temperature: f32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub humidity: f32,
}

impl TryFrom<Vec<u8>> for HiveData {
    type Error = anyhow::Error;

    fn try_from(encoded_value: Vec<u8>) -> Result<Self, Self::Error> {
        match String::from_utf8(encoded_value) {
            Ok(decoded_value) => match serde_json::from_str::<Self>(&decoded_value) {
                Ok(hive_data) => Ok(hive_data),
                Err(err) => anyhow::bail!("Error during raw payload deserializing = {err:?}"),
            },
            Err(err) => anyhow::bail!("Error during raw payload reading = {err:?}"),
        }
    }
}

impl HiveData {
    pub fn format_line_point(&self) -> String {
        format!(
            "hive_sensors,device_name={} temperature={},humidity={},weight={}",
            self.device_name, self.temperature, self.humidity, self.weight
        )
    }
}
