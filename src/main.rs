use eframe::egui::{CentralPanel, Context};
use eframe::{egui, Frame};
use once_cell::sync::Lazy;
use regex::Regex;

const BUTTON_SIZE: f32 = 60.0;
const DECIMAL_DIGITS: usize = 5;

// Font size step values: step to smaller size if too many digits
const STEP_34: usize = 12;
const STEP_26: usize = 16;
const STEP_18: usize = 24;
const STEP_10: usize = 43;

// ChatGPT, slightly modified --------------------
// Regexes for validity checks
// Each number can optionally have a + or - qualifier

// Matches a potentially partial single number, optionally signed, with optional decimal
static RE_STEP_ONE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[+-]?\d*(\.\d*)?$").unwrap());
// Matches a number followed by an operator (incomplete expression)
static RE_STEP_TWO: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[+-]?\d+(\.\d+)?[+\-x/^%]$").unwrap());
// Matches a number, operator, and optional sign for the next number
static RE_STEP_THREE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[+-]?\d+(\.\d+)?[+\-x/^%][+-]?$").unwrap());
// Matches a number, operator, optional sign, and partial second number
static RE_STEP_FOUR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[+-]?\d+(\.\d+)?[+\-x/^%][+-]?\d*(\.\d*)?$").unwrap());
// Matches a full valid expression with two numbers and an operator, capturing all parts
static RE_FULL: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([+-]?\d+(?:\.\d+)?)([+\-x/^%])([+-]?\d+(?:\.\d+)?)$").unwrap());
// Matches a single, complete number
static RE_SINGLE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[+-]?\d+(\.\d+)?$").unwrap());
// Matches an operator followed by more than one leading zero before a digit
static RE_INVALID_LEADING_ZERO_AFTER_OP: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[+\-x/^%]0\d").unwrap());

fn is_operator(c: char) -> bool {
    matches!(c, '+' | '-' | 'x' | '/' | '^' | '%')
}

fn is_unary_sign(c: char) -> bool {
    matches!(c, '+' | '-')
}
// End ChatGPT --------------------

fn build_push_button(ui: &mut egui::Ui, label: &str, calc_str: &mut String) {
    if ui
        .add_sized([BUTTON_SIZE, BUTTON_SIZE], egui::Button::new(label))
        .clicked()
    {
        let mut label_copy = label; // Make a mutable clone

        // Edge cases
        if label == "." {
            // if last char is operator, replace "." with "0."
            if !calc_str
                .chars()
                .last()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                label_copy = "0.";
            }
        }

        if calc_str == "0" {
            match label {
                "+" | "-" | "x" | "/" | "^" | "%" | "." => {
                    calc_str.push_str(label);
                    return;
                }
                _ => {
                    calc_str.clear();
                    calc_str.push_str(label);
                    return;
                }
            }
        }

        // ChatGPT, somewhat modified --------------------
        // Operator replacement logic with unary exceptions
        if let Some(new_op) = label.chars().next() {
            if is_operator(new_op) {
                let chars: Vec<char> = calc_str.chars().collect();
                let len = chars.len();

                if len >= 1 && is_operator(chars[len - 1]) {
                    let last = chars[len - 1];

                    // leading unary at start ("-", "+")
                    if len == 1 && is_unary_sign(last) {
                        if is_unary_sign(new_op) {
                            calc_str.pop();
                            calc_str.push(new_op);
                        }
                        return;
                    }

                    // unary after active operator ("5+-", "5x-")
                    if len >= 2 && is_operator(chars[len - 2]) && is_unary_sign(last) {
                        if is_unary_sign(new_op) {
                            calc_str.pop();
                            calc_str.push(new_op);
                        }
                        return;
                    }

                    // active binary operator ("5+", "9x")
                    if !is_unary_sign(new_op) {
                        calc_str.pop();
                    }
                    calc_str.push(new_op);
                    return;
                }
            }
        }
        // End ChatGPT --------------------

        let temp = format!("{}{}", calc_str, label_copy);

        // Check for invalid leading 0 after decimal
        // Only catches attempted integer after 0
        match temp.as_str() {
            s if RE_INVALID_LEADING_ZERO_AFTER_OP.is_match(s) => {
                calc_str.pop();
                calc_str.push_str(label_copy);
                return;
            }
            _ => {}
        }

        let can_push = match temp.as_str() {
            s if RE_STEP_ONE.is_match(s) => true,
            s if RE_STEP_TWO.is_match(s) => true,
            s if RE_STEP_THREE.is_match(s) => true,
            s if RE_STEP_FOUR.is_match(s) => true,
            s if RE_FULL.is_match(s) => true,
            _ => false,
        };
        if can_push {
            calc_str.push_str(label_copy);
        }
    }
}

fn build_clear_button(ui: &mut egui::Ui, calc_str: &mut String) {
    if ui
        .add_sized([BUTTON_SIZE, BUTTON_SIZE], egui::Button::new("AC"))
        .clicked()
    {
        calc_str.clear();
    };
}

fn build_backspace_button(ui: &mut egui::Ui, calc_str: &mut String) {
    if ui
        .add_sized([BUTTON_SIZE, BUTTON_SIZE], egui::Button::new("<-"))
        .clicked()
    {
        calc_str.pop();
    };
}

fn build_equals_button(ui: &mut egui::Ui, calc_str: &mut String) {
    if ui
        .add_sized([BUTTON_SIZE, BUTTON_SIZE], egui::Button::new("="))
        .clicked()
    {
        // Must clear and then push, reallocation not allowed
        let result: String = process_calculation(calc_str.as_str());
        calc_str.clear();
        calc_str.push_str(&result);
    };
}

fn fmt_f64(val: f64) -> String {
    let s = format!("{:.*}", DECIMAL_DIGITS, val);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn process_calculation(calc_str: &str) -> String {
    // If expression is a single value, return unchanged
    if RE_SINGLE_NUMBER.is_match(calc_str) {
        return fmt_f64(calc_str.to_string().parse().unwrap());
    }

    // Use regex capture groups to extract left number, operator, and right number
    if let Some(caps) = RE_FULL.captures(calc_str) {
        let left_number: f64 = caps.get(1).unwrap().as_str().parse().unwrap();
        let operator: char = caps.get(2).unwrap().as_str().chars().next().unwrap();
        let right_number: f64 = caps.get(3).unwrap().as_str().parse().unwrap();

        return match operator {
            '+' => fmt_f64(left_number + right_number),
            '-' => fmt_f64(left_number - right_number),
            'x' => fmt_f64(left_number * right_number),
            '/' => fmt_f64(left_number / right_number),
            '^' => fmt_f64(left_number.powf(right_number)),
            '%' => fmt_f64(left_number % right_number),
            _ => "Error".to_string(), // Should be unreachable
        };
    }

    "Invalid".to_string()
}

#[derive(Default)]
struct App {
    calculation_string: String,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(24.0, egui::FontFamily::Proportional),
        );

        // Dynamically shrink font size
        let label_len = self.calculation_string.len();
        let label_font_size = match label_len {
            label_len if label_len <= STEP_34 => 34.0,
            label_len if label_len <= STEP_26 => 26.0,
            label_len if label_len <= STEP_18 => 18.0,
            _ => 10.0,
        };
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(label_font_size, egui::FontFamily::Monospace),
        );

        ctx.set_style(style);

        if self.calculation_string.len() > STEP_10 {
            self.calculation_string = "Length Overflow".to_string();
        }

        CentralPanel::default().show(ctx, |ui| {
            // Top calculation string
            match label_font_size {
                34.0 => ui.add_space(2.5),
                26.0 => ui.add_space(11.3),
                18.0 => ui.add_space(20.8),
                _ => ui.add_space(29.7),
            }

            let display_text = if self.calculation_string.is_empty() {
                "—"
            } else {
                &self.calculation_string
            };
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.strong(display_text);
            });
            ui.add_space(10.0);

            // Buttons
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    build_clear_button(ui, &mut self.calculation_string);
                    build_push_button(ui, "7", &mut self.calculation_string);
                    build_push_button(ui, "4", &mut self.calculation_string);
                    build_push_button(ui, "1", &mut self.calculation_string);
                    build_backspace_button(ui, &mut self.calculation_string);
                });
                ui.vertical(|ui| {
                    build_push_button(ui, "%", &mut self.calculation_string);
                    build_push_button(ui, "8", &mut self.calculation_string);
                    build_push_button(ui, "5", &mut self.calculation_string);
                    build_push_button(ui, "2", &mut self.calculation_string);
                    build_push_button(ui, "0", &mut self.calculation_string);
                });
                ui.vertical(|ui| {
                    build_push_button(ui, "^", &mut self.calculation_string);
                    build_push_button(ui, "9", &mut self.calculation_string);
                    build_push_button(ui, "6", &mut self.calculation_string);
                    build_push_button(ui, "3", &mut self.calculation_string);
                    build_push_button(ui, ".", &mut self.calculation_string);
                });
                ui.vertical(|ui| {
                    build_push_button(ui, "/", &mut self.calculation_string);
                    build_push_button(ui, "x", &mut self.calculation_string);
                    build_push_button(ui, "-", &mut self.calculation_string);
                    build_push_button(ui, "+", &mut self.calculation_string);
                    build_equals_button(ui, &mut self.calculation_string);
                });
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([280.0, 382.5]),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_ctx| Ok(Box::<App>::default())),
    )
}
