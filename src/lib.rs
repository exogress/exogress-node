use neon::prelude::*;
use futures::channel::mpsc::{UnboundedSender, UnboundedReceiver};
use std::sync::Arc;
use futures::channel::{oneshot, mpsc};
use hashbrown::HashMap;
use exogress_common::entities::{SmolStr, AccountName, ProjectName, LabelValue, LabelName};
use exogress_common::client_core::Client;
use exogress_common::entities::AccessKeyId;
use anyhow::{anyhow, bail};
use tokio::runtime::Runtime;
use trust_dns_resolver::{TokioAsyncResolver, TokioHandle};
use log::{info, LevelFilter};
use std::sync::atomic::{AtomicBool, Ordering};
use log::{Record, Level, Metadata};
use neon::handle::Managed;
use std::str::FromStr;
use neon::result::Throw;
use parking_lot::Once;
use exogress_common::entities::ProfileName;

const CRATE_VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}


pub struct Instance {
    client: parking_lot::Mutex<Option<exogress_common::client_core::Client>>,
    reload_config_tx: parking_lot::Mutex<UnboundedSender<()>>,
    reload_config_rx: Arc<parking_lot::Mutex<Option<UnboundedReceiver<()>>>>,
    stop_tx: parking_lot::Mutex<Option<oneshot::Sender<()>>>,
    stop_rx: Arc<parking_lot::Mutex<Option<oneshot::Receiver<()>>>>,
    has_been_spawned: AtomicBool,
}

impl Finalize for Instance {}

#[derive(Debug)]
struct InstanceConfig {
    access_key_id: AccessKeyId,
    secret_access_key: String,
    project: ProjectName,
    account: AccountName,
    watch_config: Option<bool>,
    profile: Option<ProfileName>,
    config_path: Option<String>,
    labels: HashMap<LabelName, LabelValue>,
}

// Internal implementation
impl Instance {
    fn new<'a, C>(config: InstanceConfig, cx: &mut C) -> anyhow::Result<Self>
        where
            C: Context<'a>,
    {
        LOGGER.call_once(|| {
            log::set_boxed_logger(Box::new(SimpleLogger))
                .map(|()| log::set_max_level(LevelFilter::Info))
                .unwrap();
        });

        let mut client_builder = Client::builder();

        if let Some(config_path) = config.config_path {
            client_builder.config_path(config_path);
        }
        if let Some(watch_config) = config.watch_config {
            client_builder.watch_config(watch_config);
        }

        let client = client_builder
            .access_key_id(config.access_key_id)
            .secret_access_key(config.secret_access_key)
            .account(config.account)
            .project(config.project)
            .profile(config.profile)
            .labels(config.labels)
            .additional_connection_params({
                let mut map = HashMap::<SmolStr, SmolStr>::new();
                map.insert("client".into(), "node".into());
                map.insert("wrapper_version".into(), CRATE_VERSION.into());
                map
            })
            .build()?;

        let (reload_config_tx, reload_config_rx) = mpsc::unbounded();
        let (stop_tx, stop_rx) = oneshot::channel();

        let instance = Instance {
            client: parking_lot::Mutex::new(Some(client)),
            reload_config_tx: parking_lot::Mutex::new(reload_config_tx),
            reload_config_rx: Arc::new(parking_lot::Mutex::new(Some(reload_config_rx))),
            stop_tx: parking_lot::Mutex::new(Some(stop_tx)),
            stop_rx: Arc::new(parking_lot::Mutex::new(Some(stop_rx))),
            has_been_spawned: AtomicBool::new(false),
        };

        Ok(instance)
    }

    fn reload(&self) -> anyhow::Result<()> {
        if self.has_been_spawned.load(Ordering::SeqCst) {
            self.reload_config_tx
                .lock()
                .unbounded_send(())?;
        }

        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        if self.has_been_spawned.load(Ordering::SeqCst) {
            self.stop_tx
                .lock()
                .take()
                .ok_or_else(|| anyhow!("instance already stopped"))?
                .send(())
                .map_err(|_| anyhow!("failed to send reload request"))?;
        }

        Ok(())
    }
}

macro_rules! extract_optional_key {
    ($key:expr, $js_ty:ty, $params:ident, $cx:ident) => {
        if let Ok(key) = $params.get(&mut $cx, $key) {
            if let Ok(v) = key.downcast::<$js_ty, _>(&mut $cx) {
                Some(v.value(&mut $cx))
            } else {
                None
            }
        } else {
            None
        }
    }
}

macro_rules! extract_key {
    ($key:expr, $js_ty:ty, $params:ident, $cx:ident) => {
        $params
            .get(&mut $cx, $key)
            .or_else(|err| $cx.throw_error(err.to_string()))?
            .downcast::<$js_ty, _>(&mut $cx)
            .or_else(|err| $cx.throw_error(err.to_string()))?
            .value(&mut $cx)
    }
}

macro_rules! extract_parsed_key {
    ($key:expr, $ty:ty, $js_ty:ty, $params:ident, $cx:ident) => {
        extract_key!($key, $js_ty, $params, $cx)
            .parse::<$ty>()
            .or_else(|err| $cx.throw_error(err.to_string()))
    }
}

macro_rules! extract_parsed_optional_key {
    ($key:expr, $ty:ty, $js_ty:ty, $params:ident, $cx:ident) => {
        if let Some(k) = extract_optional_key!($key, $js_ty, $params, $cx) {
            Some(k
                .parse::<$ty>()
                .or_else(|err| $cx.throw_error(err.to_string()))?)
        } else {
            None
        }
    }
}



static LOGGER: Once = Once::new();

impl Instance {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<Instance>> {
        let params = cx.argument::<JsObject>(0)?;

        let access_key_id = extract_parsed_key!("access_key_id", AccessKeyId, JsString, params, cx)?;
        let secret_access_key = extract_key!("secret_access_key", JsString, params, cx);
        let account = extract_parsed_key!("account", AccountName, JsString, params, cx)?;
        let project = extract_parsed_key!("project", ProjectName, JsString, params, cx)?;
        let watch_config = extract_optional_key!("watch_config", JsBoolean, params, cx);
        let config_path = extract_optional_key!("config_path", JsString, params, cx);
        let profile = extract_parsed_optional_key!("profile", ProfileName, JsString, params, cx);

        let mut labels = HashMap::new();
        if let Ok(js_labels) = params.get(&mut cx, "labels") {
            let object_js = js_labels
                .downcast::<JsObject, _>(&mut cx)
                .or_else(|err| cx.throw_error(err.to_string()))?;

            let object_keys_js: Handle<JsArray> = object_js
                .get_own_property_names(&mut cx)?;

            let object_keys_rust: Vec<Handle<JsValue>> = object_keys_js.to_vec(&mut cx)?;
            for key in &object_keys_rust {
                let key_value = key.to_string(&mut cx)?.value(&mut cx);
                let item_value = object_js.get(&mut cx, *key)?.to_string(&mut cx)?.value(&mut cx);
                labels.insert(
                    key_value
                        .parse::<LabelName>()
                        .or_else(|err| cx.throw_error(err.to_string()))?
                        .into(),
                    item_value
                        .parse::<LabelValue>()
                        .or_else(|err| cx.throw_error("bad label value"))?
                        .into());
            }
        }


        let config = InstanceConfig {
            access_key_id,
            secret_access_key,
            project,
            account,
            watch_config,
            config_path,
            labels,
            profile,
        };

        println!("instance = {:?}", config);

        let instance = Instance::new(
            config,
            &mut cx)
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.boxed(instance))
    }

    fn js_stop(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<Instance>, _>(&mut cx)?
            .stop()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    fn js_reload(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<Instance>, _>(&mut cx)?
            .reload()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        Ok(cx.undefined())
    }

    fn js_spawn(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        let instance = cx.this().downcast_or_throw::<JsBox<Instance>, _>(&mut cx)?;
        let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
        let queue = cx.queue();

        let rt = Runtime::new()
            .or_else(|err| cx.throw_error(err.to_string()))?;

        let resolver = TokioAsyncResolver::from_system_conf(TokioHandle)
            .or_else(|err| cx.throw_error(err.to_string()))?;

        let reload_config_rx = instance
            .reload_config_rx
            .lock()
            .take()
            .ok_or_else(|| cx.throw_error::<&str, ()>("instance has already been spawned").unwrap_err())?;
        let reload_config_tx = instance.reload_config_tx.lock().clone();

        let stop_rx = instance
            .stop_rx
            .lock()
            .take()
            .ok_or_else(|| cx.throw_error::<&str, ()>("instance has already been spawned").unwrap_err())?;

        let client = instance
            .client
            .lock()
            .take()
            .ok_or_else(|| cx.throw_error::<&str, ()>("cannot start already stopped instance").unwrap_err())?;

        instance.has_been_spawned.store(true, Ordering::SeqCst);

        std::thread::spawn(move || {
            let result = rt.block_on(async move {
                let spawn = client.spawn(reload_config_tx, reload_config_rx, resolver);

                tokio::select! {
                    r = spawn => {
                        r?;
                    },
                    _ = stop_rx => {
                        info!("stop exogress instance by request");
                    }
                }

                Ok::<_, anyhow::Error>(())
            });

            queue.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let args: Vec<Handle<JsValue>> = match result {
                    Ok(()) => vec![cx.null().upcast()],
                    Err(err) => vec![cx.error(err.to_string()).unwrap().upcast()],
                };

                callback.call(&mut cx, this, args)?;

                Ok(())
            });
        });

        Ok(cx.undefined())
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("instanceNew", Instance::js_new)?;
    cx.export_function("instanceStop", Instance::js_stop)?;
    cx.export_function("instanceReload", Instance::js_reload)?;
    cx.export_function("instanceSpawn", Instance::js_spawn)?;

    Ok(())
}