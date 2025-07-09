use std::io::Write;
use std::fs::File;
use env_logger::{Builder, fmt::Color};
use log::LevelFilter;
use msgbox::IconType;

pub fn configure_logging() {
    let target = Box::new(File::create("primemh.log").expect("Can't create bot agent log file"));

    // intercept panics and log an error for them
    std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicHookInfo<'_>| {
        let msg = format!("{}", panic_info);
        log::error!("ERROR: {}", msg.replace("C:\\Users\\mjg99","").replace("mjg99","").replace("panicked at", ""));
    }));

    Builder::new()
        .format(|buf, record| {
            
            let mut path_style = buf.style();
            let mut style = buf.style();
            match record.level() {
                log::Level::Error => style.set_color(Color::Red).set_bold(true),
                log::Level::Warn => style.set_color(Color::Yellow).set_bold(true),
                log::Level::Info => style.set_color(Color::Green).set_bold(true),
                log::Level::Debug => style.set_color(Color::Rgb(128, 128, 128)).set_bold(false),
                log::Level::Trace => style.set_color(Color::Rgb(128, 128, 128)).set_bold(false),
            };

            if record.level() == log::Level::Error {
                let msg: String = format!("{}", &record.args());
                let title = format!("D2R PrimeMH {}", env!("CARGO_PKG_VERSION"));
                msgbox::create(title.as_str(), &msg, IconType::Error).unwrap();
            };

            path_style.set_color(Color::Rgb(128, 128, 128));

            let path = match record.module_path_static() {
                Some(path) => path.replace("d2r_rusty_bot::", ""),
                None => "".to_owned()
            };
            
            writeln!(buf, "{} [{}] {} - {}",
                buf.timestamp(),
                style.value(record.level()),
                path_style.value(path),
                record.args())
        })
        .target(env_logger::Target::Pipe(target))
        .filter(None, LevelFilter::Debug)
        .init();
}