use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct HiveData {
    pub apiary_id: String,
    pub hive_id: String,
    pub weight: u32,
    pub inner_temperature: u32,
    pub inner_humidity: u32,
    pub outer_temperature: u32,
    pub outer_humidity: u32,
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
        format!("hive_sensors,apiary_id={},hive_id={} inner_temperature={},inner_humidity={},outer_temperature={},outer_humidity={},weight={}", 
                self.apiary_id,
                self.hive_id,
                self.inner_temperature,
                self.inner_humidity,
                self.outer_temperature,
                self.outer_humidity,
                self.weight
        )
    }
}
