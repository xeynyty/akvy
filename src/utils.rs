pub mod response {
    pub struct ResponseTime {
        average: u32,
        count: u32,
        min: u32,
        max: u32
    }
    impl ResponseTime {
        pub const fn new() -> Self {
            Self {
                average: 0,
                count: 0,
                min: 999_999_999,
                max: 0
            }
        }

        pub fn get_average(&self) -> u32 {
            self.average
        }
        pub fn get_count(&self) -> u32 {
            self.count
        }
        pub fn get_min(&self) -> u32 {
            self.min
        }
        pub fn get_max(&self) -> u32 {
            self.max
        }

        pub fn add(&mut self, new: u32) {
            self.average = (self.average * self.count + new) / (self.count + 1);
            self.count += 1;
            self.min_check(new);
            self.max_check(new);
        }

        fn min_check(&mut self, item: u32) {
            self.min = self.min.min(item);
        }

        fn max_check(&mut self, item: u32) {
            self.max = self.max.max(item);
        }
    }
}
