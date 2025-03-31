pub mod memory;

#[cfg(test)]
mod tests {
    use crate::{open_process, Auto, ProcessBlock, ToProcessBlock};

    #[test]
    fn it_works() {
        let handle = open_process(13900).unwrap();

        let process_block: ProcessBlock<Auto> = handle.into_process_block();

        let memory_block = process_block.memory_block(vec![0x6A9EC0,0x768,0x5560]);

        memory_block.write(4096).unwrap();
    }
}

include!("process.rs");