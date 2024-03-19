#![cfg(feature = "bitmap")]

use half::f16;


use crate::prelude::VideoFrame;
#[cfg(target_os = "macos")]
use crate::platform::macos::frame::MacosVideoFrame;
#[cfg(target_os = "macos")]
use crate::platform::platform_impl::objc_wrap::CVPixelFormat;

#[cfg(target_os = "windows")]
use crate::feature::dxgi::{WindowsDxgiVideoFrame, WindowsDxgiVideoFrameError};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Dxgi::{DXGI_MAPPED_RECT, DXGI_MAP_READ};
#[cfg(target_os = "windows")]
use windows::Graphics::DirectX::DirectXPixelFormat;

pub struct FrameBitmapBgraUnorm8x4 {
    pub data: Box<[[u8; 4]]>,
    pub width:  usize,
    pub height: usize,
}

pub struct FrameBitmapRgbaUnormPacked1010102 {
    pub data: Box<[u32]>,
    pub width:  usize,
    pub height: usize,
}

pub struct FrameBitmapRgbaF16x4 {
    pub data: Box<[[f16; 4]]>,
    pub width:  usize,
    pub height: usize,
}

pub enum VideoRange {
    Video,
    Full,
}

pub struct FrameBitmapYCbCr {
    pub luma_data: Box<[u8]>,
    pub chroma_data: Box<[[u8; 2]]>,
    pub width: usize,
    pub luma_height: usize,
    pub chroma_height: usize,
    pub range: VideoRange,
}

pub enum FrameBitmap {
    BgraUnorm8x4(FrameBitmapBgraUnorm8x4),
    RgbaUnormPacked1010102(FrameBitmapRgbaUnormPacked1010102),
    RgbaF16x4(FrameBitmapRgbaF16x4),
    YCbCr(FrameBitmapYCbCr),
}

pub trait VideoFrameBitmap {
    fn get_bitmap(&self) -> Result<FrameBitmap, VideoFrameBitmapError>;
}

pub enum VideoFrameBitmapError {
    Other(String),

}

impl VideoFrameBitmap for VideoFrame {
    fn get_bitmap(&self) -> Result<FrameBitmap, VideoFrameBitmapError> {
        #[cfg(target_os = "windows")]
        {
            let (width, height) = self.impl_video_frame.frame_size;
            match self.get_dxgi_surface() {
                Err(WindowsDxgiVideoFrameError::Other(x)) => Err(VideoFrameBitmapError::Other(x)),
                Ok((surface, pixel_format)) => {
                    let mut locked_map_rect = DXGI_MAPPED_RECT::default();
                    unsafe {
                        match surface.Map(&mut locked_map_rect as *mut _, DXGI_MAP_READ) {
                            Ok(_) => {},
                            Err(e) => return Err(VideoFrameBitmapError::Other(format!("Failed to map dxgi surface: {}", e.to_string()))),
                        }
                        match pixel_format {
                            DirectXPixelFormat::B8G8R8A8UIntNormalized => {
                                let mut image_data = vec![[0u8; 4]; width * height];
                                let bpr = locked_map_rect.Pitch as usize;
                                let surface_slice = std::slice::from_raw_parts(locked_map_rect.pBits as *const u8, bpr * height);
                                for y in 0..height {
                                    let source_slice = bytemuck::cast_slice::<_, [u8; 4]>(&surface_slice[(bpr * y)..(bpr * y + 4 * width)]);
                                    image_data[(width * y)..(width * y + width)].copy_from_slice(source_slice);
                                }
                                let _ = surface.Unmap();
                                Ok(FrameBitmap::BgraUnorm8x4(FrameBitmapBgraUnorm8x4 {
                                    data: image_data.into_boxed_slice(),
                                    width,
                                    height,
                                }))
                            },
                            DirectXPixelFormat::R10G10B10A2UIntNormalized => {
                                let mut image_data = vec![0u32; width * height];
                                let bpr = locked_map_rect.Pitch as usize;
                                let surface_slice = std::slice::from_raw_parts(locked_map_rect.pBits as *const u8, bpr * height);
                                for y in 0..height {
                                    let source_slice = bytemuck::cast_slice::<_, u32>(&surface_slice[(bpr * y)..(bpr * y + 4 * width)]);
                                    image_data[(width * y)..(width * y + width)].copy_from_slice(source_slice);
                                }
                                let _ = surface.Unmap();
                                Ok(FrameBitmap::RgbaUnormPacked1010102(FrameBitmapRgbaUnormPacked1010102 {
                                    data: image_data.into_boxed_slice(),
                                    width,
                                    height,
                                }))
                            },
                            _ => {
                                Err(VideoFrameBitmapError::Other("Unknown or unsupported pixel format on DXGISurface".to_string()))
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            let iosurface = match &self.impl_video_frame {
                MacosVideoFrame::SCStream(sc_frame) => {
                    match sc_frame.sample_buffer.get_image_buffer().map(|image_buffer| image_buffer.get_iosurface()).flatten() {
                        Some(iosurface) => iosurface,
                        None => return Err(VideoFrameBitmapError::Other("Failed to get iosurface".to_string())),
                    }
                },
                MacosVideoFrame::CGDisplayStream(cg_display_frame) => {
                    cg_display_frame.io_surface.clone()
                }
            };
            if let Ok(lock_gaurd) = iosurface.lock(true, false) {
                let pixel_format = iosurface.get_pixel_format();
                match pixel_format {
                    Some(CVPixelFormat::BGRA8888) => {
                        let bpr = iosurface.get_bytes_per_row();
                        let height = iosurface.get_height();
                        let width = iosurface.get_width();
                        let mut image_data = vec![[0; 4]; width * height];
                        let base_address = lock_gaurd.get_base_address().ok_or(VideoFrameBitmapError::Other("Failed to get base address of iosurface".into()))?;
                        let iosurface_slice = unsafe { std::slice::from_raw_parts(base_address as *const u8, bpr * height) };
                        for y in 0..height {
                            let source_slice = bytemuck::cast_slice::<_, [u8; 4]>(&iosurface_slice[(bpr * y)..(bpr * y + 4 * width)]);
                            image_data[(width * y)..(width * y + width)].copy_from_slice(source_slice);
                        }
                        Ok(FrameBitmap::BgraUnorm8x4(FrameBitmapBgraUnorm8x4 {
                            data: image_data.into_boxed_slice(),
                            width,
                            height,
                        }))
                    },
                    Some(CVPixelFormat::V420) |
                    Some(CVPixelFormat::F420) => {
                        let width = iosurface.get_width();

                        let luma_bpr = iosurface.get_bytes_per_row_of_plane(0);
                        let luma_height = iosurface.get_height_of_plane(0);
                        let mut luma_image_data = vec![0u8; width * luma_height];
                        let luma_base_address = lock_gaurd.get_base_address_of_plane(0).ok_or(VideoFrameBitmapError::Other("Failed to get base address of iosurface".into()))?;
                        let luma_iosurface_slice = unsafe { std::slice::from_raw_parts(luma_base_address as *const u8, luma_bpr * luma_height) };

                        for y in 0..luma_height {
                            let luma_source_slice = &luma_iosurface_slice[(luma_bpr * y)..(luma_bpr * y * width)];
                            luma_image_data[(width * y)..(width * y + width)].copy_from_slice(luma_source_slice);                            
                        }

                        let chroma_bpr = iosurface.get_bytes_per_row_of_plane(1);
                        let chroma_height = iosurface.get_height_of_plane(1);
                        let mut chroma_image_data = vec![[0u8; 2]; width * chroma_height];
                        let chroma_base_address = lock_gaurd.get_base_address_of_plane(1).ok_or(VideoFrameBitmapError::Other("Failed to get base address of iosurface".into()))?;
                        let chroma_iosurface_slice = unsafe { std::slice::from_raw_parts(chroma_base_address as *const u8, chroma_bpr * chroma_height) };

                        for y in 0..chroma_height {
                            let chroma_source_slice = bytemuck::cast_slice::<_, [u8; 2]>(&chroma_iosurface_slice[(chroma_bpr * y)..(chroma_bpr * y + 2 * width)]);
                            chroma_image_data[(width * y)..(width * y + width)].copy_from_slice(chroma_source_slice);
                        }

                        Ok(FrameBitmap::YCbCr(FrameBitmapYCbCr {
                            luma_data: luma_image_data.into_boxed_slice(),
                            chroma_data: chroma_image_data.into_boxed_slice(),
                            width,
                            luma_height,
                            chroma_height,
                            range: if pixel_format == Some(CVPixelFormat::F420) { VideoRange::Full } else { VideoRange::Video }
                        }))
                    },
                    _ => Err(VideoFrameBitmapError::Other("Unknown pixel format on iosurface".to_string()))
                }
            } else {
                Err(VideoFrameBitmapError::Other("Failed to lock iosurface".to_string()))
            }
        }
    }
}

