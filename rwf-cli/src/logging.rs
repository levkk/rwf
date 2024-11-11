use rwf::colors::MaybeColorize;

pub fn written(something: impl ToString) {
    eprintln!("{} {}", "written".green(), something.to_string());
}

pub fn created(something: impl ToString) {
    eprintln!("{} {}", "created".green(), something.to_string());
}

pub fn error(something: impl ToString) {
    eprintln!("{}: {}", "error".red(), something.to_string());
}

pub fn warning(something: impl ToString) {
    eprintln!("{}: {}", "warning".yellow(), something.to_string());
}

pub fn removed(something: impl ToString) {
    eprintln!("{} {}", "removed".red(), something.to_string());
}

pub fn packaging(something: impl ToString) {
    eprintln!("{} {}", "packaging".green(), something.to_string());
}
