use neon::prelude::*;

struct BackgroundTask {
    client_id: String,
    client_secret: String,
    account: String,
    project: String,
}

impl Task for BackgroundTask {
    type Output = ();
    type Error = String;
    type JsEvent = JsUndefined;

    fn perform(&self) -> Result<(), String> {
        exogress_client_lib::spawn(self.client_id.clone(),
                                   self.client_secret.clone(),
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

    let client_id = js_object
        .get(&mut cx, "client_id")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("client_id should be set"))?;
    let client_secret = js_object
        .get(&mut cx, "client_secret")?
        .downcast::<JsString>()
        .or_else(|_| cx.throw_error("client_secret should be set"))?;
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
        client_id: client_id.value(),
        client_secret: client_secret.value(),
        account: account.value(),
        project: project.value(),
    }.schedule(f);
    Ok(cx.undefined())
}

register_module!(mut cx, {
    cx.export_function("spawn", spawn)
});
