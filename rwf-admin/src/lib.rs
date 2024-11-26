use rwf::controller::{Engine, StaticFiles};
use rwf::http::{Handler, Path};
use rwf::prelude::*;

mod controllers;
use controllers::*;

mod models;

pub fn routes() -> Result<Vec<Handler>, Error> {
    Ok(vec![engine!("/admin" => engine()), static_files()?])
}

pub fn engine() -> Engine {
    Engine::new(vec![
        route!("/" => index::Index),
        route!("/jobs" => jobs::Jobs),
        route!("/requests" => requests::Requests),
        route!("/models" => controllers::models::ModelsController),
        route!("/models/model" => controllers::models::ModelController),
        route!("/models/new" => controllers::models::NewModelController),
    ])
    .remount(&Path::parse("/admin").unwrap())
}

pub fn install() -> Result<(), Error> {
    use rwf::view::Templates;

    Templates::cache().preload_str(
        "templates/rwf_admin/requests.html",
        include_str!("../templates/rwf_admin/requests.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/model_pages.html",
        include_str!("../templates/rwf_admin/model_pages.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/jobs.html",
        include_str!("../templates/rwf_admin/jobs.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/head.html",
        include_str!("../templates/rwf_admin/head.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/nav.html",
        include_str!("../templates/rwf_admin/nav.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/model.html",
        include_str!("../templates/rwf_admin/model.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/model_new.html",
        include_str!("../templates/rwf_admin/model_new.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/reload.html",
        include_str!("../templates/rwf_admin/reload.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/models.html",
        include_str!("../templates/rwf_admin/models.html"),
    )?;
    Templates::cache().preload_str(
        "templates/rwf_admin/footer.html",
        include_str!("../templates/rwf_admin/footer.html"),
    )?;

    Ok(())
}

pub fn static_files() -> Result<Handler, Error> {
    let static_files = StaticFiles::new("static")?.prefix("static/rwf_admin");
    let static_files = static_files
        .preload(
            "/static/rwf_admin/images/logo.svg",
            include_bytes!("../static/rwf_admin/images/logo.svg"),
        )
        .preload(
            "/static/rwf_admin/js/bootstrap.min.js",
            include_bytes!("../static/rwf_admin/js/bootstrap.min.js"),
        )
        .preload(
            "/static/rwf_admin/js/reload_controller.js",
            include_bytes!("../static/rwf_admin/js/reload_controller.js"),
        )
        .preload(
            "/static/rwf_admin/js/requests_controller.js",
            include_bytes!("../static/rwf_admin/js/requests_controller.js"),
        )
        .preload(
            "/static/rwf_admin/js/bootstrap.min.js.map",
            include_bytes!("../static/rwf_admin/js/bootstrap.min.js.map"),
        )
        .preload(
            "/static/rwf_admin/js/popper.min.js",
            include_bytes!("../static/rwf_admin/js/popper.min.js"),
        )
        .preload(
            "/static/rwf_admin/css/bootstrap.min.css.map",
            include_bytes!("../static/rwf_admin/css/bootstrap.min.css.map"),
        )
        .preload(
            "/static/rwf_admin/css/bootstrap.min.css",
            include_bytes!("../static/rwf_admin/css/bootstrap.min.css"),
        );

    Ok(static_files.handler())
}
