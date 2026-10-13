use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::windows::EventLoopBuilderExtWindows,
};

pub fn run_tray(shutdown_tx: tokio::sync::watch::Sender<bool>) {
    let event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()
        .expect("Failed to create event loop");

    let mut app = TrayApplication {
        tray: None,
        open_item: None,
        quit_item: None,
        shutdown_tx,
    };

    event_loop
        .run_app(&mut app)
        .expect("Tray event loop failed");
}

struct TrayApplication {
    tray: Option<tray_icon::TrayIcon>,
    open_item: Option<MenuItem>,
    quit_item: Option<MenuItem>,
    shutdown_tx: tokio::sync::watch::Sender<bool>,
}

impl ApplicationHandler for TrayApplication {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        let icon = create_icon();

        let menu = Menu::new();

        let open_item = MenuItem::new("Open NowPlaying", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        menu.append(&open_item).expect("Failed to add menu item");

        menu.append(&PredefinedMenuItem::separator())
            .expect("Failed to add separator");

        menu.append(&quit_item).expect("Failed to add menu item");

        self.tray = Some(
            TrayIconBuilder::new()
                .with_tooltip("NowPlaying")
                .with_icon(icon)
                .with_menu(Box::new(menu))
                .build()
                .expect("Failed to create tray icon"),
        );

        self.open_item = Some(open_item);
        self.quit_item = Some(quit_item);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: WindowEvent,
    ) {
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if let Some(open_item) = &self.open_item {
                if event.id() == open_item.id() {
                    open_nowplaying();
                }
            }

            if let Some(quit_item) = &self.quit_item {
                if event.id() == quit_item.id() {
                    let _ = self.shutdown_tx.send(true);
                }
            }
        }
    }
}

fn open_nowplaying() {
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", "http://localhost:3000"])
        .spawn();
}

fn create_icon() -> Icon {
    let width = 32;
    let height = 32;

    let rgba = include_bytes!("../overlay/assets/trayicon.rgba").to_vec();

    Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon")
}
