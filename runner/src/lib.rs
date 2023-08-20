use std::{rc::Rc, sync::Arc};

use deno_core::v8::{HandleScope, Value};
use deno_runtime::{
    deno_core::{error::AnyError, ModuleSpecifier},
    deno_napi::v8::{self, DataError, GetPropertyNamesArgs, Global, Local, Promise, PromiseState},
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
};
use module_loader::TsModuleLoader;
use print_ext::print_extension;
use thiserror::Error;

mod module_loader;
mod print_ext;

pub use print_ext::{Printer, SimplePrinter};

fn serde_json_value_to_v8<'a>(
    scope: &mut HandleScope<'a>,
    value: &serde_json::Value,
) -> Local<'a, Value> {
    match value {
        serde_json::Value::Null => v8::null(scope).into(),
        serde_json::Value::Bool(b) => v8::Boolean::new(scope, *b).into(),
        serde_json::Value::Number(n) => v8::Number::new(scope, n.as_f64().unwrap()).into(),
        serde_json::Value::String(s) => v8::String::new(scope, s).unwrap().into(),
        serde_json::Value::Array(a) => {
            let arr: Local<'a, v8::Array> = v8::Array::new(scope, a.len() as i32);
            for (i, elem) in a.iter().enumerate() {
                let value = serde_json_value_to_v8(scope, elem);
                arr.set_index(scope, i as u32, value);
            }
            arr.into()
        }
        serde_json::Value::Object(obj) => {
            let object: Local<'a, v8::Object> = v8::Object::new(scope);
            for (key, value) in obj {
                let key = v8::String::new(scope, key).unwrap().into();
                let value = serde_json_value_to_v8(scope, value);
                object.set(scope, key, value);
            }
            object.into()
        }
    }
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error(transparent)]
    AnyError(#[from] AnyError),
    #[error("Stage {stage} is not a function: {error}")]
    StageIsNotFunction {
        stage: String,
        #[source]
        error: Arc<DataError>,
    },
    #[error("No default export")]
    NoDefaultExport,
    #[error("Default export is not an object")]
    DefaultExportIsNotObject,
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeV8Error(#[from] deno_core::serde_v8::Error),
}

pub struct RunParams<P: Printer + 'static> {
    pub main_module: ModuleSpecifier,
    pub printer: P,
    pub params: Vec<serde_json::Value>,
}

#[allow(clippy::future_not_send)]
pub async fn run<P: Printer + 'static>(params: RunParams<P>) -> Result<(), RunnerError> {
    let main_module = params.main_module;
    let mut worker = MainWorker::bootstrap_from_options(
        main_module.clone(),
        PermissionsContainer::allow_all(),
        WorkerOptions {
            module_loader: Rc::new(TsModuleLoader),
            extensions: vec![print_extension::init_ops_and_esm(params.printer)],
            ..Default::default()
        },
    );
    let main_module = worker.preload_main_module(&main_module).await?;
    worker.evaluate_module(main_module).await?;
    worker.run_event_loop(false).await?;
    let global = worker.js_runtime.get_module_namespace(main_module)?;

    let stages = {
        let scope = &mut worker.js_runtime.handle_scope();
        let global = global.open(scope);
        let default_str = v8::String::new(scope, "default").unwrap();
        let stages = global
            .get(scope, default_str.into())
            .ok_or(RunnerError::NoDefaultExport)?
            .to_object(scope)
            .ok_or(RunnerError::DefaultExportIsNotObject)?;
        Global::new(scope, stages)
    };
    let stage_names = {
        let scope = &mut worker.js_runtime.handle_scope();
        let stages: &v8::Object = stages.open(scope);
        let stage_names = stages
            .get_property_names(scope, GetPropertyNamesArgs::default())
            .unwrap();
        Global::new(scope, stage_names)
    };
    let stage_names = stage_names.open(worker.js_runtime.v8_isolate());
    let func = stages.open(worker.js_runtime.v8_isolate());
    let stage_count = stage_names.length();
    let mut result = None;
    for i in 0..stage_count {
        let func_res = {
            let scope = &mut worker.js_runtime.handle_scope();
            let name = stage_names.get_index(scope, i).unwrap();
            let name_string = name.to_rust_string_lossy(scope);
            let func = func.get(scope, name).unwrap();
            let func: v8::Local<v8::Function> =
                func.try_into()
                    .map_err(|error| RunnerError::StageIsNotFunction {
                        stage: name_string,
                        error: Arc::new(error),
                    })?;
            let recv = Local::new(scope, &global);
            let args = params
                .params
                .iter()
                // .inspect(|v| {
                //     dbg!(v);
                // })
                .map(|value| serde_json_value_to_v8(scope, value))
                // .inspect(|v| {
                //     dbg!(v);
                // })
                .collect::<Vec<_>>();
            let res = func.call(scope, recv.into(), &args).unwrap();
            Global::new(scope, res)
        };
        worker.js_runtime.run_event_loop(false).await?;
        let scope = &mut worker.js_runtime.handle_scope();
        let mut res = Local::new(scope, func_res);
        if res.is_promise() {
            let promise: Local<Promise> = res.try_into().unwrap();
            if promise.state() == PromiseState::Pending {
                panic!("Resulting promise is still pending")
            }
            if promise.has_handler() {
                panic!("Promise has handler")
            }
            res = promise.result(scope);
            // promise.
        }
        if i + 1 == stage_count {
            result = Some(Global::new(scope, res));
        }
    }
    if let Some(res) = result {
        let scope = &mut worker.js_runtime.handle_scope();
        let res = res.open(scope);
        println!("{}", res.to_rust_string_lossy(scope));
    }
    Ok(())
}
