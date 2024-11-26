use rwf::colors::MaybeColorize;

pub fn written(something: impl ToString) {
    eprintln!("{} {}", "    Written".green().bold(), something.to_string());
}

pub fn created(something: impl ToString) {
    eprintln!("{} {}", "    Created".green().bold(), something.to_string());
}

pub fn error(something: impl ToString) {
    eprintln!("{}: {}", "error".red().bold(), something.to_string());
}

pub fn warning(something: impl ToString) {
    eprintln!(
        "{}: {}",
        "    Warning".yellow().bold(),
        something.to_string()
    );
}

pub fn removed(something: impl ToString) {
    eprintln!("{} {}", "    Removed".red().bold(), something.to_string());
}

pub fn packaging(something: impl ToString) {
    eprintln!(
        "{} {}",
        "    Packaging".green().bold(),
        something.to_string()
    );
}

pub fn using(something: impl ToString) {
    eprintln!("    {} {}", "Using".green().bold(), something.to_string());
}
