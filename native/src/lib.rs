use neon::prelude::*;

struct BackgroundTask {
    access_key_id: String,
    secret_access_key: String,
    account: String,
    project: String,
}

impl Task for BackgroundTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        exogress_client_lib::spawn(self.access_key_id.clone(),
                                   self.secret_access_key.clone(),
                                   self.account.clone(),
                                   self.project.clone())
            .map_err(|e| {
                e.to_string()
            })
    }

    fn complete(self, mut cx: TaskContext, result: Result<(), String>) -> JsResult<JsUndefined> {
        match result {
            Err(s) => cx.throw_error(s),
            Ok(()) => Ok(cx.undefined()),
        }
    }
}

fn spawn(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let js_object_handle: Handle<JsObject> = cx.argument(0)?;
    let js_object = js_object_handle
        .downcast::<JsObject>()
        .unwrap_or(JsObject::new(&mut cx));

    let access_key_id = js_object
        .get(&mut cx, "access_key_id")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("access_key_id should be set"))?;
    let secret_access_key = js_object
        .get(&mut cx, "secret_access_key")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("secret_access_key should be set"))?;
    let account = js_object
        .get(&mut cx, "account")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("account should be set"))?;
    let project = js_object
        .get(&mut cx, "project")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("project should be set"))?;
    let threads = js_object
        .get(&mut cx, "threads")?
        .downcast::<JsNumber>()
        .ok();

    let f = cx.argument::<JsFunction>(1)?;
    BackgroundTask {
        access_key_id: access_key_id.value(),
        secret_access_key: secret_access_key.value(),
        account: account.value(),
        project: project.value(),
    }.schedule(f);
    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("spawn", spawn)
});
