use self::network::Network;
use self::nic::{Nat, Nic};
use crate::intcode::IntCode;
use crate::util::parse_intcode;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

const NUM_CPU: usize = 50;
const SLEEP_INTERVAL: std::time::Duration = std::time::Duration::from_millis(1);

pub fn part1(input: &str) -> Result<String> {
    let (rx, mut shutdown) = run_network(input)?;
    let first = rx.recv()?;
    shutdown.shutdown()?;
    Ok(format!("{:?}", first))
}
pub fn part2(input: &str) -> Result<String> {
    let (rx, mut shutdown) = run_network(input)?;
    let mut last = None;
    let double = loop {
        let next = rx.recv()?;
        if let Some(prev) = last {
            if next == prev {
                break next;
            }
        }
        last = Some(next);
    };
    shutdown.shutdown()?;
    Ok(format!("{:?}", double))
}

fn run_network(input: &str) -> Result<(mpsc::Receiver<i64>, Shutdown)> {
    let intcode = parse_intcode(input)?;
    let net = Network::new();
    let (tx, rx) = mpsc::channel();
    let (nat_tx, nat_rx) = mpsc::channel();
    let shutdown = Arc::new(AtomicBool::new(false));
    for cpu in 0..NUM_CPU {
        let nic = Nic::new(cpu as i64, &net);
        let mut ic = IntCode::new_from_device(intcode.clone(), nic);
        let my_tx = tx.clone();
        let my_shutdown = Arc::clone(&shutdown);
        std::thread::spawn(move || {
            let mut last_check = std::time::Instant::now();
            let r = loop {
                if last_check.elapsed() > SLEEP_INTERVAL {
                    if my_shutdown.load(Ordering::Relaxed) {
                        break Ok(());
                    }
                    last_check = std::time::Instant::now();
                }
                if ic.halt {
                    log::warn!("CPU {} HALT", cpu);
                    break Ok(());
                }
                match ic.run_one() {
                    Ok(_) => {}
                    Err(e) => break Err(e),
                }
            };
            match r {
                Ok(_) => {}
                Err(e) => log::error!("cpu {} error: {}", cpu, e),
            }
            my_tx
                .send(cpu)
                .expect(&format!("cpu {} failed to send complete", cpu));
        });
    }
    let nat_shutdown = Arc::clone(&shutdown);
    let mut nat = Nat::new(&net);
    let my_tx = tx.clone();
    std::thread::spawn(move || {
        loop {
            if nat_shutdown.load(Ordering::Relaxed) {
                break;
            }
            if let Some(p) = nat.check_update() {
                nat_tx.send(p.y).expect("failed to send nat result");
            }
            std::thread::sleep(SLEEP_INTERVAL);
        }
        my_tx
            .send(nat.id as usize)
            .expect("could not send nat complete");
    });
    Ok((
        nat_rx,
        Shutdown {
            count: NUM_CPU + 1,
            cpu_rx: rx,
            shutdown,
        },
    ))
}

struct Shutdown {
    count: usize,
    cpu_rx: mpsc::Receiver<usize>,
    shutdown: Arc<AtomicBool>,
}
impl Shutdown {
    fn shutdown(&mut self) -> Result<()> {
        self.shutdown.store(true, Ordering::Relaxed);
        for _ in 0..self.count {
            self.cpu_rx.recv()?;
        }
        Ok(())
    }
}

mod nic {
    use super::network::{Network, Packet};
    use super::SLEEP_INTERVAL;
    use crate::intcode::intcode_io::{Input, Output};

    pub(crate) struct Nat {
        pub id: i64,
        net: Network,
        last: Option<Packet>,
    }

    impl Nat {
        pub(crate) fn new(network: &Network) -> Nat {
            Nat {
                id: 255,
                net: network.clone(),
                last: None,
            }
        }
        fn update(&mut self) {
            if let Some(last) = self.net.last(self.id) {
                self.last = Some(last)
            }
        }
        pub(crate) fn check_update(&mut self) -> Option<Packet> {
            if self.net.idle(self.id) {
                self.update();
                if let Some(p) = self.last.take() {
                    self.net.send(0, p);
                    return Some(p);
                }
            }
            None
        }
    }

    #[derive(Clone, Copy)]
    enum SendState {
        Ready,
        Addr(i64),
        Buffered(i64, i64),
    }

    impl SendState {
        fn push(&mut self, input: i64) -> Option<(i64, Packet)> {
            match *self {
                SendState::Ready => {
                    *self = SendState::Addr(input);
                    None
                }
                SendState::Addr(addr) => {
                    *self = SendState::Buffered(addr, input);
                    None
                }
                SendState::Buffered(addr, x) => {
                    *self = SendState::Ready;
                    Some((addr, Packet { x, y: input }))
                }
            }
        }
    }

    pub(crate) struct Nic {
        id: i64,
        init: bool,
        read_buffer: Option<i64>,
        send_state: SendState,
        network: Network,
    }
    impl Nic {
        pub(crate) fn new(id: i64, network: &Network) -> Nic {
            Nic {
                id,
                init: false,
                read_buffer: None,
                send_state: SendState::Ready,
                network: network.clone(),
            }
        }
    }

    impl Input for Nic {
        fn input(&mut self) -> anyhow::Result<crate::intcode::Int> {
            Ok(if !self.init {
                self.init = true;
                self.id
            } else if let Some(y) = self.read_buffer.take() {
                y
            } else if let Some(Packet { x, y }) = self.network.recv(self.id) {
                self.read_buffer = Some(y);
                x
            } else {
                std::thread::sleep(SLEEP_INTERVAL);
                -1
            })
        }
    }
    impl Output for Nic {
        fn output(&mut self, out: crate::intcode::Int) -> anyhow::Result<()> {
            if let Some((addr, packet)) = self.send_state.push(out) {
                self.network.send(addr, packet);
            }
            Ok(())
        }
    }
}

mod network {
    use std::collections::{HashMap, VecDeque};
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, Copy)]
    pub(crate) struct Packet {
        pub(crate) x: i64,
        pub(crate) y: i64,
    }

    #[derive(Default)]
    pub(crate) struct NetStream {
        inner: VecDeque<Packet>,
        idle: bool,
    }

    impl NetStream {
        pub(crate) fn insert(&mut self, data: Packet) {
            self.inner.push_back(data);
            self.idle = false;
        }
        pub(crate) fn read(&mut self) -> Option<Packet> {
            let p = self.inner.pop_front();
            if p.is_none() {
                self.idle = true;
            }
            p
        }
        pub(crate) fn last(&mut self) -> Option<Packet> {
            let last = self.inner.pop_back();
            self.inner.clear();
            self.idle = true;
            last
        }
    }

    #[derive(Default, Clone)]
    pub(crate) struct Network {
        inner: Arc<Mutex<HashMap<i64, NetStream>>>,
    }
    impl Network {
        pub(crate) fn new() -> Self {
            Network {
                inner: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        pub(crate) fn send(&self, addr: i64, packet: Packet) {
            let mut map = self.inner.lock().expect("bad mutex");
            let stream = map.entry(addr).or_insert_with(|| NetStream::default());
            stream.insert(packet)
        }
        pub(crate) fn recv(&self, addr: i64) -> Option<Packet> {
            // let addr = addr + 1;
            let mut map = self.inner.lock().expect("bad mutex");
            map.get_mut(&addr).and_then(|stream| stream.read())
        }
        pub(crate) fn last(&self, addr: i64) -> Option<Packet> {
            let mut map = self.inner.lock().expect("bad mutex");
            map.get_mut(&addr).and_then(|stream| stream.last())
        }
        pub(crate) fn idle(&self, addr: i64) -> bool {
            let map = self.inner.lock().expect("bad mutex");
            map.iter().all(|(id, s)| *id == addr || s.idle)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::challenges::test::*;

    #[test]
    fn verify_part1() {
        assert_eq!(part1(DAY23_INPUT).unwrap().as_str(), "17714")
    }

    #[test]
    fn verify_part2() {
        assert_eq!(part2(DAY23_INPUT).unwrap().as_str(), "10982")
    }
}
