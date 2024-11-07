# Migrating from Rails

Rwf comes with its own [Rack](https://github.com/rack/rack) server which you can run side by side with your [controllers](controllers/index.md). This allows you to run your Rails app inside Rwf, while moving traffic from Rails controllers to Rwf controllers without downtime.

## Running Rails

!!! note
    Our Rack server is currently highly experimental. Ruby doesn't have ergonomic bindings to Rust (unlike [Python](migrating-from-python.md)), so
    a lot of this had to be written specifically for this project.

### Ruby

Depending on your operating system, Ruby installation may differ.

#### Linux

On Linux, you can install Ruby using your package manager. Since Rails uses Bundler to manage dependencies, make sure you set the `GEM_HOME` environment variable to a folder writable by your UNIX user, e.g. `~/.gems`:

```bash
mkdir -p ~/.gems
export GEM_HOME=~/.gems
```

Since we'll be compiling our Ruby bindings from source, make sure to install the Ruby headers as well. On most systems, that'll come from the `ruby-dev` package.

#### Mac OS

Mac OS comes with its own Ruby version, however it's very much out of date and won't work with modern Rails apps. You'll need to install Ruby from [homebrew](https://brew.sh/):

```bash
brew install ruby
```

When building a Rwf app with Ruby bindings, you'll need to make sure the linker can find the right Ruby library. Since you'll have two versions of Ruby installed at this point, the linker will get confused and use the system one, which won't work. To get around this, create a `.cargo/config` file in your project, and add the right library path to the linker arguments:

```toml
[build]
rustflags = ["-C", "link-args=-L/opt/homebrew/Cellar/ruby/3.3.4/lib"]
```

The path here will depend on the version of Ruby you have installed with homebrew.

### Running the app

Running a Rails app with Rwf requires only adding the built-in Rack controller to your server:

```rust
use rwf::prelude::*;
use rwf::http::{Server, self};
use rwf::controllers::RackController;

#[tokio::main]
async fn main() -> Result<(), http::Error> {
    Server::new(vec![
        RackController::new("path/to/your/rails/app")
            .wildcard("/")
    ])
    .launch("0.0.0.0:8000")
    .await
}
```

The `RackController` takes the path (relative or absolute) to your Rails application as an argument.



## Learn more

- [examples/rails](https://github.com/levkk/rwf/tree/main/examples/rails)
