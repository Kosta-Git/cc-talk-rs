use cc_talk_codegen::command;
use cc_talk_core::cc_talk::Command;

#[command]
pub struct SimplePollCommand;

fn main() {
    let command = SimplePollCommand {};

    assert_eq!(command.header(), cc_talk_core::cc_talk::Header::SimplePoll);
    assert_eq!(command.data(), &[] as &[u8]);
    assert_eq!(command.parse_response(&[]).unwrap(), ());
}
