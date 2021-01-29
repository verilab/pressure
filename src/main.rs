use env_logger::Env;
use pressure::*;

fn main() -> PressResult<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let instance = Instance::new(
        std::env::var("PRESSURE_INSTANCE").unwrap_or(
            std::env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        ),
    )?;
    serve(instance, "127.0.0.1", 8080)
}
