use deno_core::{self, error::AnyError, extension, op, Op, OpState};
pub trait Printer {
    fn stdout(&mut self, msg: &str) -> Result<(), AnyError>;
    fn stderr(&mut self, msg: &str) -> Result<(), AnyError>;
}

pub struct SimplePrinter;

impl Printer for SimplePrinter {
    fn stdout(&mut self, msg: &str) -> Result<(), AnyError> {
        println!("[STDOUT] {}", msg.trim_end());
        Ok(())
    }

    fn stderr(&mut self, msg: &str) -> Result<(), AnyError> {
        eprintln!("[STDERR] {}", msg.trim_end());
        Ok(())
    }
}

struct PrinterContainer(Box<dyn Printer>);

extension!(print_extension, parameters = [ P: Printer ], options = {
    printer: P,
}, middleware = |op| match op.name {
    "op_print" => op_print::DECL,
    _ => op
}, state = |state, config| {
    state.put(PrinterContainer(Box::new(config.printer)))
});

#[op]
fn op_print(state: &mut OpState, msg: &str, is_err: bool) -> Result<(), AnyError> {
    if is_err {
        state.borrow_mut::<PrinterContainer>().0.stderr(msg)
    } else {
        state.borrow_mut::<PrinterContainer>().0.stdout(msg)
    }
}
