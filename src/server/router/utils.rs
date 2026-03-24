use std::path::PathBuf;

use axum::body::Body;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub async fn image_to_response(filename: PathBuf) -> impl IntoResponse {
    match File::open(&filename).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            let body = Body::from_stream(stream);

            Response::builder()
                .header(header::CONTENT_TYPE, "image/png")
                .body(body)
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(404)
            .body("Image not found".into())
            .unwrap(),
    }
}
