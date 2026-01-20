use std::time::{Duration, Instant};
use image::DynamicImage;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

/// Handles sprite sheet animation state and frame management
pub struct Animation {
    frames: Vec<StatefulProtocol>,
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
        let frames = Self::extract_frames(image, frame_count, frame_size, picker);

        if frames.is_empty() {
            return Err("No frames extracted from sprite sheet".into());
        }

        Ok(Self {
            frames,
            current_frame: 0,
            last_frame_time: Instant::now(),
            frame_duration: Duration::from_millis(100),
        })
    }

    /// Extract individual frames from a sprite sheet and convert to protocols
    fn extract_frames(
        image: &DynamicImage,
        frame_count: usize,
        frame_size: [u32; 2],
        picker: &Picker,
    ) -> Vec<StatefulProtocol> {
        let mut frames = Vec::new();
        let [frame_width, frame_height] = frame_size;

        for frame_idx in 0..frame_count {
            let x = (frame_idx as u32) * frame_width;
            let y = 0; // Assuming horizontal sprite sheet

            // Extract frame
            let frame = image.crop_imm(x, y, frame_width, frame_height);

            // Convert to protocol for rendering
            let protocol = picker.new_resize_protocol(frame);
            frames.push(protocol);
        }

        frames
    }

    /// Update animation state, advancing frame if needed
    pub fn update(&mut self) {
        if self.frames.is_empty() {
            return;
        }

        let now = Instant::now();
        if now.duration_since(self.last_frame_time) >= self.frame_duration {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_frame_time = now;
        }
    }

    /// Get the current frame protocol for rendering
    pub fn current_frame(&mut self) -> Option<&mut StatefulProtocol> {
        self.frames.get_mut(self.current_frame)
    }

    /// Get the current frame index (0-based)
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Get total number of frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Set animation speed (frames per second)
    pub fn set_fps(&mut self, fps: u32) {
        self.frame_duration = Duration::from_millis(1000 / fps.max(1) as u64);
    }
}