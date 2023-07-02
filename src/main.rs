use std::{
    fs::{self, File},
    io::Write,
    net::{IpAddr, SocketAddr},
    process::Command,
    str::FromStr,
};

use anyhow::Context;
use dotenvy::dotenv;
use pdf_converter::{
    pdf_converter_service_server::{PdfConverterService, PdfConverterServiceServer},
    ConvertToPdfRequest, ConvertToPdfResponse,
};
use tokio::signal;
use tonic::{transport::Server, Request, Response, Status};
use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod pdf_converter {
    tonic::include_proto!("pdf_converter");
}

#[derive(Debug, Default)]
pub struct PdfConverter {}

#[tonic::async_trait]
impl PdfConverterService for PdfConverter {
    async fn convert_to_pdf(
        &self,
        request: Request<ConvertToPdfRequest>,
    ) -> Result<Response<ConvertToPdfResponse>, Status> {
        let request = request.into_inner();
        debug!("Received request to create pdf");
        let temp_dir = tempfile::tempdir()?;

        let temp_body_file_path = temp_dir.path().join("document.adoc");
        let mut temp_body_file = File::create(&temp_body_file_path)?;
        write!(temp_body_file, "{}", request.body)?;
        debug!("Written: {:?}", temp_body_file_path);
        let temp_theme_file_path = temp_dir.path().join("custom-theme.yml");
        let mut temp_template_file = File::create(&temp_theme_file_path)?;
        write!(temp_template_file, "{}", request.template)?;
        debug!("Written: {:?}", temp_theme_file_path);
        let temp_result_file = temp_dir.path().join("result.pdf");

        let pdfcreator =
            std::env::var("PDFCREATOR").unwrap_or_else(|_| "asciidoctor-pdf".to_string());
        debug!("Using {} to create file", pdfcreator);
        let mut command = Command::new(pdfcreator);
        command.arg("-o");
        command.arg(&temp_result_file);
        command.arg(temp_body_file_path);
        debug!("Running command: {:?}", command);
        let _ = command.output().map_err(|err| {
            error!("Error running command: {:?}", err);
            Status::internal("Error running command")
        })?;

        debug!("Reading result");
        let result = fs::read(temp_result_file)?;
        debug!("Read {} bytes", result.len());

        Ok(Response::new(pdf_converter::ConvertToPdfResponse {
            result,
        }))
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    debug!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenv();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pdf_converter=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let addr = SocketAddr::new(
        IpAddr::from_str("::")?,
        std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .context("Cannot parse PORT")?,
    );
    let pdf_service = PdfConverter::default();

    tracing::info!("listening on {}", addr);
    Server::builder()
        .add_service(PdfConverterServiceServer::new(pdf_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
    Ok(())
}
