mod dbus;
mod qt;

use std::{cell::RefCell, rc::Rc};

use futures_util::stream::StreamExt;
use qtbridge::{QApp, QmlObject, qobject};
use zbus::Connection;

use crate::{dbus::{ManagerProxy, Unit}, qt::UnitTableModel};

#[derive(Default)]
struct Backend {
    manager_proxy: Option<ManagerProxy<'static>>,
    units: Rc<RefCell<UnitTableModel>>
}

#[qobject(Singleton)]
impl Backend {
    qproperty!("units", Member = units, Notify = units_changed);

    #[qsignal]
    fn units_changed(&mut self);

    #[qslot]
    fn init(&self) {
        println!("Connecting to D-Bus");

        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            let conn = match Connection::system().await {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to connect to D-Bus: {:?}", e);
                    return;
                },
            };

            let proxy = match ManagerProxy::new(&conn).await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to create proxy: {:?}", e);
                    return;
                },
            };

            println!("D-Bus connected");
            handle.invoke_mut(move |backend| {
                backend.manager_proxy = Some(proxy);
                backend.reload_units();
                backend.start_job_listener();
            });
        });
    }
    
    #[qslot]
    fn start_job_listener(&self) {
        let proxy = self.manager_proxy.clone().unwrap();
        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            let mut stream = proxy.receive_job_removed().await.unwrap();
            while let Some(signal) = stream.next().await {
                let args = match signal.args() {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to get args from signal: {:?}", e);
                        continue;
                    }
                };

                println!("{:?}", args);

                handle.invoke(move |backend| {
                    backend.reload_units();
                });
            }
        });
    }

    #[qslot]
    fn reload_units(&self) {
        println!("Reloading units");

        let proxy = self.manager_proxy.clone().unwrap();
        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            let list = match proxy.list_units().await {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Failed to list units: {:?}", e);
                    return;
                },
            };

            let mut units = list
                .into_iter()
                .map(|u| Unit {
                    name: u.0,
                    description: u.1,
                    load_state: u.2,
                    active_state: u.3,
                    sub_state: u.4,
                    followed_unit: u.5,
                    unit_object_path: u.6.to_string(),
                    job_id: u.7,
                    job_type: u.8,
                    job_object_path: u.9.to_string()
                }).collect::<Vec<_>>();
            units.sort_by_key(|u| u.name.clone());

            handle.invoke_mut(move |backend| {
                println!("Loaded {} units", units.len());
                backend.units = Rc::new(RefCell::new(UnitTableModel(units)));
                backend.units_changed();
            });
        });
    }

    #[qslot]
    fn can_start(&self, row: usize) -> bool {
        self.units.borrow().0.get(row).is_some_and(|u| u.active_state == "inactive")
    }

    #[qslot]
    fn start_unit(&self, row: usize) {
        let unit = match self.units.borrow().0.get(row) {
            Some(u) => u.clone(),
            None => {
                eprintln!("Tried to start unit at row {} but it does not exist in units list", row);
                return;
            },
        };

        let proxy = self.manager_proxy.clone().unwrap();
        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            if proxy.start_unit(unit.name, "replace".to_string()).await.is_ok() {
                handle.invoke(move |backend| {
                    backend.reload_units();
                });
            }
        });
    }

    #[qslot]
    fn can_stop(&self, row: usize) -> bool {
        self.units.borrow().0.get(row).is_some_and(|u| u.active_state == "active")
    }

    #[qslot]
    fn stop_unit(&self, row: usize) {
        let unit = match self.units.borrow().0.get(row) {
            Some(u) => u.clone(),
            None => {
                eprintln!("Tried to stop unit at row {} but it does not exist in units list", row);
                return;
            },
        };

        let proxy = self.manager_proxy.clone().unwrap();
        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            if proxy.stop_unit(unit.name, "replace".to_string()).await.is_ok() {
                handle.invoke(move |backend| {
                    backend.reload_units();
                });
            }
        });
    }

    #[qslot]
    fn restart_unit(&self, row: usize) {
        let unit = match self.units.borrow().0.get(row) {
            Some(u) => u.clone(),
            None => {
                eprintln!("Tried to restart unit at row {} but it does not exist in units list", row);
                return;
            },
        };

        let proxy = self.manager_proxy.clone().unwrap();
        let handle = self.get_thread_handle();
        tokio::spawn(async move {
            if proxy.restart_unit(unit.name, "replace".to_string()).await.is_ok() {
                handle.invoke(move |backend| {
                    backend.reload_units();
                });
            }
        });
    }
}

#[tokio::main]
async fn main() {
    QApp::new()
        .register::<Backend>()
        .load_qml(include_bytes!("qml/Main.qml"))
        .run();
}
