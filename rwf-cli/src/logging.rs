use rwf::colors::MaybeColorize;

pub fn created(something: impl ToString) {
    eprintln!("{} {}", "created".green(), something.to_string());
}
