#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn symbol(&self) -> &str {
        match self {
            Op::Add => "+",
            Op::Sub => "−",
            Op::Mul => "×",
            Op::Div => "÷",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CalcState {
    InputFirst,
    InputSecond,
    ShowResult,
}

pub struct CalcEngine {
    display: String,
    first_operand: f64,
    operator: Option<Op>,
    state: CalcState,
    has_decimal: bool,
    error: bool,
    waiting_for_second: bool,
    expression: String,
    last_operator: Option<Op>,
    last_second_operand: f64,
}

impl CalcEngine {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            first_operand: 0.0,
            operator: None,
            state: CalcState::InputFirst,
            has_decimal: false,
            error: false,
            waiting_for_second: false,
            expression: String::new(),
            last_operator: None,
            last_second_operand: 0.0,
        }
    }

    pub fn display(&self) -> String {
        add_commas(&self.display)
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn press_digit(&mut self, d: u8) {
        if self.error {
            return;
        }
        if d > 9 {
            return;
        }

        if self.state == CalcState::ShowResult {
            self.display = d.to_string();
            self.has_decimal = false;
            self.state = CalcState::InputFirst;
            self.operator = None;
            self.waiting_for_second = false;
            self.expression.clear();
            return;
        }

        if self.waiting_for_second {
            self.display = d.to_string();
            self.has_decimal = false;
            self.waiting_for_second = false;
            return;
        }

        if self.display == "0" {
            self.display = d.to_string();
        } else {
            self.display.push_str(&d.to_string());
        }
    }

    pub fn press_decimal(&mut self) {
        if self.error {
            return;
        }

        if self.state == CalcState::ShowResult {
            self.display = "0.".to_string();
            self.has_decimal = true;
            self.state = CalcState::InputFirst;
            self.operator = None;
            self.waiting_for_second = false;
            return;
        }

        if self.waiting_for_second {
            self.display = "0.".to_string();
            self.has_decimal = true;
            self.waiting_for_second = false;
            return;
        }

        if !self.has_decimal {
            self.display.push('.');
            self.has_decimal = true;
        }
    }

    pub fn press_operator(&mut self, op: Op) {
        if self.error {
            return;
        }

        match self.state {
            CalcState::InputFirst => {
                self.first_operand = self.parse_display();
                self.expression = format!("{} {}", add_commas(&self.display), op.symbol());
                self.operator = Some(op);
                self.state = CalcState::InputSecond;
                self.waiting_for_second = true;
            }
            CalcState::InputSecond => {
                if self.waiting_for_second {
                    self.operator = Some(op);
                    // 수식에서 연산자 기호만 교체
                    if let Some(pos) = self.expression.rfind(|c: char| !c.is_ascii_whitespace()) {
                        self.expression.truncate(pos);
                    }
                    self.expression.push_str(op.symbol());
                } else {
                    self.evaluate();
                    if !self.error {
                        let result_str = add_commas(&self.display);
                        self.first_operand = self.parse_display();
                        self.expression = format!("{} {}", result_str, op.symbol());
                        self.operator = Some(op);
                        self.state = CalcState::InputSecond;
                        self.waiting_for_second = true;
                        self.has_decimal = false;
                    }
                }
            }
            CalcState::ShowResult => {
                self.first_operand = self.parse_display();
                self.expression = format!("{} {}", add_commas(&self.display), op.symbol());
                self.operator = Some(op);
                self.state = CalcState::InputSecond;
                self.waiting_for_second = true;
            }
        }
    }

    pub fn press_equals(&mut self) {
        if self.error {
            return;
        }

        if self.state == CalcState::InputSecond {
            let second_operand = if self.waiting_for_second {
                self.first_operand
            } else {
                self.parse_display()
            };
            self.last_operator = self.operator;
            self.last_second_operand = second_operand;

            if let Some(op) = self.operator {
                self.expression = format!(
                    "{} {} {} =",
                    add_commas(&format_result(self.first_operand)),
                    op.symbol(),
                    add_commas(&format_result(second_operand))
                );
            }
            self.evaluate();
        } else if self.state == CalcState::ShowResult {
            if let Some(op) = self.last_operator {
                let current = self.parse_display();
                self.expression = format!(
                    "{} {} {} =",
                    add_commas(&format_result(current)),
                    op.symbol(),
                    add_commas(&format_result(self.last_second_operand))
                );
                self.first_operand = current;
                self.operator = Some(op);
                self.display = format_result(self.last_second_operand);
                self.evaluate();
            }
        }
    }

    pub fn press_clear(&mut self) {
        *self = Self::new();
    }

    pub fn press_percent(&mut self) {
        if self.error {
            return;
        }
        let current = if self.waiting_for_second {
            self.first_operand
        } else {
            self.parse_display()
        };
        let result = match self.operator {
            // +, -: 첫째값의 퍼센트 (예: 10,000 + 10% → 10,000 + 1,000)
            Some(Op::Add) | Some(Op::Sub) => self.first_operand * current / 100.0,
            // ×, ÷: 단순히 /100 (예: 500 × 20% → 500 × 0.2)
            Some(Op::Mul) | Some(Op::Div) => current / 100.0,
            // 연산자 없이 단독: /100
            None => current / 100.0,
        };
        self.display = format_result(result);
        self.has_decimal = self.display.contains('.');
        self.waiting_for_second = false;
    }

    pub fn press_backspace(&mut self) {
        if self.error {
            return;
        }
        if self.state == CalcState::ShowResult {
            return;
        }

        if self.display.len() > 1 {
            let removed = self.display.pop().unwrap();
            if removed == '.' {
                self.has_decimal = false;
            }
        } else {
            self.display = "0".to_string();
        }
    }

    pub fn press_negate(&mut self) {
        if self.error {
            return;
        }
        if self.display == "0" {
            return;
        }

        if self.display.starts_with('-') {
            self.display.remove(0);
        } else {
            self.display.insert(0, '-');
        }
    }

    fn parse_display(&self) -> f64 {
        self.display.parse::<f64>().unwrap_or(0.0)
    }

    fn evaluate(&mut self) {
        let second = self.parse_display();
        let result = match self.operator {
            Some(Op::Add) => self.first_operand + second,
            Some(Op::Sub) => self.first_operand - second,
            Some(Op::Mul) => self.first_operand * second,
            Some(Op::Div) => {
                if second == 0.0 {
                    self.display = "Error".to_string();
                    self.error = true;
                    self.state = CalcState::ShowResult;
                    return;
                }
                self.first_operand / second
            }
            None => return,
        };

        if result.is_infinite() || result.is_nan() {
            self.display = "Error".to_string();
            self.error = true;
        } else {
            self.display = format_result(result);
            self.has_decimal = self.display.contains('.');
        }
        self.state = CalcState::ShowResult;
        self.operator = None;
    }
}

fn add_commas(s: &str) -> String {
    if s == "Error" {
        return s.to_string();
    }

    let (sign, rest) = if s.starts_with('-') {
        ("-", &s[1..])
    } else {
        ("", s.as_ref())
    };

    let (int_part, dec_part) = match rest.find('.') {
        Some(pos) => (&rest[..pos], &rest[pos..]),
        None => (rest, ""),
    };

    let mut result = String::new();
    for (i, ch) in int_part.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }

    format!("{}{}{}", sign, result.chars().rev().collect::<String>(), dec_part)
}

fn format_result(value: f64) -> String {
    if value == value.floor() && value.abs() < 1e15 {
        format!("{}", value as i64)
    } else {
        let s = format!("{}", value);
        if s.len() > 16 {
            format!("{:.10}", value)
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string()
        } else {
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc() -> CalcEngine {
        CalcEngine::new()
    }

    #[test]
    fn test_initial_display() {
        let c = calc();
        assert_eq!(c.display(), "0");
    }

    #[test]
    fn test_digit_input() {
        let mut c = calc();
        c.press_digit(1);
        c.press_digit(2);
        c.press_digit(3);
        assert_eq!(c.display(), "123");
    }

    #[test]
    fn test_leading_zero() {
        let mut c = calc();
        c.press_digit(0);
        c.press_digit(5);
        assert_eq!(c.display(), "5");
    }

    #[test]
    fn test_addition() {
        let mut c = calc();
        c.press_digit(2);
        c.press_operator(Op::Add);
        c.press_digit(3);
        c.press_equals();
        assert_eq!(c.display(), "5");
    }

    #[test]
    fn test_subtraction() {
        let mut c = calc();
        c.press_digit(1);
        c.press_digit(0);
        c.press_operator(Op::Sub);
        c.press_digit(4);
        c.press_equals();
        assert_eq!(c.display(), "6");
    }

    #[test]
    fn test_multiplication() {
        let mut c = calc();
        c.press_digit(3);
        c.press_operator(Op::Mul);
        c.press_digit(7);
        c.press_equals();
        assert_eq!(c.display(), "21");
    }

    #[test]
    fn test_division() {
        let mut c = calc();
        c.press_digit(8);
        c.press_operator(Op::Div);
        c.press_digit(2);
        c.press_equals();
        assert_eq!(c.display(), "4");
    }

    #[test]
    fn test_division_by_zero() {
        let mut c = calc();
        c.press_digit(5);
        c.press_operator(Op::Div);
        c.press_digit(0);
        c.press_equals();
        assert_eq!(c.display(), "Error");
    }

    #[test]
    fn test_decimal() {
        let mut c = calc();
        c.press_digit(1);
        c.press_decimal();
        c.press_digit(5);
        c.press_operator(Op::Add);
        c.press_digit(2);
        c.press_decimal();
        c.press_digit(3);
        c.press_equals();
        assert_eq!(c.display(), "3.8");
    }

    #[test]
    fn test_no_double_decimal() {
        let mut c = calc();
        c.press_digit(1);
        c.press_decimal();
        c.press_decimal();
        c.press_digit(5);
        assert_eq!(c.display(), "1.5");
    }

    #[test]
    fn test_operator_chaining() {
        let mut c = calc();
        c.press_digit(2);
        c.press_operator(Op::Add);
        c.press_digit(3);
        c.press_operator(Op::Add);
        assert_eq!(c.display(), "5");
        c.press_digit(4);
        c.press_equals();
        assert_eq!(c.display(), "9");
    }

    #[test]
    fn test_clear() {
        let mut c = calc();
        c.press_digit(5);
        c.press_operator(Op::Add);
        c.press_digit(3);
        c.press_clear();
        assert_eq!(c.display(), "0");
    }

    #[test]
    fn test_percent_basic() {
        // 단독: 50% = 0.5
        let mut c = calc();
        c.press_digit(5);
        c.press_digit(0);
        c.press_percent();
        assert_eq!(c.display(), "0.5");
    }

    #[test]
    fn test_percent_multiply() {
        // 비율 계산: 500 × 20% = 100
        let mut c = calc();
        c.press_digit(5);
        c.press_digit(0);
        c.press_digit(0);
        c.press_operator(Op::Mul);
        c.press_digit(2);
        c.press_digit(0);
        c.press_percent();
        assert_eq!(c.display(), "0.2");
        c.press_equals();
        assert_eq!(c.display(), "100");
    }

    #[test]
    fn test_percent_discount() {
        // 할인: 10,000 - 15% = 8,500
        let mut c = calc();
        c.press_digit(1);
        c.press_digit(0);
        c.press_digit(0);
        c.press_digit(0);
        c.press_digit(0);
        c.press_operator(Op::Sub);
        c.press_digit(1);
        c.press_digit(5);
        c.press_percent();
        assert_eq!(c.display(), "1,500");
        c.press_equals();
        assert_eq!(c.display(), "8,500");
    }

    #[test]
    fn test_percent_tax() {
        // 세금: 10,000 + 10% = 11,000
        let mut c = calc();
        c.press_digit(1);
        c.press_digit(0);
        c.press_digit(0);
        c.press_digit(0);
        c.press_digit(0);
        c.press_operator(Op::Add);
        c.press_digit(1);
        c.press_digit(0);
        c.press_percent();
        assert_eq!(c.display(), "1,000");
        c.press_equals();
        assert_eq!(c.display(), "11,000");
    }

    #[test]
    fn test_backspace() {
        let mut c = calc();
        c.press_digit(1);
        c.press_digit(2);
        c.press_digit(3);
        c.press_backspace();
        assert_eq!(c.display(), "12");
        c.press_backspace();
        assert_eq!(c.display(), "1");
        c.press_backspace();
        assert_eq!(c.display(), "0");
    }

    #[test]
    fn test_negate() {
        let mut c = calc();
        c.press_digit(5);
        c.press_negate();
        assert_eq!(c.display(), "-5");
        c.press_negate();
        assert_eq!(c.display(), "5");
    }

    #[test]
    fn test_negate_zero() {
        let mut c = calc();
        c.press_negate();
        assert_eq!(c.display(), "0");
    }

    #[test]
    fn test_result_then_digit() {
        let mut c = calc();
        c.press_digit(2);
        c.press_operator(Op::Add);
        c.press_digit(3);
        c.press_equals();
        assert_eq!(c.display(), "5");
        c.press_digit(9);
        assert_eq!(c.display(), "9");
    }

    #[test]
    fn test_result_then_operator() {
        let mut c = calc();
        c.press_digit(2);
        c.press_operator(Op::Add);
        c.press_digit(3);
        c.press_equals();
        assert_eq!(c.display(), "5");
        c.press_operator(Op::Mul);
        c.press_digit(2);
        c.press_equals();
        assert_eq!(c.display(), "10");
    }

    #[test]
    fn test_clear_after_error() {
        let mut c = calc();
        c.press_digit(5);
        c.press_operator(Op::Div);
        c.press_digit(0);
        c.press_equals();
        assert_eq!(c.display(), "Error");
        c.press_clear();
        assert_eq!(c.display(), "0");
    }

    #[test]
    fn test_error_blocks_input() {
        let mut c = calc();
        c.press_digit(5);
        c.press_operator(Op::Div);
        c.press_digit(0);
        c.press_equals();
        c.press_digit(3);
        assert_eq!(c.display(), "Error");
    }

    #[test]
    fn test_comma_in_input() {
        let mut c = calc();
        for d in [1, 2, 3, 4, 5, 6, 7] {
            c.press_digit(d);
        }
        assert_eq!(c.display(), "1,234,567");
    }

    #[test]
    fn test_comma_in_result() {
        let mut c = calc();
        // 5000 * 3 = 15000
        c.press_digit(5);
        c.press_digit(0);
        c.press_digit(0);
        c.press_digit(0);
        c.press_operator(Op::Mul);
        c.press_digit(3);
        c.press_equals();
        assert_eq!(c.display(), "15,000");
    }

    #[test]
    fn test_comma_with_decimal() {
        let mut c = calc();
        for d in [1, 2, 3, 4, 5] {
            c.press_digit(d);
        }
        c.press_decimal();
        c.press_digit(6);
        assert_eq!(c.display(), "12,345.6");
    }

    #[test]
    fn test_comma_negative() {
        let mut c = calc();
        for d in [1, 0, 0, 0] {
            c.press_digit(d);
        }
        c.press_negate();
        assert_eq!(c.display(), "-1,000");
    }

    #[test]
    fn test_repeat_equals() {
        let mut c = calc();
        // 55 × 5 = 275
        c.press_digit(5);
        c.press_digit(5);
        c.press_operator(Op::Mul);
        c.press_digit(5);
        c.press_equals();
        assert_eq!(c.display(), "275");
        // = again: 275 × 5 = 1,375
        c.press_equals();
        assert_eq!(c.display(), "1,375");
        // = again: 1375 × 5 = 6,875
        c.press_equals();
        assert_eq!(c.display(), "6,875");
    }

    #[test]
    fn test_repeat_equals_addition() {
        let mut c = calc();
        // 10 + 5 = 15
        c.press_digit(1);
        c.press_digit(0);
        c.press_operator(Op::Add);
        c.press_digit(5);
        c.press_equals();
        assert_eq!(c.display(), "15");
        // = again: 15 + 5 = 20
        c.press_equals();
        assert_eq!(c.display(), "20");
    }

    #[test]
    fn test_repeat_equals_expression() {
        let mut c = calc();
        c.press_digit(5);
        c.press_digit(5);
        c.press_operator(Op::Mul);
        c.press_digit(5);
        c.press_equals();
        assert_eq!(c.expression(), "55 × 5 =");
        c.press_equals();
        assert_eq!(c.expression(), "275 × 5 =");
    }
}
