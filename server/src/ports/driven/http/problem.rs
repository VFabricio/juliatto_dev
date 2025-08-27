use std::collections::HashMap;

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::{Value, to_string};

#[derive(Clone, Debug)]
pub struct ProblemBuilder<'a> {
    problem_type: &'a str,
    title: &'a str,
    status: StatusCode,
}

impl<'a> ProblemBuilder<'a> {
    pub const METHOD_NOT_ALLOWED: Self = Self {
        problem_type: "/router/not-allowed",
        title: "Method not allowed.",
        status: StatusCode::METHOD_NOT_ALLOWED,
    };

    pub const ROUTE_NOT_FOUND: Self = Self {
        problem_type: "/router/not-found",
        title: "Route not found.",
        status: StatusCode::NOT_FOUND,
    };

    pub const UNHEALTHY: Self = Self {
        problem_type: "/health-check/unhealthy",
        title: "The service is unhealthy.",
        status: StatusCode::SERVICE_UNAVAILABLE,
    };

    pub fn detail(self, detail: String) -> Problem {
        Problem {
            problem_type: self.problem_type.into(),
            title: self.title.into(),
            status: self.status,
            detail,
            instance: None,
            extensions: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Problem {
    problem_type: String,
    title: String,
    detail: String,
    status: StatusCode,
    instance: Option<String>,
    extensions: HashMap<String, Value>,
}

impl Problem {
    pub fn with_instance(mut self, instance: String) -> Self {
        self.instance = Some(instance);
        self
    }

    pub fn with_extension(mut self, key: String, value: Value) -> Self {
        self.extensions.insert(key, value);
        self
    }

    pub fn problem_type(&self) -> &str {
        &self.problem_type
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn instance(&self) -> Option<&str> {
        match &self.instance {
            None => None,
            Some(i) => Some(i.as_str()),
        }
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
        map.serialize_entry("type", self.problem_type())?;
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
    fn into_response(self) -> Response {
        let mut response = (self.status, to_string(&self).unwrap()).into_response();
        response
            .headers_mut()
            .insert("content-type", "application/problem+json".parse().unwrap());
        response.extensions_mut().insert(self);
        response
    }
}
