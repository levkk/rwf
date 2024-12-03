//! Wrapper around [`colored::Colorize`] to conditionally
//! use colors when the terminal is TTY.
use crate::config::get_config;
use colored::Colorize;

/// Use terminal colors only if terminal is TTY.
pub trait MaybeColorize {
    /// Make text green.
    fn green(&self) -> String;
    /// Make text red.
    fn red(&self) -> String;
    /// Make text purple.
    fn purple(&self) -> String;
    /// Make text yellow.
    fn yellow(&self) -> String;
    /// Make text bold.
    fn bold(&self) -> String;
}

impl MaybeColorize for &str {
    fn green(&self) -> String {
        let config = get_config();

        if config.general.tty {
            Colorize::green(*self).to_string()
        } else {
            self.to_string()
        }
    }

    fn red(&self) -> String {
        let config = get_config();

        if config.general.tty {
            Colorize::red(*self).to_string()
        } else {
            self.to_string()
        }
    }

    fn purple(&self) -> String {
        let config = get_config();

        if config.general.tty {
            Colorize::purple(*self).to_string()
        } else {
            self.to_string()
        }
    }

    fn yellow(&self) -> String {
        let config = get_config();

        if config.general.tty {
            Colorize::yellow(*self).to_string()
        } else {
            self.to_string()
        }
    }

    fn bold(&self) -> String {
        let config = get_config();

        if config.general.tty {
            Colorize::bold(*self).to_string()
        } else {
            self.to_string()
        }
    }
}

impl MaybeColorize for String {
    fn green(&self) -> String {
        MaybeColorize::green(&self.as_str())
    }

    fn red(&self) -> String {
        MaybeColorize::red(&self.as_str())
    }

    fn purple(&self) -> String {
        MaybeColorize::purple(&self.as_str())
    }

    fn yellow(&self) -> String {
        MaybeColorize::yellow(&self.as_str())
    }

    fn bold(&self) -> String {
        MaybeColorize::bold(&self.as_str())
    }
}

#[cfg(test)]
mod test {}
