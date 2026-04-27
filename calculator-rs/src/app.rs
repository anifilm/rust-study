use eframe::egui;

use crate::calc_engine::{CalcEngine, Op};

const DIGIT_COLOR: egui::Color32 = egui::Color32::from_rgb(58, 58, 58);
const OPERATOR_COLOR: egui::Color32 = egui::Color32::from_rgb(72, 72, 72);
const FUNC_COLOR: egui::Color32 = egui::Color32::from_rgb(46, 46, 46);
const EQUALS_COLOR: egui::Color32 = egui::Color32::from_rgb(65, 95, 125);
const TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(230, 230, 230);
const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(160, 160, 160);

pub struct CalculatorApp {
    engine: CalcEngine,
}

impl CalculatorApp {
    pub fn new() -> Self {
        Self {
            engine: CalcEngine::new(),
        }
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => match key {
                        egui::Key::Num0
                        | egui::Key::Num1
                        | egui::Key::Num2
                        | egui::Key::Num3
                        | egui::Key::Num4
                        | egui::Key::Num5
                        | egui::Key::Num6
                        | egui::Key::Num7
                        | egui::Key::Num8
                        | egui::Key::Num9
                            if !modifiers.shift =>
                        {
                            let d = *key as u8 - egui::Key::Num0 as u8;
                            self.engine.press_digit(d);
                        }
                        egui::Key::Enter => self.engine.press_equals(),
                        egui::Key::Escape => self.engine.press_clear(),
                        egui::Key::Backspace => self.engine.press_backspace(),
                        _ => {}
                    },
                    egui::Event::Text(text) => match text.as_str() {
                        "+" => self.engine.press_operator(Op::Add),
                        "-" => self.engine.press_operator(Op::Sub),
                        "*" => self.engine.press_operator(Op::Mul),
                        "/" => self.engine.press_operator(Op::Div),
                        "=" => self.engine.press_equals(),
                        _ => {}
                    },
                    _ => {}
                }
            }
        });
    }
}

impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);

        // Display area (top panel with fixed height)
        egui::TopBottomPanel::top("display")
            .exact_height(100.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(28, 28, 28))
                    .inner_margin(egui::Margin { left: 12, right: 12, top: 20, bottom: 8 }),
            )
            .show(ctx, |ui| {
                // Main display (large, right-aligned, auto-shrink)
                let display_str = self.engine.display();
                let available_width = ui.available_width();
                let mut font_size = 36.0_f32;
                let min_font_size = 14.0_f32;

                // 텍스트 너비를 측정하여 영역에 맞을 때까지 축소
                while font_size > min_font_size {
                    let galley = ui.fonts(|f| {
                        f.layout_no_wrap(
                            display_str.clone(),
                            egui::FontId::proportional(font_size),
                            TEXT_COLOR,
                        )
                    });
                    if galley.rect.width() <= available_width {
                        break;
                    }
                    font_size -= 2.0;
                }

                let display_text = egui::RichText::new(display_str)
                    .size(font_size)
                    .color(TEXT_COLOR);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.label(display_text);
                });

                // Expression (small, right-aligned, below the number)
                let expr = self.engine.expression();
                let expr_text = egui::RichText::new(if expr.is_empty() { " " } else { expr })
                    .size(14.0)
                    .color(TEXT_DIM);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                    ui.label(expr_text);
                });
            });

        // Button grid (central panel)
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(4.0, 4.0);

            let available = ui.available_size();
            let btn_width = ((available.x - 12.0) / 4.0).max(1.0);
            let btn_height = ((available.y - 16.0) / 5.0).max(1.0);
            let btn_size = [btn_width, btn_height];

            let button = |ui: &mut egui::Ui,
                          label: &str,
                          color: egui::Color32,
                          text_color: egui::Color32,
                          size: [f32; 2]|
             -> bool {
                let btn = egui::Button::new(
                    egui::RichText::new(label).size(20.0).color(text_color),
                )
                .fill(color)
                .corner_radius(6.0);
                ui.add_sized(size, btn).clicked()
            };

            // Row 1: CE, C, ←, ÷
            ui.horizontal(|ui| {
                if button(ui, "%", FUNC_COLOR, TEXT_DIM, btn_size) {
                    self.engine.press_percent();
                }
                if button(ui, "C", FUNC_COLOR, TEXT_DIM, btn_size) {
                    self.engine.press_clear();
                }
                if button(ui, "⌫", FUNC_COLOR, TEXT_DIM, btn_size) {
                    self.engine.press_backspace();
                }
                if button(ui, "÷", OPERATOR_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_operator(Op::Div);
                }
            });

            // Row 2: 7, 8, 9, ×
            ui.horizontal(|ui| {
                for d in [7, 8, 9] {
                    if button(ui, &d.to_string(), DIGIT_COLOR, TEXT_COLOR, btn_size) {
                        self.engine.press_digit(d);
                    }
                }
                if button(ui, "×", OPERATOR_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_operator(Op::Mul);
                }
            });

            // Row 3: 4, 5, 6, −
            ui.horizontal(|ui| {
                for d in [4, 5, 6] {
                    if button(ui, &d.to_string(), DIGIT_COLOR, TEXT_COLOR, btn_size) {
                        self.engine.press_digit(d);
                    }
                }
                if button(ui, "−", OPERATOR_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_operator(Op::Sub);
                }
            });

            // Row 4: 1, 2, 3, +
            ui.horizontal(|ui| {
                for d in [1, 2, 3] {
                    if button(ui, &d.to_string(), DIGIT_COLOR, TEXT_COLOR, btn_size) {
                        self.engine.press_digit(d);
                    }
                }
                if button(ui, "+", OPERATOR_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_operator(Op::Add);
                }
            });

            // Row 5: ±, 0, ., =
            ui.horizontal(|ui| {
                if button(ui, "±", DIGIT_COLOR, TEXT_DIM, btn_size) {
                    self.engine.press_negate();
                }
                if button(ui, "0", DIGIT_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_digit(0);
                }
                if button(ui, ".", DIGIT_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_decimal();
                }
                if button(ui, "=", EQUALS_COLOR, TEXT_COLOR, btn_size) {
                    self.engine.press_equals();
                }
            });
        });
    }
}
