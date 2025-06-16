use anyhow::Result;
use std::future::Future;

pub fn error_chain_string(error: anyhow::Error) -> String {
    let mut message = String::new();

    let mut chain_iter = error.chain();

    message += &format!("Error: {}\n", chain_iter.next().unwrap());

    for err in chain_iter {
        message += &format!("\nCaused by:\n\t{err}");
    }

    message
}

pub async fn error_popup(description: String) {
    rfd::AsyncMessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_title("An Error has occurred")
        .set_level(rfd::MessageLevel::Error)
        .set_description(description)
        .show()
        .await;
}

pub fn blocking_error_popup(description: String) {
    rfd::MessageDialog::new()
        .set_buttons(rfd::MessageButtons::Ok)
        .set_title("An Error has occurred")
        .set_level(rfd::MessageLevel::Error)
        .set_description(description)
        .show();
}

pub async fn async_popup_wrapper<T>(fut: impl Future<Output = Result<T>>) -> Option<T> {
    match fut.await {
        Err(error) => {
            error_popup(error_chain_string(error)).await;
            None
        }
        Ok(x) => Some(x),
    }
}

pub fn failing_task<T: fm_core::MaybeSend + 'static>(
    fut: impl Future<Output = Result<T>> + fm_core::MaybeSend + 'static,
) -> iced::Task<T> {
    iced::Task::future(async { async_popup_wrapper(fut).await }).and_then(iced::Task::done)
}
