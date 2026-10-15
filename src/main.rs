mod model;
mod outstation;
mod sim;
mod state;
mod web;

use std::collections::VecDeque;
use std::path::PathBuf;

use model::Config;
use state::{App, Shared};

struct Args {
    data: PathBuf,
    http: String,
    dnp3_port: Option<u16>,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args {
        data: PathBuf::from("simulator-data.json"),
        http: "0.0.0.0:8080".into(),
        dnp3_port: std::env::var("DNP3_PORT").ok().and_then(|v| v.parse().ok()),
    };
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--data" => args.data = iter.next().ok_or("--data butuh path file")?.into(),
            "--http" => args.http = iter.next().ok_or("--http butuh host:port")?,
            "--dnp3-port" => {
                let value = iter.next().ok_or("--dnp3-port butuh nomor port")?;
                args.dnp3_port = Some(
                    value
                        .parse()
                        .map_err(|_| format!("port DNP3 tidak valid: {value}"))?,
                );
            }
            "-h" | "--help" => {
                println!(
                    "simulator-rtu-dnp3 [--data simulator-data.json] [--http 0.0.0.0:8080] [--dnp3-port 20000]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("argumen tidak dikenal: {other}")),
        }
    }
    Ok(args)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_target(false).init();

    let args = parse_args().unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });
    let mut config = Config::load(&args.data).unwrap_or_else(|err| {
        eprintln!("gagal membaca penyimpanan: {err}");
        std::process::exit(1);
    });
    if let Some(port) = args.dnp3_port {
        config.settings.port = port;
    }

    let shared = Shared::new(App {
        config,
        path: args.data.clone(),
        runtime: None,
        connected: false,
        log: VecDeque::new(),
        error: None,
        locked_port: args.dnp3_port,
    });
    shared.lock().persist();

    if let Err(err) = outstation::start(&shared).await {
        tracing::error!("{err}");
        let mut app = shared.lock();
        app.error = Some(err.clone());
        app.log(false, err);
    }

    tokio::spawn(sim::run(shared.clone()));

    let listener = tokio::net::TcpListener::bind(&args.http)
        .await
        .unwrap_or_else(|err| {
            eprintln!("gagal membuka UI di {}: {err}", args.http);
            std::process::exit(1);
        });
    tracing::info!(
        "UI simulator: http://{}  (penyimpanan: {})",
        args.http,
        args.data.display()
    );
    if let Err(err) = axum::serve(listener, web::router(shared)).await {
        eprintln!("server UI berhenti: {err}");
    }
}
