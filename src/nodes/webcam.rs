use core::result::Result::Ok;
use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Output,
    node::{ChangeObserver, Node, ReadyError, SendError, UpdateError},
};
use image::DynamicImage;
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable)]
pub struct WebcamNode {
    camera: Option<Camera>,
    #[output]
    pub output: Output<DynamicImage>,
}

impl WebcamNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            camera: None,
            output: Output::new(change_observer),
        }
    }
}

impl Node for WebcamNode {
    fn on_ready(&self) -> Result<(), ReadyError> {
        let index = CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let camera = Camera::new(index, requested);

        match camera {
            Ok(mut cam) => {
                let test_frame = cam.frame();
                match test_frame {
                    Err(err) => Err(ReadyError::Other(err.into())),
                    Ok(_) => {
                        self.camera = Some(cam); // Wrap Camera in Arc<Mutex>
                        Ok(())
                    }
                }
            }
            Err(err) => Err(ReadyError::Other(err.into())),
        }
    }

    fn on_update(&mut self) -> Result<(), UpdateError> {
        let no_cam_err = "No camera available";
        match &self.camera {
            None => Err(UpdateError::SendError {
                message: no_cam_err.to_string(),
            }),
            Some(mut cam) => {
                let r_frame = cam.frame();
                match r_frame {
                    Err(err) => Err(UpdateError::Other(err.into())),
                    Ok(frame) => {
                        let r_decode = frame.decode_image::<RgbFormat>();
                        match r_decode {
                            Err(err) => Err(UpdateError::Other(err.into())),
                            Ok(decoded) => {
                                let dyn_img = DynamicImage::ImageRgb8(decoded);
                                self.output
                                    .clone()
                                    .send(dyn_img)
                                    .map_err(|e| UpdateError::Other(e.into()))?;
                                Ok(())
                            }
                        }
                    }
                }
            }
        }
    }
}