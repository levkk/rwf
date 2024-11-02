use rwf::colors::MaybeColorize;

pub fn created(something: impl ToString) {
    eprintln!("{} {}", "created".green(), something.to_string());
}

pub fn error(something: impl ToString) {
    eprintln!("{}: {}", "error".red(), something.to_string());
}
