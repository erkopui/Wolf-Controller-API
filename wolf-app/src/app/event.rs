use crate::App;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

type Callback = Arc<dyn Fn(&App, &str, &Value) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct Event {
    store: HashMap<String, Callback>,
}

impl Event {
    pub fn new() -> Event {
        let store: HashMap<String, Callback> = HashMap::new();
        Event { store }
    }

    pub fn subscribe(&mut self, path: String, f: Callback) -> &mut Event {
        self.store.insert(path, f);
        self
    }

    pub fn emit(app: &App, path: String, c: &Value) {
        if let Value::Object(c) = c {
            for (k, v) in c {
                let p = format!("{}/{}", &path, &k);
                crate::app::Event::emit(app, p, v);
            }
        } else {
            if let Some(f) = app.event.store.get(&path) {
                f(&app, path.as_str(), &c);
            }
            println!("new event p:{} d:{}", path, &c);
        }
    }
}
