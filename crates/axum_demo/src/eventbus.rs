#![allow(unused)]

use std::{
    collections::{HashMap, LinkedList},
    sync::{Arc, Condvar, Mutex, RwLock, atomic::AtomicU64},
    thread,
};

const SLOT_DEFAULT: &str = "event_slot_default";
type Handler = Box<dyn Fn(Vec<u8>) + Sync + Send + 'static>;

pub struct EventBus {
    event_slots: RwLock<HashMap<String, String>>,
    slot_queues: RwLock<HashMap<String, Arc<EventQueue>>>,
    event_handlers: RwLock<HashMap<String, Handler>>,
    dispatch_count: AtomicU64,
    receive_count: AtomicU64,
}

struct Event {
    name: String,
    data: Vec<u8>,
}

struct EventQueue {
    queue: Mutex<LinkedList<Event>>,
    signal: Condvar,
}

impl EventQueue {
    fn new() -> Self {
        EventQueue {
            queue: Mutex::new(LinkedList::new()),
            signal: Condvar::new(),
        }
    }

    fn queue_event(&self, event: Event) {
        self.queue
            .lock()
            .expect("eventbus queue event failed")
            .push_back(event);
        self.signal.notify_one();
    }

    fn pull_event(&self) -> Event {
        let mut guard = self.queue.lock().expect("eventbus pull event failed");
        if let Some(event) = guard.pop_front() {
            return event;
        }
        loop {
            guard = self
                .signal
                .wait(guard)
                .expect("eventbus wait signal failed");
            if let Some(event) = guard.pop_front() {
                return event;
            }
        }
    }
}

impl EventBus {
    pub fn new() -> Self {
        EventBus {
            event_slots: RwLock::new(HashMap::new()),
            slot_queues: RwLock::new(HashMap::new()),
            event_handlers: RwLock::new(HashMap::new()),
            dispatch_count: AtomicU64::new(0),
            receive_count: AtomicU64::new(0),
        }
    }

    pub fn slot_register<F>(self: &Arc<Self>, slot: &str, event: &str, f: F) -> &Arc<Self>
    where
        F: Fn(Vec<u8>) + 'static + Send + Sync,
    {
        self.event_slots
            .write()
            .unwrap()
            .insert(event.to_string(), slot.to_string());

        self.event_handlers
            .write()
            .unwrap()
            .insert(event.to_string(), Box::new(f));

        let mut slot_queues = self
            .slot_queues
            .write()
            .expect("eventbus slot_queue lock failed");
        if slot_queues.get(slot).is_some() {
            return self;
        }

        let queue = Arc::new(EventQueue::new());
        slot_queues.insert(slot.to_string(), queue.clone());

        let _ = thread::Builder::new().name(slot.to_string()).spawn({
            let event_bus = self.clone();
            move || loop {
                let event = queue.pull_event();
                if let Some(handler) = event_bus
                    .event_handlers
                    .read()
                    .expect("eventbus event_handlers lock failed")
                    .get(&event.name)
                {
                    let count = event_bus
                        .receive_count
                        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    log::debug!("eventbus receive_count:{}", count);
                    handler(event.data);
                }
            }
        });

        self
    }

    pub fn register<F>(self: &Arc<Self>, event: &str, f: F) -> &Arc<Self>
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        self.slot_register(SLOT_DEFAULT, event, f)
    }

    pub fn dispatch(&self, event: &str, data: Vec<u8>) {
        let dispatch_count = self
            .dispatch_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        log::debug!("eventbus dispatch_count:{}", dispatch_count);
        if let Some(slot) = self
            .event_slots
            .read()
            .expect("eventbus lock failed")
            .get(event)
        {
            if let Some(slot_queues) = self
                .slot_queues
                .read()
                .expect("eventbus lock failed")
                .get(slot)
            {
                slot_queues.queue_event(Event {
                    name: event.to_string(),
                    data,
                });
            }
        }
    }
}
