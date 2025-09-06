pub fn throw_error(message: &str, dry_run: bool) {
    if dry_run {
        panic!("{}", message);
    } else {
        eprintln!("{}", message);
    }
}