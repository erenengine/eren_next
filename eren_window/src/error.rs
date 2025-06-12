use native_dialog::{DialogBuilder, MessageLevel};

pub fn show_error_popup(message: &str) {
    DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title("Error")
        .set_text(message)
        .alert()
        .show()
        .unwrap();
}

pub fn handle_fatal_error<E: std::fmt::Display>(error: E, context: &str) -> ! {
    let message = format!("{}: {}", context, error);
    show_error_popup(&message);
    panic!("{}", message);
}
