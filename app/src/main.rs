use rust_plugins_core::{Function, InvocationError, PluginDeclaration};
use std::rc::Rc;
use libloading::Library;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::path::PathBuf;
use std::alloc::System;
use std::env;

#[global_allocator]
static ALLOCATOR: System = System;

fn main() {
    let args = env::args().skip(1);
    let args = Args::parse(args)
        .expect("Usage: app <plugin-path> <function> <args>...");

    let mut functions = ExternalFunctions::new();
    unsafe {
        functions
            .load(&args.plugin_library)
            .expect("Function loading failed");
    }

    let result = functions
        .call(&args.function, &args.arguments)
        .expect("Invocation failed");

    println!(
        "{}({}) = {}",
        args.function,
        args.arguments
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        result
    );
}

struct Args {
    plugin_library: PathBuf,
    function: String,
    arguments: Vec<f64>,
}

impl Args {
    fn parse(mut args: impl Iterator<Item = String>) -> Option<Args> {
        let plugin_library = PathBuf::from(args.next()?);
        let function = args.next()?;
        let mut arguments = Vec::new();

        for arg in args {
            arguments.push(arg.parse().ok()?);
        }

        Some(Args {
            plugin_library,
            function,
            arguments,
        })
    }
}

#[derive(Default)]
pub struct ExternalFunctions {
    functions: HashMap<String, FunctionProxy>,
    libraries: Vec<Rc<Library>>
}

impl ExternalFunctions {
    pub fn new() -> Self {
        ExternalFunctions::default()
    }

    pub fn call(
        &self,
        function: &str,
        args: &[f64]
    ) -> Result<f64, InvocationError> {
        self.functions
            .get(function)
            .ok_or_else(|| format!("\"{}\"", function))?
            .call(args)
    }

    pub unsafe fn load<P: AsRef<OsStr>>(
        &mut self,
        library_path: P
    ) -> io::Result<()> {
        let library = Rc::new(Library::new(library_path)?);

        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        if decl.rustc_version != rust_plugins_core::RUSTC_VERSION
            || decl.core_version != rust_plugins_core::CORE_VERSION
        {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Version mismatch"
            ));
        }

        let mut registrar = PluginRegistrar::new(library.clone());
        (decl.register)(&mut registrar);

        self.functions.extend(registrar.functions);
        self.libraries.push(library);

        Ok(())
    }
}

pub struct FunctionProxy {
    function: Box<dyn Function>,
    _lib: Rc<Library>
}

impl Function for FunctionProxy {
    fn call(&self, args: &[f64]) -> Result<f64, InvocationError> {
        self.function.call(args)
    }

    fn help(&self) -> Option<&str> {
        self.function.help()
    }
}

struct PluginRegistrar {
    functions: HashMap<String, FunctionProxy>,
    lib: Rc<Library>
}

impl PluginRegistrar {
    fn new(lib: Rc<Library>) -> Self {
        PluginRegistrar {
            lib,
            functions: HashMap::default()
        }
    }
}

impl rust_plugins_core::PluginRegistrar for PluginRegistrar {
    fn register_function(
        &mut self,
        name: &str,
        function: Box<dyn Function>
    ) {
        let proxy = FunctionProxy {
            function,
            _lib: self.lib.clone()
        };
        self.functions.insert(name.to_string(), proxy);
    }
}
