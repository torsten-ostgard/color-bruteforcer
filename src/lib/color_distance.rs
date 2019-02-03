extern crate palette;

use palette::white_point::D65;
use palette::Laba;

pub fn distance(lab1: &Laba<D65>, lab2: &Laba<D65>) -> f32 {
    // Adapted slightly from the Scarlet library. The original implementation can be found here:
    // https://github.com/nicholas-miklaucic/scarlet/blob/66bf96f/src/color.rs#L713

    // implementation reference found here:
    // https://pdfs.semanticscholar.org/969b/c38ea067dd22a47a44bcb59c23807037c8d8.pdf

    // I'm going to match the notation in that text pretty much exactly: it's the only way to
    // keep this both concise and readable

    // step 1: calculation of C and h
    // the method hypot returns sqrt(a^2 + b^2)
    let c_star_1: f32 = lab1.a.hypot(lab1.b);
    let c_star_2: f32 = lab2.a.hypot(lab2.b);

    let c_bar_ab: f32 = (c_star_1 + c_star_2) / 2.0;
    let g = 0.5 * (1.0 - ((c_bar_ab.powi(7)) / (c_bar_ab.powi(7) + 25.0f32.powi(7))).sqrt());

    let a_prime_1 = (1.0 + g) * lab1.a;
    let a_prime_2 = (1.0 + g) * lab2.a;

    let c_prime_1 = a_prime_1.hypot(lab1.b);
    let c_prime_2 = a_prime_2.hypot(lab2.b);

    // this closure simply does the atan2 like CIELCH, but safely accounts for a == b == 0
    // we're gonna do this twice, so I just use a closure
    let h_func = |a: f32, b: f32| {
        if a == 0.0 && b == 0.0 {
            0.0
        } else {
            let val = b.atan2(a).to_degrees();
            if val < 0.0 {
                val + 360.0
            } else {
                val
            }
        }
    };

    let h_prime_1 = h_func(a_prime_1, lab1.b);
    let h_prime_2 = h_func(a_prime_2, lab2.b);

    // step 2: computing delta L, delta C, and delta H
    // take a deep breath, you got this!

    let delta_l = lab2.l - lab1.l;
    let delta_c = c_prime_2 - c_prime_1;
    // essentially, compute the difference in hue but keep it in the right range
    let delta_angle_h = if c_prime_1 * c_prime_2 == 0.0 {
        0.0
    } else if (h_prime_2 - h_prime_1).abs() <= 180.0 {
        h_prime_2 - h_prime_1
    } else if h_prime_2 - h_prime_1 > 180.0 {
        h_prime_2 - h_prime_1 - 360.0
    } else {
        h_prime_2 - h_prime_1 + 360.0
    };
    // now get the Cartesian equivalent of the angle difference in hue
    // this also corrects for chromaticity mattering less at low luminances
    let delta_h = 2.0 * (c_prime_1 * c_prime_2).sqrt() * (delta_angle_h / 2.0).to_radians().sin();

    // step 3: the color difference
    // if you're reading this, it's not too late to back out
    let l_bar_prime = (lab1.l + lab2.l) / 2.0;
    let c_bar_prime = (c_prime_1 + c_prime_2) / 2.0;
    let h_bar_prime = if c_prime_1 * c_prime_2 == 0.0 {
        h_prime_1 + h_prime_2
    } else if (h_prime_2 - h_prime_1).abs() <= 180.0 {
        (h_prime_1 + h_prime_2) / 2.0
    } else if h_prime_1 + h_prime_2 < 360.0 {
        (h_prime_1 + h_prime_2 + 360.0) / 2.0
    } else {
        (h_prime_1 + h_prime_2 - 360.0) / 2.0
    };

    // we're gonna use this a lot
    let deg_cos = |x: f32| x.to_radians().cos();

    let t = 1.0 - 0.17 * deg_cos(h_bar_prime - 30.0)
        + 0.24 * deg_cos(2.0 * h_bar_prime)
        + 0.32 * deg_cos(3.0 * h_bar_prime + 6.0)
        - 0.20 * deg_cos(4.0 * h_bar_prime - 63.0);

    let delta_theta = 30.0 * (-((h_bar_prime - 275.0) / 25.0).powi(2)).exp();
    let r_c = 2.0 * (c_bar_prime.powi(7) / (c_bar_prime.powi(7) + 25.0f32.powi(7))).sqrt();
    let s_l = 1.0
        + ((0.015 * (l_bar_prime - 50.0).powi(2)) / (20.0 + (l_bar_prime - 50.0).powi(2)).sqrt());
    let s_c = 1.0 + 0.045 * c_bar_prime;
    let s_h = 1.0 + 0.015 * c_bar_prime * t;
    let r_t = -r_c * (2.0 * delta_theta).to_radians().sin();
    // finally, the end result
    // in the original there are three parametric weights, used for weighting differences in
    // lightness, chroma, or hue. In pretty much any application, including this one, all of
    // these are 1, so they're omitted
    ((delta_l / s_l).powi(2)
        + (delta_c / s_c).powi(2)
        + (delta_h / s_h).powi(2)
        + r_t * (delta_c / s_c) * (delta_h / s_h))
        .sqrt()
}
