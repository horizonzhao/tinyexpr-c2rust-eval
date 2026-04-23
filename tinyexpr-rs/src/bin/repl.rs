use std::io::{self, BufRead, Write};
use tinyexpr_rs::te_interp;

/// Replicate C's printf("%g", v): 6 significant figures, scientific when
/// exponent < -4 or >= 6, trailing zeros stripped, exponent sign always shown
/// with at least 2 digits (e.g. "1e+06", "1e-05").
fn format_g(v: f64) -> String {
    if v.is_nan() {
        return "nan".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    if v == 0.0 {
        return "0".to_string();
    }

    // Format in scientific notation to extract the exponent reliably.
    let sci = format!("{:.5e}", v); // 6 sig-figs = 5 after the decimal in sci form
    let exp: i32 = sci
        .split('e')
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if exp >= -4 && exp < 6 {
        // Fixed notation: PREC-1-exp decimal places (clamped to 0)
        let dec = (5 - exp).max(0) as usize;
        trim_zeros(format!("{:.prec$}", v, prec = dec))
    } else {
        // Scientific notation in C format
        let mantissa = trim_zeros(sci.split('e').next().unwrap_or("").to_string());
        if exp >= 0 {
            format!("{}e+{:02}", mantissa, exp)
        } else {
            format!("{}e-{:02}", mantissa, exp.abs())
        }
    }
}

fn trim_zeros(s: String) -> String {
    if !s.contains('.') {
        return s;
    }
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}

fn main() {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        eprint!("> ");
        io::stderr().flush().ok();
        match lines.next() {
            None => break,
            Some(Err(_)) => break,
            Some(Ok(line)) => {
                if line == "q" || line == "quit" {
                    break;
                }
                let mut err = 0i32;
                let result = te_interp(&line, &mut err);
                if err != 0 {
                    println!("Error at position {}", err);
                } else {
                    println!("{}", format_g(result));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn g_integer() {
        assert_eq!(format_g(3.0), "3");
    }
    #[test]
    fn g_pi() {
        assert_eq!(format_g(std::f64::consts::PI), "3.14159");
    }
    #[test]
    fn g_one_third() {
        assert_eq!(format_g(1.0 / 3.0), "0.333333");
    }
    #[test]
    fn g_large() {
        assert_eq!(format_g(1e10), "1e+10");
    }
    #[test]
    fn g_million() {
        assert_eq!(format_g(1e6), "1e+06");
    }
    #[test]
    fn g_small_fixed() {
        assert_eq!(format_g(0.0001), "0.0001");
    }
    #[test]
    fn g_small_sci() {
        assert_eq!(format_g(0.00001), "1e-05");
    }
    #[test]
    fn g_fp_sum() {
        assert_eq!(format_g(0.1 + 0.2), "0.3");
    }
    #[test]
    fn g_pow2_53() {
        assert_eq!(format_g(f64::powi(2.0, 53)), "9.0072e+15");
    }
    #[test]
    fn g_nan() {
        assert_eq!(format_g(f64::NAN), "nan");
    }
    #[test]
    fn g_inf() {
        assert_eq!(format_g(f64::INFINITY), "inf");
    }
    #[test]
    fn g_neg_inf() {
        assert_eq!(format_g(f64::NEG_INFINITY), "-inf");
    }
    #[test]
    fn g_zero() {
        assert_eq!(format_g(0.0), "0");
    }
    #[test]
    fn g_1e308() {
        assert_eq!(format_g(1e308), "1e+308");
    }
}
