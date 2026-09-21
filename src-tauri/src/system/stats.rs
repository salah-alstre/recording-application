//! Real system statistics. Nothing here is estimated or faked: values the platform cannot report
//! (for example GPU load on non-NVIDIA GPUs) are `None` and the UI shows "n/a".
use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_pct: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub gpu_pct: Option<u32>,
    pub gpu_temp_c: Option<u32>,
    pub gpu_mem_used: Option<u64>,
    pub gpu_mem_total: Option<u64>,
    pub encoder_pct: Option<u32>,
    /// Working set of the Rimlight process itself.
    pub app_mem: u64,
    /// CPU used by Rimlight + its ffmpeg encoders, in percent of the whole machine.
    pub app_cpu_pct: f32,
}

pub struct Sampler {
    sys: System,
    nvml: Option<nvml_wrapper::Nvml>,
    cores: f32,
}

impl Sampler {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        let cores = sys.cpus().len().max(1) as f32;
        let nvml = nvml_wrapper::Nvml::init().ok();
        if nvml.is_none() {
            tracing::debug!("NVML unavailable; GPU metrics will be reported as n/a");
        }
        Self { sys, nvml, cores }
    }

    /// `extra_pids` are child processes (ffmpeg encoders) attributed to the app.
    pub fn sample(&mut self, extra_pids: &[u32]) -> SystemStats {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        let mut pids: Vec<Pid> = vec![Pid::from_u32(std::process::id())];
        pids.extend(extra_pids.iter().map(|p| Pid::from_u32(*p)));
        self.sys
            .refresh_processes(ProcessesToUpdate::Some(&pids), true);
        let mut app_cpu = 0.0;
        let mut app_mem = 0;
        for (i, p) in pids.iter().enumerate() {
            if let Some(proc_) = self.sys.process(*p) {
                app_cpu += proc_.cpu_usage();
                if i == 0 {
                    app_mem = proc_.memory();
                }
            }
        }
        let mut s = SystemStats {
            cpu_pct: self.sys.global_cpu_usage(),
            ram_used: self.sys.used_memory(),
            ram_total: self.sys.total_memory(),
            app_mem,
            app_cpu_pct: app_cpu / self.cores,
            ..Default::default()
        };
        if let Some(n) = &self.nvml {
            if let Ok(d) = n.device_by_index(0) {
                if let Ok(u) = d.utilization_rates() {
                    s.gpu_pct = Some(u.gpu);
                }
                s.gpu_temp_c = d
                    .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .ok();
                if let Ok(m) = d.memory_info() {
                    s.gpu_mem_used = Some(m.used);
                    s.gpu_mem_total = Some(m.total);
                }
                s.encoder_pct = d.encoder_utilization().ok().map(|e| e.utilization);
            }
        }
        s
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampler_reports_real_memory() {
        let mut s = Sampler::new();
        let st = s.sample(&[]);
        assert!(st.ram_total > 0);
        assert!(st.app_mem > 0);
    }
}
