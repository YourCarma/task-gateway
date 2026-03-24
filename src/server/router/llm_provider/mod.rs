use std::fs as std_fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use crate::modules::llm_provider::errors::GenerationsErrors;
use crate::server::errors::ServerError;
use axum::body::Bytes;
use base64::Engine;
use base64::engine::general_purpose;

pub mod image_editing;
pub mod image_generation;

pub fn save_file_from_bytes(
    universe: String,
    user_id: i64,
    data: Bytes,
    task_id: &String,
    image_name: String,
) -> Result<PathBuf, ServerError> {
    tracing::debug!("Saving image {}", image_name);
    let save_dir_path = format!(
        "images/editing/{universe}/{user}/{task_id}/raw",
        universe = universe,
        user = user_id,
        task_id = task_id
    );
    std_fs::create_dir_all(&save_dir_path)?;
    let filename = format!(
        "{save_dir_path}/{image_name}.png",
        image_name = image_name,
        save_dir_path = save_dir_path
    );
    let mut file = File::create(&filename)?;
    file.write_all(&data)?;
    let filename_converted = PathBuf::from(filename);
    tracing::info!("Image saved!");
    Ok(filename_converted)
}

pub fn image_to_base64(image_path: PathBuf) -> Result<String, GenerationsErrors> {
    tracing::debug!("Convreting image to base64");
    const B64_DATA: &str = "data:image/png;base64,";
    let file = std_fs::read(image_path)?;
    let b64_image = general_purpose::STANDARD.encode(file);
    let converted = format!(
        "{b64_start}{b64_image}",
        b64_start = B64_DATA,
        b64_image = b64_image
    );
    tracing::debug!("Image converted to base64");
    Ok(converted)
}
