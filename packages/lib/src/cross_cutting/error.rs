use std::error::Error;
use tracing::error;

pub trait LogError {
    fn log_error(self) -> Self;
}

impl<T, E: Error> LogError for Result<T, E> {
    fn log_error(self) -> Self {
        self.inspect_err(|error| {
            let chain = {
                let mut current: &dyn Error = error;
                let mut result = current.to_string();
                while let Some(error) = current.source() {
                    current = error;
                    result.push_str("\nCaused by: ");
                    result.push_str(&current.to_string());
                }
                result
            };
            error!(chain, "{error}");
        })
    }
}
