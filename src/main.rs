use anyhow::Result;
use log::info;
use std::{
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use crabgrab::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    simple_logger::SimpleLogger::new().init().unwrap();
    start_recorder().await;
    loop {
        thread::yield_now();
    }
}

async fn start_recorder() -> Result<()> {
    let fps = Arc::new(AtomicI32::new(0));

    let fps_loop = Arc::clone(&fps);
    thread::spawn(move || loop {
        println!("fps: {}", fps_loop.load(Ordering::Relaxed));
        fps_loop.store(0, Ordering::Relaxed);
        thread::sleep(Duration::from_secs(1));
    });

    let token = match CaptureStream::test_access(false) {
        Some(token) => token,
        None => CaptureStream::request_access(false)
            .await
            .expect("Expected capture access"),
    };
    let filter = CapturableContentFilter::EVERYTHING;
    let content = CapturableContent::new(filter).await?;

    let pixel_format = CaptureStream::supported_pixel_formats()[0];
    info!("pixel format: {:#?}", pixel_format);
    let config = CaptureConfig::with_display(
        content
            .displays()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Expected at least one display"))?,
        pixel_format,
    );

    let fps_recorder = Arc::clone(&fps);
    loop {
        let result = crabgrab::feature::screenshot::take_screenshot(token, config.clone()).await;
        let start = Instant::now();
        println!("received frame in {}ms", start.elapsed().as_millis());
        fps_recorder.fetch_add(1, Ordering::Relaxed);
    }
}
