//! An async-std version Notify, like tokio Notify but implement Clone.

use async_std::sync::{channel, Receiver, Sender};
use futures_util::select;
use futures_util::FutureExt;

/// Notify a single task to wake up.
///
/// `Notify` provides a basic mechanism to notify a single task of an event.
/// `Notify` itself does not carry any data. Instead, it is to be used to signal
/// another task to perform an operation.
///
/// If [`notify()`] is called **before** [`notified().await`], then the next call to
/// [`notified().await`] will complete immediately, consuming the permit. Any
/// subsequent calls to [`notified().await`] will wait for a new permit.
///
/// If [`notify()`] is called **multiple** times before [`notified().await`], only a
/// **single** permit is stored. The next call to [`notified().await`] will
/// complete immediately, but the one after will wait for a new permit.
///
/// [`notify()`]: Notify::notify
/// [`notified().await`]: Notify::notified()
///
/// # Examples
///
/// Basic usage.
///
/// ```
/// use async_notify::Notify;
///
/// #[async_std::main]
/// async fn main() {
///     let notify = Notify::new();
///     let notify2 = notify.clone();
///
///     async_std::task::spawn(async move {
///         notify2.notified().await;
///         println!("received notification");
///     });
///
///     println!("sending notification");
///     notify.notify();
/// }
/// ```
pub struct Notify {
    sender: Sender<()>,
    receiver: Receiver<()>,
}

/// Like tokio Notify, this is a async-std version Notify and implement Clone.
impl Notify {
    pub fn new() -> Self {
        let (sender, receiver) = channel(1);

        Self { sender, receiver }
    }

    /// Notifies a waiting task
    ///
    /// If a task is currently waiting, that task is notified. Otherwise, a
    /// permit is stored in this `Notify` value and the **next** call to
    /// [`notified().await`] will complete immediately consuming the permit made
    /// available by this call to `notify()`.
    ///
    /// At most one permit may be stored by `Notify`. Many sequential calls to
    /// `notify` will result in a single permit being stored. The next call to
    /// `notified().await` will complete immediately, but the one after that
    /// will wait.
    ///
    /// [`notified().await`]: Notify::notified()
    ///
    /// # Examples
    ///
    /// ```
    /// use async_notify::Notify;
    ///
    /// #[async_std::main]
    /// async fn main() {
    ///     let notify = Notify::new();
    ///     let notify2 = notify.clone();
    ///
    ///     async_std::task::spawn(async move {
    ///         notify2.notified().await;
    ///         println!("received notification");
    ///     });
    ///
    ///     println!("sending notification");
    ///     notify.notify();
    /// }
    /// ```
    pub fn notify(&self) {
        select! {
            _ = self.sender.send(()).fuse() => (),
            default => (),
        }
    }

    /// Wait for a notification.
    ///
    /// Each `Notify` value holds a single permit. If a permit is available from
    /// an earlier call to [`notify()`], then `notified().await` will complete
    /// immediately, consuming that permit. Otherwise, `notified().await` waits
    /// for a permit to be made available by the next call to `notify()`.
    ///
    /// [`notify()`]: Notify::notify
    ///
    /// # Examples
    ///
    /// ```
    /// use async_notify::Notify;
    ///
    /// #[async_std::main]
    /// async fn main() {
    ///     let notify = Notify::new();
    ///     let notify2 = notify.clone();
    ///
    ///     async_std::task::spawn(async move {
    ///         notify2.notified().await;
    ///         println!("received notification");
    ///     });
    ///
    ///     println!("sending notification");
    ///     notify.notify();
    /// }
    /// ```
    pub async fn notified(&self) {
        // Option never be None because sender and receiver always stay together.
        self.receiver.recv().await;
    }
}

impl Default for Notify {
    fn default() -> Notify {
        Notify::new()
    }
}

impl Clone for Notify {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures_util::select;
    use futures_util::FutureExt;

    use super::*;

    #[async_std::test]
    async fn test() {
        let notify = Notify::new();
        let notify2 = notify.clone();

        notify.notify();

        select! {
            _ = notify2.notified().fuse() => (),
            default => panic!("should be notified")
        }
    }
}
