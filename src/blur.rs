
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BlurMethod {
    CPU,
    GPU,
}

pub fn do_blur(image: &mut blur::Image, method: BlurMethod) -> gtk::gdk::MemoryTexture {
    let rowstride = (image.width * 4) as usize; // bytes per row (width * 4 bytes for RGBA)

    match method {
        BlurMethod::GPU => {
            #[cfg(feature = "gpu")]
            {
                use futures::executor::block_on;
                use image::Rgba;
                let filters = futures::executor::block_on(blur::Filters::new());
                let operation = image
                    .operation(&filters)
                    .gaussian_blur(16.0)
                    .gaussian_blur(16.0)
                    .gaussian_blur(16.0);
                let result = block_on(operation.execute());

                match image::ImageBuffer::<Rgba<u8>, _>::from_raw(
                    result.width,
                    result.height,
                    result.as_raw(),
                ) {
                    Some(buffer) => gtk::gdk::MemoryTexture::new(
                        buffer.width() as i32,
                        buffer.height() as i32,
                        gtk::gdk::MemoryFormat::R8g8b8a8,
                        &gtk::glib::Bytes::from_owned(bytemuck::cast_vec(buffer.to_vec())),
                        rowstride,
                    ),
                    None => {
                        do_blur(image, BlurMethod::CPU)
                    },
                }
            }
            #[cfg(not(feature = "gpu"))]
            {
                crate::log::warning!("GPU blur method is not supported, fallback to CPU");
                do_blur(image, BlurMethod::CPU)
            }
        }
        BlurMethod::CPU => {
            {
                use libblur::{
                    AnisotropicRadius,
                    BlurImageMut,
                    EdgeMode,
                    EdgeMode2D,
                    FastBlurChannels,
                    ThreadingPolicy,
                };

                // TODO...
                let data: &mut _ = unsafe { std::mem::transmute(image.as_mut_raw()) };
                let mut blur = BlurImageMut::borrow(
                    data,
                    image.width,
                    image.height,
                    FastBlurChannels::Channels4,
                );

                //stack_blur(&mut blur, AnisotropicRadius::new(32), ThreadingPolicy::Adaptive);
                //libblur::fast_gaussian(&mut blur, AnisotropicRadius::new(8), ThreadingPolicy::Adaptive, EdgeMode2D::new(EdgeMode::Clamp));

                for _ in 0..3 {
                    libblur::fast_gaussian_next(
                        &mut blur,
                        AnisotropicRadius::new(24),
                        ThreadingPolicy::Adaptive,
                        EdgeMode2D::new(EdgeMode::Reflect),
                    );
                }

                gtk::gdk::MemoryTexture::new(
                    image.width as i32,
                    image.height as i32,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &gtk::glib::Bytes::from_static(data),
                    rowstride,
                )
            }
        }
    }
}
