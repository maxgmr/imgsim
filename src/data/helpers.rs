#![warn(missing_docs)]
//! Various helper functions for different `imgsim` processes.

/// Convert HSL value to RGB value.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let l_min = if l > 1.0 - l { 1.0 - l } else { l };
    let a = s * l_min;
    fn f(n: u128, h: f32, l: f32, a: f32) -> u8 {
        let k = ((n + (h / 30.0) as u128) % 12) as u8;
        let k_min: i16 = vec![k as i16 - 3, 9 - k as i16, 1]
            .into_iter()
            .min()
            .unwrap()
            .into();
        let k_max: i16 = if k_min > -1 { k_min } else { -1 };
        let colour = l - a * k_max as f32;
        (255.0 * colour).round() as u8
    }
    (f(0, h, l, a), f(8, h, l, a), f(4, h, l, a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn htr() {
        assert_eq!(hsl_to_rgb(60.0, 0.8182, 0.4314), (200, 200, 20));
    }
}
