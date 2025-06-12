use native_dialog::{DialogBuilder, MessageLevel};

pub fn show_error_popup<E: std::fmt::Display>(error: E, context: &str) {
    DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(context)
        .set_text(error.to_string())
        .alert()
        .show()
        .unwrap();
}

pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    show_error_popup(&error, context);
    panic!("{}: {}", context, error);
}
