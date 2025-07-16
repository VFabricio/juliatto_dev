use axum::{
    body::{Body, HttpBody},
    extract::ConnectInfo,
    http::{Request, Response, Version},
};
use regex::Regex;
use std::net::SocketAddr;
use tower_http::classify::ServerErrorsFailureClass;
use tracing::{field::Empty, Span};

#[derive(Debug)]
struct ForwardedHeaderFields<'a> {
    by: Option<&'a str>,
    for_field: Option<&'a str>,
    host: Option<&'a str>,
    proto: Option<&'a str>,
}

fn parse_forwarded_header(request: &Request<Body>) -> ForwardedHeaderFields<'_> {
    fn get_field_value<'a>(pair: &'a str, key: &str) -> Option<&'a str> {
        let regex =
            Regex::new(format!(r"^{}=([^=;]*)$", key).as_str()).expect("The regex must be valid.");
        regex
            .captures(pair.trim())?
            .iter()
            .last()?
            .map(|c| c.as_str())
    }

    let mut result = ForwardedHeaderFields {
        by: None,
        for_field: None,
        host: None,
        proto: None,
    };

    let sections = request
        .headers()
        .get_all("forwarded")
        .iter()
        .flat_map(|h| h.to_str().unwrap_or("").split(','));

    for section in sections {
        for pair in section.split(';') {
            if let Some(by) = get_field_value(pair, "by") {
                result.by = Some(by);
            }

            if let Some(for_field) = get_field_value(pair, "for") {
                result.for_field = Some(for_field);
            }

            if let Some(host) = get_field_value(pair, "host") {
                result.host = Some(host);
            }

            if let Some(proto) = get_field_value(pair, "proto") {
                result.proto = Some(proto);
            }
        }
    }
    result
}

fn get_client_information(
    request: &Request<Body>,
    peer_addr: Option<SocketAddr>,
) -> Option<SocketAddr> {
    if let Some(forwarded_for) = parse_forwarded_header(request).for_field {
        if let Ok(address) = forwarded_for.parse::<SocketAddr>() {
            return Some(address);
        }
    }

    let x_forwarded_for = request
        .headers()
        .get_all("x-forwarded-for")
        .iter()
        .flat_map(|h| h.to_str().unwrap_or("").split(','));

    for section in x_forwarded_for {
        if let Ok(address) = section.trim().parse::<SocketAddr>() {
            return Some(address);
        }
    }

    peer_addr
}

fn get_scheme(request: &Request<Body>) -> &str {
    if let Some(proto) = parse_forwarded_header(request).proto {
        return proto;
    }

    if let Some(proto) = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|header| header.to_str().ok())
    {
        return proto;
    }

    request
        .uri()
        .scheme()
        .map_or("unknown", |scheme| scheme.as_str())
}

fn get_server_host(request: &Request<Body>) -> Option<SocketAddr> {
    if let Some(host) = parse_forwarded_header(request).host {
        if let Ok(addr) = host.parse::<SocketAddr>() {
            return Some(addr);
        }
    }

    if let Some(host) = request
        .headers()
        .get("x-forwarded-host")
        .and_then(|header| header.to_str().ok())
    {
        if let Ok(addr) = host.parse::<SocketAddr>() {
            return Some(addr);
        }
    }

    if let Some(host) = request
        .headers()
        .get("host")
        .and_then(|header| header.to_str().ok())
    {
        if let Ok(addr) = host.parse::<SocketAddr>() {
            return Some(addr);
        }
    }

    None
}

fn get_header_value<'a>(
    request: &'a Request<Body>,
    header_name: &'static str,
) -> Box<dyn tracing::Value + 'a> {
    request
        .headers()
        .get(header_name)
        .map_or(Box::new(Empty), move |header| {
            Box::new(header.to_str().unwrap_or("INVALID_UTF8"))
        })
}

fn get_header_as_u64<'a>(
    request: &'a Request<Body>,
    header_name: &'static str,
) -> Box<dyn tracing::Value + 'a> {
    request
        .headers()
        .get(header_name)
        .and_then(|header| header.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(Box::new(Empty), |value| Box::new(value))
}

fn get_response_header_as_u64(response: &Response<Body>, header_name: &'static str) -> Option<u64> {
    response
        .headers()
        .get(header_name)
        .and_then(|header| header.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

fn get_response_header_value<'a>(
    response: &'a Response<Body>,
    header_name: &'static str,
) -> Option<&'a str> {
    response
        .headers()
        .get(header_name)
        .and_then(|header| header.to_str().ok())
}

pub fn make_span_with(
    host: std::net::IpAddr,
    port: u16,
) -> impl Fn(&Request<Body>) -> Span + Clone {
    move |request: &Request<Body>| {
        let body_size = request.body().size_hint().lower();

        let method = request.method().as_str();
        let path = request.uri().path();
        let scheme = get_scheme(request);

        let query: Box<dyn tracing::Value> = request
            .uri()
            .query()
            .map_or(Box::new(Empty), |q| Box::new(q));

        let version = match request.version() {
            Version::HTTP_09 => "0.9",
            Version::HTTP_10 => "1.0",
            Version::HTTP_11 => "1.1",
            Version::HTTP_2 => "2.0",
            Version::HTTP_3 => "3.0",
            other => &format!("{:?}", other),
        };

        let user_agent = get_header_value(request, "user-agent");

        let peer_addr = request
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|connect_info| connect_info.0);

        let client_addr = get_client_information(request, peer_addr);
        let (client_address, client_port): (Box<dyn tracing::Value>, Box<dyn tracing::Value>) =
            client_addr.map_or((Box::new(Empty), Box::new(Empty)), |addr| {
                (Box::new(addr.ip().to_string()), Box::new(addr.port()))
            });

        let (peer_address, peer_port): (Box<dyn tracing::Value>, Box<dyn tracing::Value>) =
            peer_addr.map_or((Box::new(Empty), Box::new(Empty)), |addr| {
                (Box::new(addr.ip().to_string()), Box::new(addr.port()))
            });

        let content_length = get_header_as_u64(request, "content-length");
        let content_type = get_header_value(request, "content-type");
        let origin = get_header_value(request, "origin");
        let referer = get_header_value(request, "referer");
        let accept = get_header_value(request, "accept");

        let (server_address, server_port) = get_server_host(request)
            .map_or((host.to_string(), port), |addr| {
                (addr.ip().to_string(), addr.port())
            });

        tracing::info_span!(
            "HTTP request",
            otel.name = format!("{} {}", method, path),
            http.request.method = method,
            url.scheme = scheme,
            url.path = path,
            url.query = query,
            server.address = server_address,
            server.port = server_port,
            client.address = client_address,
            client.port = client_port,
            network.local.address = host.to_string(),
            network.local.port = port,
            network.peer.address = peer_address,
            network.peer.port = peer_port,
            network.transport = "tcp",
            network.protocol.version = version,
            user_agent.original = user_agent,
            http.request.body.size = body_size,
            http.response.body.size = Empty,
            http.response.status_code = Empty,
            "http.request.header.content-length" = content_length,
            "http.request.header.content-type" = content_type,
            "http.request.header.origin" = origin,
            "http.request.header.referer" = referer,
            "http.request.header.accept" = accept,
            "http.response.header.content-length" = Empty,
            "http.response.header.content-type" = Empty,
            user_agent.synthetic.type = Empty,
            exception.message = Empty,
            exception.type = Empty,
            error.type = Empty,
        )
    }
}

pub fn on_response() -> impl Fn(&Response<Body>, std::time::Duration, &Span) + Clone {
    |response: &Response<Body>, _latency: std::time::Duration, span: &Span| {
        let status_code = response.status();
        let body_size = response.body().size_hint().lower();
        span.record("http.response.status_code", status_code.as_str());
        span.record("http.response.body.size", body_size);

        if let Some(content_length) = get_response_header_as_u64(response, "content-length") {
            span.record("http.response.header.content-length", content_length);
        }

        if let Some(content_type) = get_response_header_value(response, "content-type") {
            span.record("http.response.header.content-type", content_type);
        }
    }
}

pub fn on_failure() -> impl Fn(ServerErrorsFailureClass, std::time::Duration, &Span) + Clone {
    |error: ServerErrorsFailureClass, _latency: std::time::Duration, _span: &Span| {
        println!("{:?}", error);
    }
}
