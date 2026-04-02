use shared_essential::presentation::dto::response_request::Messages;

pub(crate) trait Validate {
    fn validate(&self) -> ValidationResults;

    fn validate_json(&self) -> Option<Messages> {
        let res = self.validate();
        if !res.messages.is_empty() {
            tracing::info!(kind = "tcp", "validation messages {:?}", &res.messages);
            return Some(res.messages);
        }
        None
    }
}

#[derive(Default)]
pub(crate) struct ValidationResults {
    pub(crate) messages: Messages,
}

pub(crate) struct Utils {}

impl Utils {
    pub(crate) fn convert_vec_i64_to_string(v: &[i64]) -> String {
        v.iter().fold("".to_string(), |mut i, j| {
            if !i.is_empty() {
                i.push_str(", ")
            };
            i.push_str(j.to_string().as_str());
            i
        })
    }
}
