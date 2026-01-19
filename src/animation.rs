use std::time::{Duration, Instant};
use image::DynamicImage;
use ratatui_image::picker::Picker;

/// Handles sprite sheet animation state and frame management
pub struct Animation {
    frames: Vec<DynamicImage>,
    current_frame: usize,
    last_frame_time: Instant,
    frame_duration: Duration,
}

impl Animation {
    /// Create a new animation from a sprite sheet
    pub fn from_sprite_sheet(
        image: &DynamicImage,
        frame_count: usize,
        frame_size: [u32; 2],
        picker: &Picker,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let frames = Self::extract_frames(image, frame_count, frame_size);

        let mut animation = Self {
            frames,
            current_frame: 0,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_millis(100),
        };

        Ok(animation)
    }

    /// Extract individual frames from a sprite sheet
    fn extract_frames(
        image: &DynamicImage,
        frame_count: usize,
        frame_size: [u32; 2],
    ) -> Vec<DynamicImage> {
        let mut frames = Vec::new();
        let [frame_width, frame_height] = frame_size;

        for frame_idx in 0..frame_count {
            let frame_col = (frame_idx as u32);
            let frame_row = (frame_idx as u32);

            let x = frame_col * frame_width;
            let y = frame_row * frame_height;

            let frame = image.crop_imm(x, y, frame_width, frame_height);
            frames.push(frame);
        }

        frames
    }

    /// Update animation state, advancing frame if needed
    pub fn update(&mut self, picker: &Picker) -> Result<(), Box<dyn std::error::Error>> {
        if self.frames.is_empty() {
            return Ok(());
        }

        let now = Instant::now();
        if now.duration_since(self.last_frame_time) >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_frame_time = now;
        }

        Ok(())
    }

    /// Get the current frame index (0-based)
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Get total number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}