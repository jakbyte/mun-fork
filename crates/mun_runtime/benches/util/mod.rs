use mlua::Lua;
use mun_compiler::{Config, Driver, OptimizationLevel, PathOrInline};
use mun_runtime::RuntimeBuilder;
use std::cell::RefCell;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use termcolor::NoColor;
use wasmer_runtime::{instantiate, Instance};

fn compute_resource_path<P: AsRef<Path>>(p: P) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches/resources/")
        .join(p)
}

pub fn runtime_from_file<P: AsRef<Path>>(p: P) -> Rc<RefCell<mun_runtime::Runtime>> {
    let path = PathOrInline::Path(compute_resource_path(p));
    let (mut driver, file_id) = Driver::with_file(
        Config {
            optimization_lvl: OptimizationLevel::Aggressive,
            ..Config::default()
        },
        path,
    )
    .unwrap();
    let mut cursor = NoColor::new(Cursor::new(Vec::new()));
    if driver.emit_diagnostics(&mut cursor).unwrap() {
        let errors = String::from_utf8(cursor.into_inner().into_inner())
            .unwrap_or_else(|e| format!("<could not utf8 decode error string: {}>", e));
        panic!("compiler errors..\n{}", errors);
    }

    let out_path = driver.write_assembly(file_id).unwrap();
    Rc::new(RefCell::new(RuntimeBuilder::new(out_path).spawn().unwrap()))
}

pub fn lua_from_file<P: AsRef<Path>>(p: P) -> Lua {
    let lua = Lua::new();
    lua.load(&std::fs::read_to_string(compute_resource_path(p)).unwrap())
        .exec()
        .unwrap();
    lua
}

pub fn wasmer_from_file<P: AsRef<Path>>(p: P) -> Instance {
    let wasm_content = std::fs::read(compute_resource_path(p)).unwrap();
    let import_objects = wasmer_runtime::imports! {};
    instantiate(&wasm_content, &import_objects).unwrap()
}
