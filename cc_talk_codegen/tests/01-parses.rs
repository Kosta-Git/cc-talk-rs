use cc_talk_codegen::command;

#[command]
pub struct SimplePollCommand;

#[command]
pub struct SimpleCommandWithData {
    pub array: [u8; 4],
}

pub fn main() {}
