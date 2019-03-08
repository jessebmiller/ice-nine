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
            let mut subs = subscription_listener.lock().unwrap();
            subs.push(sub);
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
    let commands = publish(stdin.keys().map(|k| {k.unwrap()}));

    let sub1 = subscribe(&commands);
    let mut out1 = stdout().into_raw_mode().unwrap();
    thread::spawn(move || {
        for k in sub1.iter() {
            write!(&mut out1, "one-{}\r\n", parse_command(k));
            out1.flush().unwrap();
        }
    });

    let sub2 = subscribe(&commands);
    let mut out2 = stdout().into_raw_mode().unwrap();
    for k in sub2.iter() {
        write!(&mut out2, "two-{}\r\n", parse_command(k));
        out2.flush().unwrap();
    }
}
