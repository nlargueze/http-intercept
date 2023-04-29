//! A simple echo server

use std::{convert::Infallible, net::SocketAddr};

use clap::Parser;
use colored::Colorize;
use hyper::{
    body::Bytes,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};

#[tokio::main]
async fn main() {
    real_main().await;
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
// TODO: add color to output (cf. https://github.com/clap-rs/clap/issues/4132)
struct Args {
    /// Port
    #[arg(short, long, default_value("8080"))]
    port: u16,
}

/// Real main
async fn real_main() {
    let args = Args::parse();

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler)) });
    let server = Server::bind(&addr).serve(make_svc);

    println!(
        "{}",
        format!("Echo server listening on http://{}", addr).yellow()
    );
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

/// Handler
async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let req_str = convert_req_to_str(req).await;
    println!("{req_str}");

    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();

    Ok(res)
}

/// Format a request
async fn convert_req_to_str(req: Request<Body>) -> String {
    let head_str = format!(
        "{} {} {}",
        req.method().to_string().blue(),
        req.uri().to_string().blue(),
        format!("{:?}", req.version()).blue()
    );

    let mut headers_str = String::new();
    for (k, v) in req.headers() {
        if !headers_str.is_empty() {
            headers_str.push('\n');
        }
        match v.to_str() {
            Ok(v) => {
                headers_str.push_str(format!("{}: {}", k.to_string().cyan(), v).as_str());
            }
            Err(err) => {
                headers_str.push_str(
                    format!(
                        "{}",
                        format!("weird header {} not ASCII: {:?} | {:?}", k, v, err).red()
                    )
                    .as_str(),
                );
                continue;
            }
        }
    }

    let mut body_str = String::new();
    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(err) => {
            body_str.push_str(
                format!(
                    "{}",
                    format!("cannot concatenate request body: {err:?}").red()
                )
                .as_str(),
            );
            Bytes::new()
        }
    };

    match std::str::from_utf8(&body) {
        Ok(s) => {
            body_str.push_str(s);
        }
        Err(err) => {
            body_str.push_str(format!("{}", format!("not UTF-8: {err:?}").red()).as_str());
        }
    }

    format!("{head_str}\n{headers_str}\n\n{body_str}")
}
