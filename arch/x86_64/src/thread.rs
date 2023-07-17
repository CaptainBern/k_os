use crate::linker;

#[derive(Debug)]
pub struct Tcb<'a> {
    stack: &'a [u8; linker::STACK_SIZE],
}
