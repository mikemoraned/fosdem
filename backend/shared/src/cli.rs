use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;

pub fn progress_bar(total_size: u64) -> ProgressBar {
    use indicatif::HumanDuration;

    let bar = ProgressBar::new(total_size);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>5}/{len:5} ({eta})",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{}", HumanDuration(state.eta())).unwrap()
        }),
    );
    bar
}
