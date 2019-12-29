use super::Int;
use anyhow::{anyhow as ah, Context, Result};
use std::ops::DerefMut;

pub trait Input {
    fn input(&mut self) -> Result<Int>;
}

pub trait Output {
    fn output(&mut self, out: Int) -> Result<()>;
}

impl Input for Box<dyn Input + Send> {
    fn input(&mut self) -> Result<Int> {
        self.deref_mut().input()
    }
}
impl Output for Box<dyn Output + Send> {
    fn output(&mut self, out: Int) -> Result<()> {
        self.deref_mut().output(out)
    }
}

#[derive(Debug)]
pub struct NullIO;

impl Input for NullIO {
    fn input(&mut self) -> Result<Int> {
        Ok(0)
    }
}

impl Output for NullIO {
    fn output(&mut self, out: Int) -> Result<()> {
        trace!(slog_scope::logger(), "NullIO output => {}", out);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VecIO {
    inner: Vec<Int>,
}

impl VecIO {
    pub fn input(mut input: Vec<Int>) -> VecIO {
        input.reverse();
        VecIO { inner: input }
    }
    pub fn into_vec(self) -> Vec<Int> {
        self.inner
    }
}

impl Input for VecIO {
    fn input(&mut self) -> Result<Int> {
        self.inner.pop().ok_or_else(|| ah!("no more input"))
    }
}

impl Output for VecIO {
    fn output(&mut self, out: Int) -> Result<()> {
        trace!(slog_scope::logger(), "VecIO output => {}", out);
        self.inner.push(out);
        Ok(())
    }
}

#[derive(Debug)]
pub struct StreamInput {
    inner: std::sync::mpsc::Receiver<Int>,
}

impl Input for StreamInput {
    fn input(&mut self) -> Result<Int> {
        self.inner.recv().context("input stream closed")
    }
}

#[derive(Debug)]
pub struct StreamOutput {
    inner: std::sync::mpsc::Sender<Int>,
}

impl Output for StreamOutput {
    fn output(&mut self, out: Int) -> Result<()> {
        self.inner.send(out).context("output stream closed")
    }
}

#[derive(Debug)]
pub struct MultiIO<U, V> {
    first: U,
    second: V,
    first_done: bool,
}

impl<U, V> MultiIO<U, V> {
    pub fn new(first: U, second: V) -> MultiIO<U, V> {
        MultiIO {
            first,
            second,
            first_done: false,
        }
    }
    pub fn split(self) -> (U, V) {
        (self.first, self.second)
    }
}

impl<U: Input, V: Input> Input for MultiIO<U, V> {
    fn input(&mut self) -> Result<Int> {
        if !self.first_done {
            match self.first.input() {
                Result::Ok(x) => return Ok(x),
                Result::Err(_) => self.first_done = true,
            }
        }
        self.second.input()
    }
}

impl<U: Output, V: Output> Output for MultiIO<U, V> {
    fn output(&mut self, out: Int) -> Result<()> {
        let mut msg_send = false;
        let log = slog_scope::logger();
        if let Err(e) = self.first.output(out) {
            info!(log, "output stream failed: {}", e);
        } else {
            msg_send = true;
        }
        if let Err(e) = self.second.output(out) {
            info!(log, "output stream failed: {}", e);
        } else {
            msg_send = true;
        }
        if !msg_send {
            return Err(ah!("both outputs failed"));
        }
        Ok(())
    }
}

pub fn create_stream_io() -> (StreamInput, StreamOutput) {
    let (tx, rx) = std::sync::mpsc::channel();
    let input = StreamInput { inner: rx };
    let output = StreamOutput { inner: tx };
    (input, output)
}

pub fn create_stream_tap() -> (StreamOutput, std::sync::mpsc::Receiver<Int>) {
    let (tx, rx) = std::sync::mpsc::channel();
    let output = StreamOutput { inner: tx };
    (output, rx)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn fuse_inputs() {
        let f = VecIO::input(vec![1]);
        let s = NullIO;
        let mut fused = MultiIO::new(f, s);
        assert_eq!(fused.input().unwrap(), 1);
        assert_eq!(fused.input().unwrap(), 0);
        for _ in 0..100 {
            assert_eq!(fused.input().unwrap(), 0);
        }
    }

    #[test]
    fn fuse_inputs_multi_vec() {
        let v1 = VecIO::input(vec![0, 1, 2]);
        let v2 = VecIO::input(vec![3, 4, 5]);
        let mut fused = MultiIO::new(v1, v2);
        for expected in 0..6 {
            assert_eq!(fused.input().unwrap(), expected);
        }
        assert!(fused.input().is_err());
    }

    #[test]
    fn split_output() {
        let data = vec![3, 1, 4, 1, 5, 9];
        let mut output = MultiIO::new(VecIO::default(), VecIO::default());
        for x in &data {
            output.output(*x).unwrap();
        }
        let (vio1, vio2) = output.split();
        assert_eq!(vio1.into_vec(), data);
        assert_eq!(vio2.into_vec(), data);
    }
}
