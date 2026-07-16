//! Data model for the Neoland web console.

#[derive(Clone)]
pub struct Message {
    pub role: &'static str,
    pub body: String,
    pub code: Option<&'static str>,
}
