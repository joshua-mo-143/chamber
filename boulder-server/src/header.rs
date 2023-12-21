use axum::headers::{Header, HeaderName, HeaderValue};

static X: HeaderName = HeaderName::from_static("x-boulder-key");
static CUSTOM_BOULDER_HEADER: &HeaderName = &X;

pub struct BoulderHeader(String);

impl BoulderHeader {
    pub fn key(self) -> String {
        self.0
    }
}

impl Header for BoulderHeader {
    fn name() -> &'static HeaderName {
        CUSTOM_BOULDER_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;

        Ok(BoulderHeader(value.to_str().unwrap().to_owned()))
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let s = &self.0;

        let value = HeaderValue::from_str(s).unwrap();

        values.extend(std::iter::once(value));
    }
}
