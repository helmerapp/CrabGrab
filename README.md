# CrabGrab

A cross-platform screen-capturing crate for rust

**Note:** Crate built by AugmendTech. We initially made PRs to the original but have since then forked it for our own use at Helmer.

## Helmer Team Changelog

1. Allow specific windows to be excluded from recordings
2. Allow specifying color space on macOS
3. Expose display and window IDs as u32
4. Expose CMSampleBuffer on macOS
5. ...

# Original Readme

The rest of the README is preserved from the original, minus a few things.

[![docs.rs](https://img.shields.io/docsrs/crabgrab)](https://docs.rs/crabgrab/)
[MacOS Documentation](https://augmendtech.github.io/CrabGrab/macos_docs/crabgrab/index.html)

Capturing video from screens and applications can be very hard, and it's even worse when you want to do it in a cross-platform application. CrabGrab makes it easy to do continuous frame capture that can be used for individual screenshots or for capturing video. It also includes common functionality needed for enumerating screens and applications. You can get from a window to a pixel buffer in just a few lines of code that will work on both Windows and MacOS.

```rust
#[tokio::main]
async fn main() {
    let token = match CaptureStream::test_access(false) {
        Some(token) => token,
        None => CaptureStream::request_access(false).await.expect("Expected capture access")
    };
    let filter = CapturableContentFilter::NORMAL_WINDOWS;
    let content = CapturableContent::new(filter).await.unwrap();
    let config = CaptureConfig::with_display(content.displays().next().unwrap(), CapturePixelFormat::Bgra8888);

    let mut stream = CaptureStream::new(token, config, |stream_event| {
        // The stream_event here could be a video frame or audio frame
        println!("result: {:?}", stream_event);
    }).unwrap();

    std::thread::sleep(Duration::from_millis(2000));

    stream.stop().unwrap();
}
```

CrabGrab makes it easy.

## Features:

-   Screen and window capture supported
-   Compatible with multiple GPU APIs:
    -   Wgpu
    -   DX11
    -   DXGI
    -   Metal
    -   IOSurface
-   Easy frame bitmap generation
-   Platform specific extension features
-   Screenshot facility
-   Sound capture (WIP)

## Examples

Examples showing how to use the CrabGrab crate can be found at [crabgrab/examples](examples). You can run the examples from the repository:

`cargo run --example <example_name>`

Note that feature examples will require that feature:

`cargo run --example <example name> --feature <feature name>`

## MacOS Docs

Unfortunately due to our dependence on metal-rs, building docs for macos doesn't work on docs.rs, since they use linux containers. As a workaround, we host macos documentation in this repository - link above.
