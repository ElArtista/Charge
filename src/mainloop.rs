use time::PreciseTime;

fn timeit<T, F: FnOnce() -> T>(f: F) -> (f32, T) {
    let start = PreciseTime::now();
    let r = f();
    let elapsed = start.to(PreciseTime::now()).num_nanoseconds().unwrap_or(0) as f32 / 1_000_000.0;
    (elapsed, r)
}

pub struct MainLoop<'a, T: 'a> {
    update_cb: Box<Fn(&mut T, f32) -> bool>,
    render_cb: Box<Fn(&T, f32)>,
    perf_cb: Option<Box<Fn(&mut T, f32, f32, f32)>>,
    should_terminate: bool,
    updates_per_second: u32,
    perf_refr_rate: f32,
    perf_samples_acc: [f32; 3], // Total, Update, Render
    perf_samples_cnt: u32,
    userdata: &'a mut T,
}

impl<'a, T> MainLoop<'a, T> {
    pub fn new(
        update_cb: Box<Fn(&mut T, f32) -> bool>,
        render_cb: Box<Fn(&T, f32)>,
        perf_cb: Option<Box<Fn(&mut T, f32, f32, f32)>>,
        userdata: &'a mut T,
    ) -> MainLoop<'a, T> {
        MainLoop {
            update_cb: update_cb,
            render_cb: render_cb,
            perf_cb: perf_cb,
            should_terminate: false,
            updates_per_second: 60,
            perf_refr_rate: 0.5,
            perf_samples_acc: [0.0; 3],
            perf_samples_cnt: 0,
            userdata: userdata,
        }
    }

    pub fn run(&mut self) {
        let ms_per_update: f32 = 1000.0 / (self.updates_per_second as f32);

        let mut lag = 0.0;
        let mut elapsed = 0.0;
        while !self.should_terminate {
            elapsed = timeit(|| {
                let mut update_time = 0.0;
                while lag > ms_per_update {
                    let dt = ms_per_update / 1000.0;
                    let rt = timeit(|| (self.update_cb)(&mut self.userdata, dt));
                    update_time = rt.0;
                    self.should_terminate = rt.1;
                    lag -= ms_per_update;
                }

                let interpolation = lag / ms_per_update;
                let (render_time, _) = timeit(|| (self.render_cb)(&self.userdata, interpolation));

                if let Some(perf_cb) = &self.perf_cb {
                    self.perf_samples_acc
                        .iter_mut()
                        .zip(&[elapsed, update_time, render_time])
                        .for_each(|(a, b)| *a += b);
                    self.perf_samples_cnt += 1;
                    if self.perf_samples_acc[0] >= self.perf_refr_rate * 1000.0 {
                        let nsamples = self.perf_samples_cnt;
                        self.perf_samples_acc
                            .iter_mut()
                            .for_each(|x| *x /= nsamples as f32);
                        let avgms = self.perf_samples_acc;
                        perf_cb(&mut self.userdata, avgms[0], avgms[1], avgms[2]);
                        self.perf_samples_acc = [0.0; 3];
                        self.perf_samples_cnt = 0;
                    }
                }
            }).0;
            lag += elapsed;
        }
    }
}
