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
    println!(
        "{} {} {}",
        req.method().to_string().blue(),
        req.uri().to_string().blue(),
        format!("{:?}", req.version()).blue()
    );

    for (k, v) in req.headers() {
        let v_str = match v.to_str() {
            Ok(v) => v,
            Err(err) => {
                println!(
                    "{}",
                    format!("weird header {} not ASCII: {:?} | {:?}", k, v, err).red()
                );
                continue;
            }
        };
        println!("{}: {}", k.to_string().cyan(), v_str);
    }

    let body = match hyper::body::to_bytes(req.into_body()).await {
        Ok(b) => b,
        Err(err) => {
            println!(
                "{}",
                format!("cannot concatenate request body: {err:?}").red()
            );
            Bytes::new()
        }
    };
    match std::str::from_utf8(&body) {
        Ok(s) => {
            println!();
            println!("{s}");
        }
        Err(err) => {
            println!(
                "{}",
                format!("cannot concatenate request body: {err:?}").red()
            );
        }
    }

    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();

    Ok(res)
}
