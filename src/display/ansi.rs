macro_rules! ansicode {
    ($code:expr) => {
        concat!("\x1b[", $code, "m")
    };
}

pub(crate) const STYLE_END: &str = ansicode!(0);
pub(crate) const STYLE_BOLD: &str = ansicode!(1);
pub(crate) const STYLE_DIM: &str = ansicode!(2);
// pub(crate) const STYLE_ITALIC: &str = ansicode!(3);
// pub(crate) const STYLE_UNDERLINE: &str = ansicode!(4);

// pub(crate) const FG_BLACK: &str = ansicode!(30);
// pub(crate) const BG_BLACK: &str = ansicode!(40);

pub(crate) const FG_RED: &str = ansicode!(31);
//  pub(crate) const BG_RED: &str = ansicode!(41);

pub(crate) const FG_BLUE: &str = ansicode!(34);

pub(crate) const FG_DEFAULT: &str = ansicode!(39);
// pub(crate) const BG_DEFAULT: &str = ansicode!(49);
