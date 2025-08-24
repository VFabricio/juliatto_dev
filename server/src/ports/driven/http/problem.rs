use std::collections::HashMap;

use axum::response::IntoResponse;
use http::{StatusCode, Uri};
use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::{Value, to_string};

#[derive(Clone, Debug)]
pub enum ProblemType {
    HealthChecker(HealthChecker),
    Router(Router),
}

impl ProblemType {
    fn get_type(&self) -> &str {
        match self {
            Self::HealthChecker(h) => h.get_type(),
            Self::Router(r) => r.get_type(),
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::HealthChecker(h) => h.title(),
            Self::Router(r) => r.get_type(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HealthChecker {
    Unhealthy,
}

impl HealthChecker {
    fn get_type(&self) -> &str {
        match self {
            Self::Unhealthy => "/health-check/unhealthy",
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::Unhealthy => "Service unhealthy.",
        }
    }
}

impl From<HealthChecker> for ProblemType {
    fn from(value: HealthChecker) -> Self {
        Self::HealthChecker(value)
    }
}

#[derive(Clone, Debug)]
pub enum Router {
    NotFound,
}

impl Router {
    fn get_type(&self) -> &str {
        match self {
            Self::NotFound => "/router/not-found",
        }
    }

    fn title(&self) -> &str {
        match self {
            Self::NotFound => "Resource not found.",
        }
    }
}

impl From<Router> for ProblemType {
    fn from(value: Router) -> Self {
        Self::Router(value)
    }
}

#[derive(Clone, Debug)]
pub struct Problem {
    problem_type: ProblemType,
    detail: String,
    status: StatusCode,
    instance: Option<Uri>,
    extensions: HashMap<String, Value>,
}

impl Problem {
    pub fn new<S: Into<String>>(problem_type: ProblemType, detail: S, status: StatusCode) -> Self {
        Self {
            problem_type,
            detail: detail.into(),
            status,
            instance: None,
            extensions: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_instance(mut self, instance: Uri) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn with_extension(mut self, key: String, value: Value) -> Self {
        self.extensions.insert(key, value);
        self
    }

    pub fn get_type(&self) -> &str {
        self.problem_type.get_type()
    }

    pub fn title(&self) -> &str {
        self.problem_type.title()
    }

    pub fn instance(&self) -> Option<Uri> {
        self.instance.clone()
    }

    pub fn extensions(&self) -> &HashMap<String, Value> {
        &self.extensions
    }
}

impl Serialize for Problem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4 + self.extensions.len()))?;
        map.serialize_entry("type", self.get_type())?;
        map.serialize_entry("title", self.title())?;
        map.serialize_entry("status", &self.status.as_u16())?;
        map.serialize_entry("detail", &self.detail)?;
        for (key, value) in &self.extensions {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> axum::response::Response {
        let mut response = (self.status, to_string(&self).unwrap()).into_response();
        response
            .headers_mut()
            .insert("content-type", "application/problem+json".parse().unwrap());
        response.extensions_mut().insert(self);
        response
    }
}
