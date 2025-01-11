// Copyright 2016 John Ward under MIT

use std::f64::consts::FRAC_PI_2;

use fixed::types::I16F16;
use fixed_exp::FixedPowF;
use fixed_trigonometry::sin;
pub(crate) type Fix = I16F16;

macro_rules! easer {
    ($f:ident, $t:ident, $e:expr) => {
        pub struct $t {
            start: Fix,
            dist: Fix,
            step: u64,
            steps: u64,
        }

        pub fn $f(start: Fix, end: Fix, steps: u64) -> $t {
            $t {
                start,
                dist: end - start,
                step: 0,
                steps,
            }
        }

        impl Iterator for $t {
            type Item = Fix;

            fn next(&mut self) -> Option<Fix> {
                self.step += 1;
                if self.step > self.steps {
                    None
                } else {
                    let x: Fix = Fix::from_num(self.step) / Fix::from_num(self.steps);
                    Some(Fix::from_num($e(x)).mul_add(self.dist, self.start))
                }
            }
        }
    };
}

easer!(linear, Linear, |x: Fix| { x });
easer!(quad_in, QuadIn, |x: Fix| { x * x });
easer!(quad_out, QuadOut, |x: Fix| {
    -(x * (x - Fix::from_num(2)))
});
easer!(quad_inout, QuadInOut, |x: Fix| -> Fix {
    if x < Fix::from_num(0.5) {
        Fix::from_num(2) * x * x
    } else {
        (Fix::from_num(-2) * x * x) + x.mul_add(Fix::from_num(4), Fix::from_num(-1))
    }
});
easer!(cubic_in, CubicIn, |x: Fix| { x * x * x });
easer!(cubic_out, CubicOut, |x: Fix| {
    let y = x - Fix::from_num(1);
    y * y * y + Fix::from_num(1)
});
easer!(cubic_inout, CubicInOut, |x: Fix| {
    if x < Fix::from_num(0.5) {
        Fix::from_num(4) * x * x * x
    } else {
        let y = x.mul_add(2.into(), Fix::from_num(-2));
        (y * y * y).mul_add(Fix::from_num(0.5), Fix::from_num(1))
    }
});
easer!(quartic_in, QuarticIn, |x: Fix| { x * x * x * x });
easer!(quartic_out, QuarticOut, |x: Fix| {
    let y = x - Fix::from_num(1);
    (y * y * y).mul_add(Fix::from_num(1) - x, Fix::from_num(1))
});
easer!(quartic_inout, QuarticInOut, |x: Fix| {
    if x < Fix::from_num(0.5) {
        Fix::from_num(8) * x * x * x * x
    } else {
        let y = x - Fix::from_num(1);
        (y * y * y * y).mul_add(Fix::from_num(-8), Fix::from_num(1))
    }
});
easer!(sin_in, SinIn, |x: Fix| {
    let y = (x - Fix::from_num(1)) * Fix::from_num(FRAC_PI_2);
    sin(y) + Fix::from_num(1)
});
easer!(sin_out, SinOut, |x: Fix| {
    sin(x * Fix::from_num(FRAC_PI_2))
});
easer!(sin_inout, SinInOut, |x: Fix| {
    if x < Fix::from_num(0.5) {
        Fix::from_num(0.5)
            * (Fix::from_num(1) - (x * x).mul_add(Fix::from_num(-4), Fix::from_num(1)).sqrt())
    } else {
        Fix::from_num(0.5)
            * ((x.mul_add(Fix::from_num(-2), Fix::from_num(3))
                * x.mul_add(Fix::from_num(2), Fix::from_num(-1)))
            .sqrt()
                + Fix::from_num(1))
    }
});
easer!(exp_in, ExpIn, |x: Fix| {
    if x == 0. {
        Fix::from_num(0)
    } else {
        Fix::from_num(2).powf(Fix::from_num(10) * (x - Fix::from_num(1)))
    }
});

easer!(exp_out, ExpOut, |x: Fix| {
    if x == Fix::from_num(1) {
        Fix::from_num(1)
    } else {
        Fix::from_num(2).powf(-Fix::from_num(10) * x) * Fix::from_num(-1) + Fix::from_num(1)
    }
});
easer!(exp_inout, ExpInOut, |x: Fix| {
    if x == Fix::from_num(1) {
        Fix::from_num(1)
    } else if x == 0. {
        Fix::from_num(0)
    } else if x < Fix::from_num(0.5) {
        Fix::from_num(2).powf(x.mul_add(Fix::from_num(20), Fix::from_num(-10))) * Fix::from_num(0.5)
    } else {
        Fix::from_num(2)
            .powf(x.mul_add(Fix::from_num(-20), Fix::from_num(10)))
            .mul_add(Fix::from_num(-0.5), Fix::from_num(1))
    }
});

#[cfg(test)]
mod test {
    // acceptable relative error (0.1%)
    const ERROR_MARGIN_FAC: f64 = 0.00015;

    use std::{fs::File, iter::zip, path::PathBuf};

    use anyhow::anyhow;

    use super::*;
    macro_rules! function {
        () => {{
            fn f() {}
            fn type_name_of<T>(_: T) -> &'static str {
                std::any::type_name::<T>()
            }
            let name = type_name_of(f);
            let full_path = name.strip_suffix("::f").unwrap();
            full_path.split("::").last().unwrap()
        }};
    }

    /// accept a Vec of `is_data` if it is within the defined error margin of `ought_data`
    fn must_be_withing_error_margin_else_write(
        test_name: &str,
        ought_data: Vec<f64>,
        is_data: Vec<Fix>,
    ) -> anyhow::Result<()> {
        let min = ought_data
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = ought_data
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let value_range = max - min;

        let max_error = (value_range * ERROR_MARGIN_FAC).abs();
        let inside_margin = |is: f64, ought: f64| -> bool {
            let delta = (is - ought).abs();

            let res = delta <= max_error;
            if !res {
                eprintln!("measured error outside of acceptable margin: {is} <> {ought}");
            }
            res
        };

        let mut ok = true;
        for (ought, is) in zip(ought_data.iter(), is_data.iter()) {
            let is = is.to_num::<f64>();
            if !inside_margin(is, *ought) {
                ok = false;
            }
        }

        if !ok {
            let root = option_env!("CARGO_MANIFEST_DIR")
                .ok_or(anyhow!("missing env var CARGO_MANIFEST_DIR"))?;
            let root = PathBuf::from(root);
            let target_path = root.join("jupyter-tests");

            let ought_file = File::create(target_path.join(format!("{test_name}-ought.json")))?;
            let is_file = File::create(target_path.join(format!("{test_name}-is.json")))?;

            let is_converted = is_data
                .into_iter()
                .map(|fix| fix.to_num())
                .collect::<Vec<f64>>();
            serde_json::to_writer(ought_file, &ought_data)?;
            serde_json::to_writer(is_file, &is_converted)?;

            panic!("{test_name} outside of error {ERROR_MARGIN_FAC} margin: {is_converted:?} <> {ought_data:?}");
        }
        Ok(())
    }

    #[test]
    fn linear_test() -> anyhow::Result<()> {
        let model = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let res: Vec<Fix> = linear(Fix::from_num(0), Fix::from_num(1), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quad_in_test() -> anyhow::Result<()> {
        let model = vec![
            100., 400., 900., 1600., 2500., 3600., 4900., 6400., 8100., 10000.,
        ];
        let res: Vec<Fix> = quad_in(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quad_out_test() -> anyhow::Result<()> {
        let model = vec![
            1900., 3600., 5100., 6400., 7500., 8400., 9100., 9600., 9900., 10000.,
        ];
        let res: Vec<Fix> = quad_out(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quad_inout_test() -> anyhow::Result<()> {
        let model = vec![
            200., 800., 1800., 3200., 5000., 6800., 8200., 9200., 9800., 10000.,
        ];
        let res: Vec<Fix> = quad_inout(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn cubic_in_test() -> anyhow::Result<()> {
        let model = vec![
            10., 80., 270., 640., 1250., 2160., 3430., 5120., 7290., 10000.,
        ];
        let res: Vec<Fix> = cubic_in(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn cubic_out_test() -> anyhow::Result<()> {
        let model = vec![
            2710., 4880., 6570., 7840., 8750., 9360., 9730., 9920., 9990., 10000.,
        ];
        let res: Vec<Fix> = cubic_out(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quartic_in_test() -> anyhow::Result<()> {
        let model = vec![1., 16., 81., 256., 625., 1296., 2401., 4096., 6561., 10000.];
        let res: Vec<Fix> = quartic_in(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quartic_out_test() -> anyhow::Result<()> {
        let model = vec![
            3439., 5904., 7599., 8704., 9375., 9744., 9919., 9984., 9999., 10000.,
        ];
        let res: Vec<Fix> = quartic_out(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn quartic_inout_test() -> anyhow::Result<()> {
        let model = vec![
            8., 128., 648., 2048., 5000., 7952., 9352., 9872., 9992., 10000.,
        ];
        let res: Vec<Fix> = quartic_inout(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn sin_in_test() -> anyhow::Result<()> {
        let model = vec![
            123.116594,
            489.434837,
            1089.934758,
            1909.830056,
            2928.932188,
            4122.147477,
            5460.095003,
            6909.830056,
            8435.655350,
            10000.,
        ];
        let res: Vec<Fix> = sin_in(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn sin_out_test() -> anyhow::Result<()> {
        let model = vec![
            1564.344650,
            3090.169944,
            4539.904997,
            5877.852523,
            7071.067812,
            8090.169944,
            8910.065242,
            9510.565163,
            9876.883406,
            10000.,
        ];
        let res: Vec<Fix> = sin_out(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn sin_inout_test() -> anyhow::Result<()> {
        let model = vec![
            101.020514,
            417.424305,
            1000.,
            2000.,
            5000.,
            8000.,
            9000.,
            9582.575695,
            9898.979486,
            10000.,
        ];
        let res: Vec<Fix> = sin_inout(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn exp_in_test() -> anyhow::Result<()> {
        let model = vec![
            19.53125, 39.0625, 78.125, 156.25, 312.5, 625., 1250., 2500., 5000., 10000.,
        ];
        let res: Vec<Fix> = exp_in(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn exp_out_test() -> anyhow::Result<()> {
        let model = vec![
            5000., 7500., 8750., 9375., 9687.5, 9843.75, 9921.875, 9960.9375, 9980.46875, 10000.,
        ];
        let res: Vec<Fix> = exp_out(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }

    #[test]
    fn exp_inout_test() -> anyhow::Result<()> {
        let model = vec![
            19.53125, 78.125, 312.5, 1250., 5000., 8750., 9687.5, 9921.875, 9980.46875, 10000.,
        ];
        let res: Vec<Fix> = exp_inout(Fix::from_num(0), Fix::from_num(10000), 10).collect();
        must_be_withing_error_margin_else_write(function!(), model, res)
    }
}
