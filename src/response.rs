use serde_json;

#[derive(Debug, Serialize)]
pub struct TaggedResponse {
    pub id: String,
    pub response: serde_json::Value,
}

impl TaggedResponse {
    pub fn new(id: &str, response: &str) -> TaggedResponse {
        // Deserialise JSON response if possible
        let response = serde_json::from_str(response).unwrap_or_else(|_| {
            json!({
                "error_message": "Malformed JSON received from server.",
                "routes": [],
                "status": "MALFORMED_JSON"
            })
        });

        // Construct TaggedResponse
        TaggedResponse {
            id: id.to_string(),
            response,
        }
    }
}
