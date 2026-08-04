use color_eyre::eyre::Result;
use gtk4::{
    ApplicationWindow, Builder,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    prelude::GtkWindowExt,
};
use libadwaita::Application;

const APP_ID: &str = "org.luxvao.pd2lcp";

fn main() -> Result<()> {
    color_eyre::install()?;

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();

    Ok(())
}

fn build_ui(app: &Application) {
    let builder = Builder::from_string(include_str!("../PD2LCP.ui"));

    let window: ApplicationWindow = builder.object("MainWindow").expect("Couldn't find object");

    window.set_application(Some(app));

    window.set_resizable(false);

    window.present();
}
