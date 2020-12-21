use pressure::Pressure;

fn main() -> std::io::Result<()> {
    let pressure = Pressure::new(
        std::env::var("INSTANCE_PATH").unwrap_or(
            std::env::current_dir()
                .unwrap()
                .into_os_string()
                .into_string()
                .unwrap(),
        ),
    );
    pressure.serve("127.0.0.1", 8080)
}
