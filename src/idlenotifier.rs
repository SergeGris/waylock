use std::rc::Rc;

use wayland_client::{
    Connection,
    Dispatch,
    EventQueue,
    Proxy,
    QueueHandle,
    globals::registry_queue_init,
    protocol::{wl_registry::WlRegistry, wl_seat::WlSeat},
};
use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{Event as IdleEvent, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};

// Application state, which will implement Dispatch for the relevant Wayland objects.
struct IdleClient {
    idle_notifier: Option<ExtIdleNotifierV1>,
    seat: Option<WlSeat>,
}

impl IdleClient {
    fn new() -> Self {
        Self {
            idle_notifier: None,
            seat: None,
        }
    }

    // Creates an idle notification on the seat with the given timeout.
    // Returns the proxy object for the notification.
    fn create_notification<D>(
        &self,
        qh: &QueueHandle<D>,
        timeout_ms: u32,
    ) -> Option<ExtIdleNotificationV1>
    where
        D: Dispatch<ExtIdleNotificationV1, ()> + 'static,
    {
        let notifier = self.idle_notifier.as_ref()?;
        let seat = self.seat.as_ref()?;
        Some(notifier.get_idle_notification(timeout_ms, seat, qh, ()))
    }
}

// Implement Dispatch for registry to catch global objects
impl Dispatch<WlRegistry, ()> for IdleClient {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wayland_client::protocol::wl_registry::Event,
        _: &(),
        conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        use wayland_client::protocol::wl_registry::Event;

        match event {
            Event::Global {
                name,
                interface,
                version,
            } => {
                // Bind the idle-notify global when announced
                if interface == ExtIdleNotifierV1::interface().name {
                    let notifier = registry.bind::<ExtIdleNotifierV1, _, ()>(name, version, qh, ());
                    //log::info!("Bound ExtIdleNotifierV1: {:?}", notifier);
                    state.idle_notifier = Some(notifier);
                }
                // Bind a seat if we need it
                else if interface == WlSeat::interface().name {
                    let seat = registry.bind::<WlSeat, _, ()>(name, version, qh, ());
                    //log::info!("Bound WlSeat: {:?}", seat);
                    state.seat = Some(seat);
                }
            }
            // Handle global removal if needed
            Event::GlobalRemove { name } => {
                //log::warn!("Global removed: id={}", name);
            }
        }
    }
}

// Dispatch trait for idle notification object
impl Dispatch<ExtIdleNotificationV1, ()> for IdleClient {
    fn event(
        _state: &mut Self,
        _proxy: &ExtIdleNotificationV1,
        event: IdleEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            IdleEvent::Idled => {
                // log::info!("Received idled event: user is idle");
                // React: e.g. lock screen, trigger action, etc.
            }
            IdleEvent::Resumed => {
                //log::info!("Received resumed event: user resumed activity");
                // React: user came back
            }
            _ => { /* no other events in this protocol */ }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for debug
    // env_logger::init();

    // Connect to Wayland compositor
    let conn = Connection::connect_to_env()?;
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // Setup state and registry
    let mut state = IdleClient::new();
    let (globals, _) = registry_queue_init::<IdleClient>(&conn).expect("Failed to init registry");

    // Do a roundtrip so we have all globals
    event_queue.roundtrip(&mut state)?;

    // Check we got the notifier and seat
    if state.idle_notifier.is_none() {
        //log::error!("No ext-idle-notify manager available");
        // You can decide to exit or fallback
    }
    if state.seat.is_none() {
        //log::error!("No wl_seat available; idle notifications may not work");
    }

    // Create an idle notification: for example, 10 seconds timeout
    let timeout_ms = 10_000;
    let idle_notification = state
        .create_notification(&qh, timeout_ms)
        .expect("Failed to create idle notification");

    // Assign a dispatch for the notification
    idle_notification.assign(move |_notif, event, _| {
        match event {
            IdleEvent::Idled => {
                //log::info!("(quick_assign) Idle event");
            }
            IdleEvent::Resumed => {
                //log::info!("(quick_assign) Resumed event");
            }
            _ => {}
        }
    });

    // Main event loop: dispatch Wayland events and handle idle events
    loop {
        event_queue.dispatch(&mut state, |_, _, _| {}).unwrap();
        // You could sleep or do other work here between dispatch calls
    }
}
