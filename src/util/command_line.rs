use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};

use crossbeam::thread;

struct TeeWriter<'a, W0: Write, W1: Write> {
    w0: &'a mut W0,
    w1: &'a mut W1,
}

impl<'a, W0: Write, W1: Write> TeeWriter<'a, W0, W1> {
    fn new(w0: &'a mut W0, w1: &'a mut W1) -> Self {
        Self { w0, w1 }
    }
}

impl<'a, W0: Write, W1: Write> Write for TeeWriter<'a, W0, W1> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // We have to use write_all() otherwise what happens if different
        // amounts are written?
        self.w0.write_all(buf)?;
        self.w1.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.w0.flush()?;
        self.w1.flush()?;
        Ok(())
    }
}

pub fn run_and_capture(command: &mut Command) -> io::Result<(ExitStatus, Vec<u8>, Vec<u8>)> {
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    // These expects should be guaranteed to be ok because we used piped().
    let mut child_stdout = child.stdout.take().expect("logic error getting stdout");
    let mut child_stderr = child.stderr.take().expect("logic error getting stderr");

    thread::scope(|s| {
        let stdout_thread = s.spawn(|_| -> io::Result<Vec<u8>> {
            let stdout = io::stdout();
            let mut stdout = stdout.lock();
            let mut stdout_log = Vec::<u8>::new();
            let mut tee = TeeWriter::new(&mut stdout, &mut stdout_log);
            io::copy(&mut child_stdout, &mut tee)?;
            Ok(stdout_log)
        });
        let stderr_thread = s.spawn(|_| -> io::Result<Vec<u8>> {
            let stderr = io::stderr();
            let mut stderr = stderr.lock();
            let mut stderr_log = Vec::<u8>::new();
            let mut tee = TeeWriter::new(&mut stderr, &mut stderr_log);

            io::copy(&mut child_stderr, &mut tee)?;
            Ok(stderr_log)
        });

        let status = child.wait().expect("child wasn't running");

        let stdout_log = stdout_thread.join().expect("stdout thread panicked")?;
        let stderr_log = stderr_thread.join().expect("stderr thread panicked")?;

        Ok((status, stdout_log, stderr_log))
    })
    .expect("stdout/stderr thread panicked")
}
