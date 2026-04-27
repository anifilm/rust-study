#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod calc_engine;

use app::CalculatorApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([320.0, 480.0])
            .with_resizable(false)
            .with_maximize_button(false),
        ..Default::default()
    };

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(eframe::egui::Visuals::dark());

            // Segoe UI Symbol을 fallback 폰트로 추가 (⌫ 등 기호 지원)
            let mut fonts = eframe::egui::FontDefinitions::default();
            if let Ok(font_data) = std::fs::read("C:/Windows/Fonts/seguisym.ttf") {
                fonts.font_data.insert(
                    "segoe_symbol".to_owned(),
                    eframe::egui::FontData::from_owned(font_data).into(),
                );
                fonts
                    .families
                    .get_mut(&eframe::egui::FontFamily::Proportional)
                    .unwrap()
                    .push("segoe_symbol".to_owned());
                fonts
                    .families
                    .get_mut(&eframe::egui::FontFamily::Monospace)
                    .unwrap()
                    .push("segoe_symbol".to_owned());
            }
            cc.egui_ctx.set_fonts(fonts);

            Ok(Box::new(CalculatorApp::new()))
        }),
    )
}
