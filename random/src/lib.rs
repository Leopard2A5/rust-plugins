extern crate reqwest;

use rust_plugins_core::{InvocationError, Function, PluginRegistrar};

fn fetch(request: RequestInfo) -> Result<f64, InvocationError> {
    let url = request.format();
    let response_body = reqwest::get(&url)?.text()?;
    response_body.trim().parse().map_err(Into::into)
}

struct RequestInfo {
    min: i32,
    max: i32
}

impl RequestInfo {
    pub fn format(&self) -> String {
        format!(
            "https://www.random.org/integers/?num=1&min={}&max={}&col=1&base=10&format=plain",
            self.min,
            self.max
        )
    }
}

pub struct Random;

impl Function for Random {
    fn call(
        &self,
        args: &[f64]
    ) -> Result<f64, InvocationError> {
        parse_args(args).and_then(fetch)
    }
}

fn parse_args(args: &[f64]) -> Result<RequestInfo, InvocationError> {
    match args.len() {
        0 => Ok(RequestInfo { min: 0, max: 100 }),
        1 => Ok(RequestInfo {
            min: 0,
            max: args[0].round() as i32,
        }),
        2 => Ok(RequestInfo {
            min: args[0].round() as i32,
            max: args[1].round() as i32,
        }),
        _ => Err("0, 1, or 2 arguments are required".into()),
    }
}

rust_plugins_core::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_function("random", Box::new(Random));
}
