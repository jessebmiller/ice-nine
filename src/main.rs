extern crate termion;

use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin, Stdin};
use termion::event::Key;
use termion::input::{TermRead, Keys};

use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::sync::{Arc, Mutex};

fn publish<I, T>(iter: I) -> Sender<Sender<T>> where
    I: Iterator<Item=T> + Send + 'static,
    T: Copy + Send + 'static{

    // publish Ts from iter to all subscribers
    let shared_subscriptions: Arc<Mutex<Vec<Sender<T>>>> = Arc::new(
        Mutex::new(vec![]),
    );

    let subscriptions = shared_subscriptions.clone();
    thread::spawn(move || {
        for e in iter {
            let subs = subscriptions.lock().unwrap();
            for s in subs.iter() {
                s.send(e.clone());
            }
        }
    });

    // accept subscriptions (senders) over a channel
    let (tx, rx) = channel();
    let subscription_listener = shared_subscriptions.clone();
    thread::spawn(move || {
        for sub in rx.iter() {
            let mut subscriptions = subscription_listener.lock().unwrap();
            subscriptions.push(sub);
        }
    });
    return tx;
}

fn subscribe<T>(publisher: &Sender<Sender<T>>) -> Receiver<T> {
    let (tx, rx) = channel();
    publisher.send(tx);
    return rx;
}

fn parse_command(k: Key) -> String {
    match k {
        Key::Char('q') => panic!(),
        Key::Char(c) => c.to_string(),
        _ => "Other".to_string(),
    }
}

fn main() {
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let commands = publish(stdin.keys().map(|k| {k.unwrap()}));

    let sub1 = subscribe(&commands);
    thread::spawn(move || {
        for k in sub1.iter() {
            println!("one-{}", parse_command(k));
        }
    });

    let sub2 = subscribe(&commands);
    for k in sub2.iter() {
        println!("two-{}", parse_command(k));
    }
}
