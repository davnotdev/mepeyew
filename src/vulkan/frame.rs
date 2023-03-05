pub struct VkFrameDependent<T> {
    d: Vec<T>,
}

impl<T> VkFrameDependent<T> {
    pub fn get(&self, frame: &VkFrame) -> &T {
        &self.d[frame.get_current_frame()]
    }

    pub fn take_all(self) -> Vec<T> {
        self.d
    }
}

impl<T> FromIterator<T> for VkFrameDependent<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        VkFrameDependent {
            d: iter.into_iter().collect(),
        }
    }
}

pub struct VkFrame {
    flight_frames: usize,
    current_frame: usize,
}

impl VkFrame {
    pub fn new(flight_frames: usize) -> Self {
        VkFrame {
            flight_frames,
            current_frame: 0,
        }
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn get_flight_frames_count(&self) -> usize {
        self.flight_frames
    }

    pub fn advance_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.flight_frames
    }
}
