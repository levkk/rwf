use rwf::http::Server;
use rwf::prelude::*;

use rand::Rng;

#[derive(Clone, rwf::macros::Model)]
struct User {
    id: Option<i64>,
    email: String,
}

#[derive(Default)]
struct IndexController;

#[derive(rwf::macros::Context)]
struct IndexTemplate {
    title: String,
    items: Vec<String>,
    show: bool,
    planets: i64,
    users: Vec<User>,
}

#[async_trait]
impl Controller for IndexController {
    async fn handle(&self, _request: &Request) -> Result<Response, Error> {
        let context = IndexTemplate {
            title: "Rum templates are fun!".into(),
            items: vec!["why".into(), "are".into(), "you".into(), "yelling".into()],
            show: rand::thread_rng().gen::<bool>(),
            planets: rand::thread_rng().gen_range(1..=3),
            users: vec![User {
                id: Some(1),
                email: "hello@test.com".into(),
            }],
        };

        println!("strt render");
        let rendered = Template::load("templates/index.html")?.render(context)?;
        println!("render ok");

        Ok(Response::new().html(rendered))
    }
}

#[tokio::main]
async fn main() {
    Logger::init();

    Server::new(vec![IndexController::default().route("/")])
        .launch("0.0.0.0:8000")
        .await
        .expect("error shutting down server");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_turtles() -> Result<(), Error> {
        let template = Template::from_str(
            r#"
<div class="profile">
    <h2><%= username %></h2>
    <p><%= bio %></p>
</div>
        "#,
        )?;

        let html = template.render([("username", "Alice"), ("bio", "I like turtles")])?;

        assert_eq!(
            html,
            r#"
<div class="profile">
    <h2>Alice</h2>
    <p>I like turtles</p>
</div>
        "#
        );
        Ok(())
    }

    #[test]
    fn test_context() -> Result<(), Error> {
        let _ctx = context!(
            "var1" => "A string value",
            "var2" => vec![
                1_i64, 1, 2, 3, 5, 8,
            ],
        );

        #[derive(macros::Context)]
        struct Variables {
            title: String,
            r2d2_password: Vec<i64>,
        }

        let ctx = Variables {
            title: "hello".into(),
            r2d2_password: vec![1, 2, 3, 4],
        };

        let template = Template::from_str(
            "<%= title %><% for digit in r2d2_password %><%= digit %><% end %>",
        )?;

        let result = template.render(&ctx)?;
        assert_eq!(result, "hello1234");

        Ok(())
    }
}
