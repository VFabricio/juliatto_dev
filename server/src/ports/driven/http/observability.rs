use axum::{
    body::{Body, HttpBody},
    extract::ConnectInfo,
    http::{Request, Response, Version, header::AsHeaderName},
};
use regex::Regex;
use std::fmt::Display;
use std::net::{IpAddr, SocketAddr};
use std::sync::LazyLock;
use std::time::Duration;
use tower_http::{
    classify::{ClassifiedResponse, ClassifyResponse, NeverClassifyEos, SharedClassifier},
    trace::{DefaultOnBodyChunk, DefaultOnEos, DefaultOnRequest, TraceLayer},
};
use tracing::{Span, error, field::Empty};

use super::problem::Problem;

macro_rules! define_regex {
    ($name: ident, $key: expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(format!(r"^{}=([^=;]*)$", $key).as_str())
                .expect("Regex for extracting forwarded header field value is invalid")
        });
    };
}

define_regex!(FOR_REGEX, "for");
define_regex!(HOST_REGEX, "host");
define_regex!(PROTO_REGEX, "proto");

fn get_field_value<'a>(pair: &'a str, regex: &Regex) -> Option<&'a str> {
    regex
        .captures(pair.trim())?
        .iter()
        .last()?
        .map(|c| c.as_str())
}

fn parse_host_header(host: &str) -> (&str, Option<u16>) {
    let mut parts = host.splitn(2, ':');
    let name = parts.next();
    let port = parts.next().and_then(|p| p.parse::<u16>().ok());
    if let (Some(name), Some(port)) = (name, port) {
        return (name, Some(port));
    }
    (host, None)
}

#[derive(Clone, Copy)]
enum ClientAddress {
    WithPort(SocketAddr),
    WithoutPort(IpAddr),
}

impl ClientAddress {
    fn parse(input: &str) -> Option<Self> {
        input
            .parse()
            .ok()
            .map(Self::WithPort)
            .or(input.parse().ok().map(Self::WithoutPort))
    }
}

struct ForwardedHeaderFields<'a> {
    client: Option<ClientAddress>,
    host: Option<(&'a str, Option<u16>)>,
    proto: Option<&'a str>,
}

impl<'a> ForwardedHeaderFields<'a> {
    fn parse(request: &'a Request<Body>) -> Self {
        let mut result = ForwardedHeaderFields {
            client: None,
            host: None,
            proto: None,
        };

        let headers = request
            .headers()
            .get_all("forwarded")
            .into_iter()
            .flat_map(|h| h.to_str());

        for header in headers {
            if result.proto.is_none() {
                let pairs = header.split(';');
                for pair in pairs {
                    if let Some(proto) = get_field_value(pair, &PROTO_REGEX) {
                        result.proto = Some(proto);
                        break;
                    }
                }
            }
            if result.host.is_none() {
                let pairs = header.split(';');
                for pair in pairs {
                    if let Some(host) = get_field_value(pair, &HOST_REGEX) {
                        result.host = Some(parse_host_header(host));
                        break;
                    }
                }
            }
            if result.client.is_none() {
                let pairs = header.split(';');
                for pair in pairs {
                    if let Some(address) =
                        get_field_value(pair, &FOR_REGEX).and_then(ClientAddress::parse)
                    {
                        result.client = Some(address);
                        break;
                    }
                }
            }
        }

        result
    }
}

fn get_header<H: AsHeaderName>(request: &Request<Body>, header: H) -> Option<&str> {
    request.headers().get(header).and_then(|h| h.to_str().ok())
}

fn get_scheme<'a>(
    request: &'a Request<Body>,
    forwarded: &'a ForwardedHeaderFields<'a>,
) -> Option<&'a str> {
    forwarded
        .proto
        .or(get_header(request, "x-forwarded-proto"))
        .or(request.uri().scheme().map(|s| s.as_str()))
}

fn get_server_address<'a>(
    request: &'a Request<Body>,
    forwarded: &'a ForwardedHeaderFields<'a>,
) -> Option<(&'a str, Option<u16>)> {
    forwarded
        .host
        .or(get_header(request, "x-forwarded-host").map(parse_host_header))
        .or(get_header(request, "host").map(parse_host_header))
}

fn parse_x_forwarded_for(request: &Request<Body>) -> Option<ClientAddress> {
    for address in get_header(request, "x-forwarded-for")?.split(',') {
        if let Some(client_address) = ClientAddress::parse(address.trim()) {
            return Some(client_address);
        }
    }
    None
}

fn get_client_address(
    request: &Request<Body>,
    forwarded: &ForwardedHeaderFields,
) -> Option<(IpAddr, Option<u16>)> {
    forwarded
        .client
        .or(parse_x_forwarded_for(request))
        .map(|c| match c {
            ClientAddress::WithPort(s) => (s.ip(), Some(s.port())),
            ClientAddress::WithoutPort(i) => (i, None),
        })
}

fn make_span_with(listener_address: SocketAddr) -> impl Fn(&Request<Body>) -> Span + Clone {
    move |request| {
        let forwarded = &ForwardedHeaderFields::parse(request);
        let listener_ip = listener_address.ip().to_string();
        let (server_host, server_port) = get_server_address(request, forwarded)
            .unwrap_or((&listener_ip, Some(listener_address.port())));
        let client_address = get_client_address(request, forwarded);
        let client_ip = client_address.map(|(ip, _)| ip.to_string());
        let client_port = client_address.and_then(|(_, port)| port);
        let peer_address = request.extensions().get::<ConnectInfo<SocketAddr>>();
        let peer_ip = peer_address.map(|a| a.ip().to_string());
        let peer_port = peer_address.map(|a| a.port());

        let method = request.method().as_str();
        let path = request.uri().path();
        let scheme = get_scheme(request, forwarded);
        let query = request.uri().query();
        let version = match request.version() {
            Version::HTTP_09 => "0.9",
            Version::HTTP_10 => "1.0",
            Version::HTTP_11 => "1.1",
            Version::HTTP_2 => "2.0",
            Version::HTTP_3 => "3.0",
            other => &format!("{:?}", other),
        };
        let user_agent = get_header(request, "user_agent");
        let accept = get_header(request, "accept");
        let referer = get_header(request, "referer");
        let origin = get_header(request, "origin");
        let content_length =
            get_header(request, "content-length").and_then(|h| h.parse::<u64>().ok());
        let content_type = get_header(request, "content-type");

        let body_size = request.body().size_hint().lower();
        tracing::info_span!(
            "HTTP request",
            otel.name = format!("{} {}", method, path),
            http.request.method = method,
            url.scheme = scheme,
            url.path = path,
            url.query = query,
            server.address = server_host,
            server.port = server_port,
            client.address = client_ip,
            client.port = client_port,
            network.local.address = listener_address.ip().to_string(),
            network.local.port = listener_address.port(),
            network.peer.address = peer_ip,
            network.peer.port = peer_port,
            network.transport = "tcp",
            network.protocol.version = version,
            user_agent.original = user_agent,
            http.request.body.size = body_size,
            http.request.header.accept = accept,
            http.request.header.referer = referer,
            http.request.header.origin = origin,
            "http.request.header.content-length" = content_length,
            "http.request.header.content-type" = content_type,
            http.response.status_code = Empty,
            http.response.body.size = Empty,
            "http.response.header.content-length" = Empty,
            "http.response.header.content-type" = Empty,
            http.response.problem.type = Empty,
            http.response.problem.title = Empty,
            http.response.problem.instance = Empty,
            http.response.problem.extensions = Empty,
            user_agent.synthetic.type = Empty,
            exception.type = Empty,
            error.type = Empty,
        )
    }
}

fn get_response_header<H: AsHeaderName>(response: &Response<Body>, header: H) -> Option<&str> {
    response.headers().get(header).and_then(|h| h.to_str().ok())
}

fn on_response() -> impl Fn(&Response<Body>, Duration, &Span) + Clone {
    |response, _latency, span| {
        span.record("http.response.status_code", response.status().as_u16())
            .record(
                "http.response.body.size",
                response.body().size_hint().lower(),
            );
        get_response_header(response, "content-type").inspect(|h| {
            span.record("http.response.header.content-type", h);
        });
        get_response_header(response, "content-length")
            .and_then(|h| h.parse::<u64>().ok())
            .inspect(|h| {
                span.record("http.response.header.content-type", h);
            });
    }
}

fn on_failure() -> impl Fn(ClassifierError, Duration, &Span) + Clone {
    |error, _latency, span| match error {
        ClassifierError::Problem(problem) => {
            let problem_type = problem.problem_type();
            let title = problem.title();
            span.record("error.type", problem_type);
            span.record("http.response.problem.type", problem_type);
            span.record("http.response.problem.title", title);
            span.record(
                "http.response.problem.instance",
                problem.instance().map(|i| i.to_string()),
            );
            span.record(
                "http.response.problem.extensions",
                serde_json::to_string(&problem.extensions()).unwrap(),
            );
            error!("{}", title);
        }
        ClassifierError::Unknown => {
            span.record("error.type", "unknown");
            error!("{}", "unknown");
        }
        ClassifierError::Error(error) => {
            span.record("error.type", &error);
            error!("{}", error);
        }
    }
}

#[derive(Clone)]
pub struct Classifier {}

#[derive(Debug)]
pub enum ClassifierError {
    Problem(Problem),
    Unknown,
    Error(String),
}

impl ClassifyResponse for Classifier {
    type FailureClass = ClassifierError;
    type ClassifyEos = NeverClassifyEos<ClassifierError>;

    fn classify_error<E>(self, error: &E) -> Self::FailureClass
    where
        E: Display + 'static,
    {
        ClassifierError::Error(error.to_string())
    }

    fn classify_response<B>(
        self,
        response: &Response<B>,
    ) -> ClassifiedResponse<Self::FailureClass, Self::ClassifyEos> {
        if let Some(problem) = response.extensions().get::<Problem>() {
            ClassifiedResponse::Ready(Err(ClassifierError::Problem(problem.clone())))
        } else {
            ClassifiedResponse::Ready(Err(ClassifierError::Unknown))
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn make_trace_layer(
    listener_address: SocketAddr,
) -> TraceLayer<
    SharedClassifier<Classifier>,
    impl Fn(&Request<Body>) -> Span + Clone,
    DefaultOnRequest,
    impl Fn(&Response<Body>, Duration, &Span) + Clone,
    DefaultOnBodyChunk,
    DefaultOnEos,
    impl Fn(ClassifierError, Duration, &Span) + Clone,
> {
    tower_http::trace::TraceLayer::new(SharedClassifier::new(Classifier {}))
        .make_span_with(make_span_with(listener_address))
        .on_failure(on_failure())
        .on_response(on_response())
}
