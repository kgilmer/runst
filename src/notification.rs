use crate::error::Result;
use serde::Serialize;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tera::Context as TeraContext;

/// Possible urgency levels for the notification.
#[derive(Clone, Debug, Serialize)]
pub enum Urgency {
    /// Low urgency.
    Low,
    /// Normal urgency (default).
    Normal,
    /// Critical urgency.
    Critical,
}

impl From<u64> for Urgency {
    fn from(value: u64) -> Self {
        match value {
            0 => Self::Low,
            1 => Self::Normal,
            2 => Self::Critical,
            _ => Self::default(),
        }
    }
}

impl Default for Urgency {
    fn default() -> Self {
        Self::Normal
    }
}

/// Representation of a notification.
///
/// See [D-Bus Notify Parameters](https://specifications.freedesktop.org/notification-spec/latest/ar01s09.html)
#[derive(Clone, Debug, Default)]
pub struct Notification {
    /// The optional notification ID.
    pub id: u32,
    /// Name of the application that sends the notification.
    pub app_name: String,
    /// Summary text.
    pub summary: String,
    /// Body.
    pub body: String,
    /// The timeout time in milliseconds.
    pub expire_timeout: Option<Duration>,
    /// Urgency.
    pub urgency: Urgency,
    /// Whether if the notification is read.
    pub is_read: bool,
}

impl Notification {
    /// Converts [`Notification`] into [`Context`].
    pub fn into_context<'a>(
        &'a self,
        urgency_text: &'a str,
        unread_count: usize,
    ) -> Result<TeraContext> {
        Ok(TeraContext::from_serialize(Context {
            app_name: &self.app_name,
            summary: &self.summary,
            body: &self.body,
            urgency_text,
            unread_count,
        })?)
    }
}

/// Template context for the notification.
#[derive(Clone, Debug, Default, Serialize)]
struct Context<'a> {
    /// Name of the application that sends the notification.
    pub app_name: &'a str,
    /// Summary text.
    pub summary: &'a str,
    /// Body.
    pub body: &'a str,
    /// Urgency.
    #[serde(rename = "urgency")]
    pub urgency_text: &'a str,
    /// Count of unread notifications.
    pub unread_count: usize,
}

/// Possible actions for a notification.
#[derive(Debug)]
pub enum Action {
    /// Show a notification.
    Show(Notification),
    /// Show the last notification.
    ShowLast,
    /// Close a notification.
    Close(Option<u32>),
    /// Close all the notifications.
    CloseAll,
}

/// Notification manager.
#[derive(Debug)]
pub struct Manager {
    /// Inner type that holds the notifications in thread-safe way.
    inner: Arc<RwLock<Vec<Notification>>>,
}

impl Clone for Manager {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Manager {
    /// Initializes the notification manager.
    pub fn init() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Adds a new notifications to manage.
    pub fn add(&self, notification: Notification) {
        self.inner
            .write()
            .expect("failed to retrieve notifications")
            .push(notification);
    }

    /// Returns the last unread notification.
    pub fn get_last_unread(&self) -> Notification {
        let notifications = self.inner.read().expect("failed to retrieve notifications");
        let notifications = notifications
            .iter()
            .filter(|v| !v.is_read)
            .collect::<Vec<&Notification>>();
        notifications[notifications.len() - 1].clone()
    }

    /// Marks the last notification as read.
    pub fn mark_last_as_read(&self) {
        let mut notifications = self
            .inner
            .write()
            .expect("failed to retrieve notifications");
        if let Some(notification) = notifications.iter_mut().filter(|v| !v.is_read).last() {
            notification.is_read = true;
        }
    }

    /// Marks the next notification as unread starting from the first one.
    ///
    /// Returns true if there is an unread notification remaining.
    pub fn mark_next_as_unread(&self) -> bool {
        let mut notifications = self
            .inner
            .write()
            .expect("failed to retrieve notifications");
        let last_unread_index = notifications.iter_mut().position(|v| !v.is_read);
        if last_unread_index.is_none() {
            let len = notifications.len();
            notifications[len - 1].is_read = false;
        }
        if let Some(index) = last_unread_index {
            notifications[index].is_read = true;
            if index > 0 {
                notifications[index - 1].is_read = false;
            } else {
                return false;
            }
        }
        true
    }

    /// Marks the given notification as read.
    pub fn mark_as_read(&self, id: u32) {
        let mut notifications = self
            .inner
            .write()
            .expect("failed to retrieve notifications");
        if let Some(notification) = notifications
            .iter_mut()
            .find(|notification| notification.id == id)
        {
            notification.is_read = true;
        }
    }

    /// Marks all the notifications as read.
    pub fn mark_all_as_read(&self) {
        let mut notifications = self
            .inner
            .write()
            .expect("failed to retrieve notifications");
        notifications.iter_mut().for_each(|v| v.is_read = true);
    }

    /// Returns the number of unread notifications.
    pub fn get_unread_count(&self) -> usize {
        let notifications = self.inner.read().expect("failed to retrieve notifications");
        notifications.iter().filter(|v| !v.is_read).count()
    }

    /// Returns true if the notification is unread.
    pub fn is_unread(&self, id: u32) -> bool {
        let notifications = self.inner.read().expect("failed to retrieve notifications");
        notifications
            .iter()
            .find(|notification| notification.id == id)
            .map(|v| !v.is_read)
            .unwrap_or_default()
    }
}