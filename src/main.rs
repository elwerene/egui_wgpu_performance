#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew};
use egui::{Id, Slider, ViewportId};
use std::{
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
    time::Duration,
};
use wgpu::{PowerPreference, PresentMode};

static FPS: AtomicUsize = AtomicUsize::new(0);
static FPS_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn main() -> eframe::Result {
    start_update_fps_thread();

    let mut wgpu_options = WgpuConfiguration::default();
    wgpu_options.present_mode = PresentMode::AutoNoVsync; // We do not care about vsync as we have our own framerate limiter
    wgpu_options.wgpu_setup = match wgpu_options.wgpu_setup {
        WgpuSetup::CreateNew(create_new) => WgpuSetup::CreateNew(WgpuSetupCreateNew {
            power_preference: PowerPreference::HighPerformance,
            ..create_new
        }),
        _ => unreachable!(),
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        vsync: false,
        wgpu_options,
        ..Default::default()
    };
    eframe::run_native(
        "egui_wgpu_performance",
        native_options,
        Box::new(|_cc| Ok(Box::new(TemplateApp::default()))),
    )
}

#[derive(Default)]
pub struct TemplateApp {
    viewport_ids: Vec<egui::ViewportId>,
}

impl eframe::App for TemplateApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "{} fps",
                FPS.load(std::sync::atomic::Ordering::Relaxed)
            ));

            let mut number_of_windows = self.viewport_ids.len();
            if ui
                .add(Slider::new(&mut number_of_windows, 0..=10).text("Number of windows"))
                .changed()
            {
                self.viewport_ids = (0..number_of_windows)
                    .map(|i| ViewportId(Id::new(format!("w{i}"))))
                    .collect();
            }
        });

        for viewport_id in &self.viewport_ids {
            ctx.show_viewport_immediate(
                *viewport_id,
                egui::ViewportBuilder::default()
                    .with_inner_size([400.0, 300.0])
                    .with_min_inner_size([300.0, 220.0]),
                |ctx, _viewport_class| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Extra Window");
                    });
                },
            );
        }

        ctx.request_repaint();
        FPS_COUNTER.fetch_add(1, Relaxed);
    }
}

fn start_update_fps_thread() {
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let fps = FPS_COUNTER.swap(0, Relaxed);
            FPS.store(fps, Relaxed);
        }
    });
}
