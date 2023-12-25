use axum::headers::{Header, HeaderName, HeaderValue};

static X: HeaderName = HeaderName::from_static("x-chamber-key");
static CUSTOM_CHAMBER_HEADER: &HeaderName = &X;

pub struct ChamberHeader(String);

impl ChamberHeader {
    pub fn key(self) -> String {
        self.0
    }
}

impl Header for ChamberHeader {
    fn name() -> &'static HeaderName {
        CUSTOM_CHAMBER_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;

        Ok(ChamberHeader(value.to_str().unwrap().to_owned()))
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
