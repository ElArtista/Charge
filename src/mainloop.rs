use time::PreciseTime;

pub struct MainLoop<'a, T: 'a> {
    update_cb: Box<Fn(&mut T, f32) -> bool>,
    render_cb: Box<Fn(&T, f32)>,
    perf_cb: Option<Box<Fn(&T, f32, f32)>>,
    should_terminate: bool,
    updates_per_second: u32,
    perf_refr_rate: f32,
    perf_samples_sum: f32,
    perf_samples_cnt: u32,
    userdata: &'a mut T,
}

impl<'a, T> MainLoop<'a, T> {
    pub fn new(
        update_cb: Box<Fn(&mut T, f32) -> bool>,
        render_cb: Box<Fn(&T, f32)>,
        perf_cb: Option<Box<Fn(&T, f32, f32)>>,
        userdata: &'a mut T,
    ) -> MainLoop<'a, T> {
        MainLoop {
            update_cb: update_cb,
            render_cb: render_cb,
            perf_cb: perf_cb,
            should_terminate: false,
            updates_per_second: 60,
            perf_refr_rate: 0.5,
            perf_samples_sum: 0.0,
            perf_samples_cnt: 0,
            userdata: userdata,
        }
    }

    pub fn run(&mut self) {
        let ms_per_update: f32 = 1000.0 / (self.updates_per_second as f32);

        let mut lag = 0.0;
        let mut previous = PreciseTime::now();
        while !self.should_terminate {
            let current = PreciseTime::now();
            let elapsed = match previous.to(current).num_nanoseconds() {
                Some(elapsed) => elapsed as f32 / 1000000.0,
                None => 0.0,
            };
            previous = current;
            lag += elapsed;

            while lag > ms_per_update {
                self.should_terminate =
                    (self.update_cb)(&mut self.userdata, ms_per_update / 1000.0);
                lag -= ms_per_update;
            }

            let interpolation = lag / ms_per_update;
            (self.render_cb)(&mut self.userdata, interpolation);

            if let Some(perf_cb) = &self.perf_cb {
                self.perf_samples_sum += elapsed;
                self.perf_samples_cnt += 1;
                if self.perf_samples_sum / 1000.0 >= self.perf_refr_rate {
                    let avgms = self.perf_samples_sum / self.perf_samples_cnt as f32;
                    perf_cb(&mut self.userdata, avgms, 1000.0 / avgms);
                    self.perf_samples_sum = 0.0;
                    self.perf_samples_cnt = 0;
                }
            }
        }
    }
}
