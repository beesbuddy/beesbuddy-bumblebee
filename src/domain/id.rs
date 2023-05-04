use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Id(Uuid);

impl Id {
    pub fn parse(s: String) -> Result<Id, String> {
        match Uuid::try_parse(&s) {
            Ok(u) => Ok(Self(u)),
            Err(_) => Err(format!("{} is not a valid uuid.", s)),
        }
    }
}

impl AsRef<Uuid> for Id {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Id;
    use claims::{assert_err, assert_ok};
    use uuid::Uuid;

    #[test]
    fn a_36_long_uuid_is_valid() {
        let id = Uuid::new_v4();
        assert_ok!(Id::parse(id.to_string()));
    }

    #[test]
    fn a_uuid_that_not_match_schema_is_rejected() {
        let uuid = "67e55044+10b1-426f-9247-bb680e5fe0c8";
        assert_err!(Id::parse(uuid.to_string()));
    }
}
