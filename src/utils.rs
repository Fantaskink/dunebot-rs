use crate::Error;

use reqwest::Client as ReqwestClient;

extern crate color_thief;
extern crate image;

use color_thief::get_palette;
use color_thief::ColorFormat;
use image::load_from_memory;

pub fn format_currency(value: u64) -> String {
    let value_str = value.to_string().chars().rev().collect::<Vec<_>>();
    let value_str = value_str
        .chunks(3)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(",");
    value_str.chars().rev().collect::<String>()
}

pub async fn get_image_primary_color(url: &str) -> Result<(u8, u8, u8), Error> {
    let reqwest_client = ReqwestClient::new();
    let response = reqwest_client.get(url).send().await?;
    let image_data = response.bytes().await?;
    let image = load_from_memory(&image_data)?;
    let pixels = image.to_rgba8();
    let pixels = pixels.as_raw();
    let palette = get_palette(pixels, ColorFormat::Rgba, 10, 2)?;
    let primary_color = palette.first().ok_or("No primary color found")?;
    Ok((primary_color.r, primary_color.g, primary_color.b))
}
