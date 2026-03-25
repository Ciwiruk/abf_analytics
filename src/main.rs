#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use std::path::Path;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "abf_analytics",
        options,
        Box::new(|_cc| Ok(Box::new(AbfAnalyticsApp::default()))),
    )
}

#[derive(Default)]
struct AbfAnalyticsApp;

impl eframe::App for AbfAnalyticsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Test");
            ui.label("This is a basic egui app running with eframe.");
        });
    }
}
