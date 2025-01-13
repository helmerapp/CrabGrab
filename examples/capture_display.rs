use std::time::Duration;

use crabgrab::prelude::*;

#[tokio::main]
async fn main() { 
    let token = match CaptureStream::test_access(false) {
        Some(token) => token,
        None => CaptureStream::request_access(false).await.expect("Expected capture access")
    };
    let filter = CapturableContentFilter::DISPLAYS;
    let content = CapturableContent::new(filter).await.unwrap();

    let crop = Rect {
        origin: Point { x: 1116.0, y: 309.0},
        size: Size {
            width: 496.0,
            height: 440.0,
        },
    };

    let config = CaptureConfig::with_display(content.displays().next().unwrap(), CapturePixelFormat::Bgra8888, None).with_source_rect(
        Some(crop)
    );

    let mut stream = CaptureStream::new(token, config, |result| {
        if let StreamEvent::Video(frame) = result.expect("Expected stream event") {
            println!("Got frame: {}", frame.frame_id());
        }
    }).unwrap();

    std::thread::sleep(Duration::from_millis(20000));

    stream.stop().unwrap();
}
