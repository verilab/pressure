use pressure::*;

fn main() -> PressResult<()> {
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
