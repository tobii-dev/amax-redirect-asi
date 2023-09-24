mod redirects;

use blur_plugins_core::{BlurAPI, BlurEvent, BlurPlugin};
use log::LevelFilter;
use simplelog::{
	ColorChoice, CombinedLogger, Config, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};

#[repr(C)]
struct MyRedirectsPlugin {}

impl BlurPlugin for MyRedirectsPlugin {
	fn name(&self) -> &'static str {
		"AmaxRedirectorPlugin"
	}

	fn on_event(&self, _event: &BlurEvent) {}

	fn free(&self) {}
}

#[no_mangle]
fn plugin_init(_api: &mut dyn BlurAPI) -> Box<dyn BlurPlugin> {
	let plugin = MyRedirectsPlugin {};

	let cfg = ConfigBuilder::new()
		.set_time_offset_to_local()
		.unwrap()
		.build();
	let log_path = std::format!(".\\amax\\log\\{}.log", plugin.name());
	let log_file = std::fs::File::create(&log_path).unwrap_or_else(|_| {
		panic!("Couldn't create log file: {log_path}");
	});
	CombinedLogger::init(vec![
		TermLogger::new(
			LevelFilter::Trace,
			cfg,
			TerminalMode::Mixed,
			ColorChoice::Auto,
		),
		WriteLogger::new(LevelFilter::Trace, Config::default(), log_file),
	])
	.unwrap();
	log_panics::init();

	redirects::init();

	Box::new(plugin)
}
